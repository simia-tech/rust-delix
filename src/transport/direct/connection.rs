// Copyright 2015 The Delix Project Authors. See the AUTHORS file at the top level directory.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.
//

use std::fmt;
use std::io::{self, Read, Write};
use std::net::{self, SocketAddr};
use std::result;
use std::sync::{Arc, Mutex, mpsc};
use std::thread;

use protobuf::{self, Message};
use message;
use byteorder::{self, WriteBytesExt, ReadBytesExt};

use node::{self, ID, request};
use transport::cipher::{self, Cipher};
use util::{reader, writer};

pub struct Connection {
    stream: cipher::Stream,
    thread: Option<thread::JoinHandle<()>>,

    node_id: ID,
    peer_node_id: ID,
    peer_public_address: SocketAddr,

    on_add_services: Arc<Mutex<Option<Box<Fn(ID, Vec<String>) + Send>>>>,
    on_remove_services: Arc<Mutex<Option<Box<Fn(ID, Vec<String>) + Send>>>>,
    on_request: Arc<Mutex<Option<Box<Fn(&str, Box<request::Reader>) -> request::Response + Send>>>>,
    on_response: Arc<Mutex<Option<Box<Fn(u32, request::Response) + Send>>>>,
    on_shutdown: Arc<Mutex<Option<Box<Fn(ID) + Send>>>>,
    on_drop: Arc<Mutex<Option<Box<Fn(ID) + Send>>>>,
}

pub type Result<T> = result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    ConnectionLost,
    Id(node::IDError),
    AddrParse(net::AddrParseError),
    Protobuf(protobuf::ProtobufError),
    Io(io::Error),
    Cipher(cipher::Error),
}

impl Connection {
    pub fn new_inbound(tcp_stream: net::TcpStream,
                       cipher: Arc<Box<Cipher>>,
                       node_id: ID,
                       public_address: SocketAddr,
                       peers: &[(ID, SocketAddr)])
                       -> Result<Connection> {

        let mut stream = cipher::Stream::new(tcp_stream, cipher.box_clone());

        let (connection, sender) = try!(Self::new(stream.try_clone().unwrap(),
                                                  node_id,
                                                  public_address));

        try!(write_peers(&mut stream, peers));
        sender.send(true).unwrap();

        Ok(connection)
    }

    pub fn new_outbound(tcp_stream: net::TcpStream,
                        cipher: Arc<Box<Cipher>>,
                        node_id: ID,
                        public_address: SocketAddr)
                        -> Result<(Connection, Vec<(ID, SocketAddr)>)> {

        let mut stream = cipher::Stream::new(tcp_stream, cipher.box_clone());

        let (connection, sender) = try!(Self::new(stream.try_clone().unwrap(),
                                                  node_id,
                                                  public_address));

        let container = try!(read_container(&mut stream));
        let peers = try!(read_peers(&container));
        sender.send(true).unwrap();

        Ok((connection, peers))
    }

    fn new(mut stream: cipher::Stream,
           node_id: ID,
           public_address: SocketAddr)
           -> Result<(Connection, mpsc::Sender<bool>)> {

        let mut stream_clone = stream.try_clone().unwrap();

        let on_add_services: Arc<Mutex<Option<Box<Fn(ID, Vec<String>) + Send>>>> =
            Arc::new(Mutex::new(None));
        let on_add_services_clone = on_add_services.clone();

        let on_remove_services: Arc<Mutex<Option<Box<Fn(ID, Vec<String>) + Send>>>> =
            Arc::new(Mutex::new(None));
        let on_remove_services_clone = on_remove_services.clone();

        let on_request: Arc<Mutex<Option<Box<Fn(&str, Box<request::Reader>) -> request::Response + Send>>>> =
            Arc::new(Mutex::new(None));
        let on_request_clone = on_request.clone();

        let on_response: Arc<Mutex<Option<Box<Fn(u32, request::Response) + Send>>>> =
            Arc::new(Mutex::new(None));
        let on_response_clone = on_response.clone();

        let on_shutdown: Arc<Mutex<Option<Box<Fn(ID) + Send>>>> = Arc::new(Mutex::new(None));
        let on_shutdown_clone = on_shutdown.clone();

        try!(write_introduction(&mut stream, node_id, public_address));

        let container = try!(read_container(&mut stream));
        let (peer_node_id, peer_public_address) = try!(read_introduction(&container));

        let (sender, receiver) = mpsc::channel();
        let thread = Some(thread::spawn(move || {
            receiver.recv().unwrap();
            loop {
                let container = match read_container(&mut stream_clone) {
                    Ok(container) => container,
                    Err(Error::ConnectionLost) => {
                        debug!("{}: connection lost", node_id);
                        if let Some(ref f) = *on_shutdown_clone.lock().unwrap() {
                            f(peer_node_id);
                        }
                        break;
                    }
                    Err(err) => {
                        error!("{}: error reading connection: {:?}", node_id, err);
                        break;
                    }
                };
                match container.get_kind() {
                    message::Kind::AddServicesMessage => {
                        if let Some(ref f) = *on_add_services_clone.lock().unwrap() {
                            f(peer_node_id, read_add_services(&container).unwrap());
                        }
                    }
                    message::Kind::RemoveServicesMessage => {
                        if let Some(ref f) = *on_remove_services_clone.lock().unwrap() {
                            f(peer_node_id, read_remove_services(&container).unwrap());
                        }
                    }
                    message::Kind::RequestMessage => {
                        if let Some(ref f) = *on_request_clone.lock().unwrap() {
                            let (request_id, name) = read_request(&container).unwrap();
                            debug!("{}: request 1", node_id);

                            // let response =
                            // f(&name,
                            // Box::new(reader::Chunk::new(stream_clone.try_clone()
                            // .unwrap())));
                            //

                            // let buffer = {
                            //     let mut buffer = Vec::with_capacity(16);
                            //     unsafe {
                            //         buffer.set_len(16);
                            //     }
                            //
                            //     let number = stream_clone.read(&mut buffer).unwrap();
                            //     debug!("buffer {:?} / {}", buffer, number);
                            //
                            //     buffer
                            // };

                            let buffer = {
                                let mut reader = reader::Chunk::new(&mut stream_clone);
                                let mut buffer = Vec::new();
                                reader.read_to_end(&mut buffer).unwrap();
                                buffer
                            };

                            debug!("{}: request 2 / buffer {}", node_id, buffer.len());
                            write_response(&mut stream_clone,
                                           request_id,
                                           Ok(Box::new(io::Cursor::new(buffer))))
                                .unwrap();
                            debug!("{}: request 3", node_id);
                        }
                    }
                    message::Kind::ResponseMessage => {
                        if let Some(ref f) = *on_response_clone.lock().unwrap() {
                            debug!("{}: got response", node_id);
                            let (request_id, response) = read_response(&container).unwrap();
                            f(request_id, response);
                        }
                    }
                    _ => {
                        error!("{}: got unexpected container {:?}", node_id, container);
                    }
                }
            }
        }));

        Ok((Connection {
            stream: stream,
            thread: thread,
            node_id: node_id,
            peer_node_id: peer_node_id,
            peer_public_address: peer_public_address,
            on_add_services: on_add_services,
            on_remove_services: on_remove_services,
            on_request: on_request,
            on_response: on_response,
            on_shutdown: on_shutdown,
            on_drop: Arc::new(Mutex::new(None)),
        },
            sender))
    }

    pub fn peer_node_id(&self) -> ID {
        self.peer_node_id
    }

    pub fn peer_public_address(&self) -> SocketAddr {
        self.peer_public_address
    }

    pub fn peer_address(&self) -> Option<SocketAddr> {
        self.stream.get_ref().peer_addr().ok()
    }

    pub fn local_address(&self) -> Option<SocketAddr> {
        self.stream.get_ref().local_addr().ok()
    }

    pub fn send_add_services(&mut self, service_names: &[String]) -> Result<()> {
        try!(write_add_services(&mut self.stream, service_names));
        Ok(())
    }

    pub fn set_on_add_services(&mut self, f: Box<Fn(ID, Vec<String>) + Send>) {
        *self.on_add_services.lock().unwrap() = Some(f);
    }

    pub fn send_remove_services(&mut self, service_names: &[String]) -> Result<()> {
        try!(write_remove_services(&mut self.stream, service_names));
        Ok(())
    }

    pub fn set_on_remove_services(&mut self, f: Box<Fn(ID, Vec<String>) + Send>) {
        *self.on_remove_services.lock().unwrap() = Some(f);
    }

    pub fn send_request(&mut self,
                        id: u32,
                        name: &str,
                        reader: &mut request::Reader)
                        -> Result<()> {
        debug!("{}: send 1", self.node_id);
        try!(write_request(&mut self.stream, id, name));
        debug!("{}: send 2", self.node_id);

        assert_eq!(8, self.stream.write(&[0, 0, 0, 0, 0, 0, 0, 16]).unwrap());
        assert_eq!(16, io::copy(reader, &mut self.stream).unwrap());
        assert_eq!(8, self.stream.write(&[0, 0, 0, 0, 0, 0, 0, 0]).unwrap());

        // {
        //     let mut writer = writer::Chunk::new(&mut self.stream);
        //     debug!("{}: send request with {} bytes",
        //            self.node_id,
        //            try!(io::copy(reader, &mut writer)));
        //     self.stream.flush().unwrap();
        // }

        debug!("{}: send 3", self.node_id);
        Ok(())
    }

    pub fn set_on_request(&mut self,
                          f: Box<Fn(&str, Box<request::Reader>) -> request::Response + Send>) {
        *self.on_request.lock().unwrap() = Some(f);
    }

    pub fn set_on_response(&mut self, f: Box<Fn(u32, request::Response) + Send>) {
        *self.on_response.lock().unwrap() = Some(f);
    }

    pub fn set_on_shutdown(&mut self, f: Box<Fn(ID) + Send>) {
        *self.on_shutdown.lock().unwrap() = Some(f);
    }

    pub fn clear_on_shutdown(&mut self) {
        *self.on_shutdown.lock().unwrap() = None;
    }

    pub fn set_on_drop(&mut self, f: Box<Fn(ID) + Send>) {
        *self.on_drop.lock().unwrap() = Some(f);
    }
}

impl fmt::Display for Connection {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if let (Some(local_address), Some(peer_address)) = (self.local_address(),
                                                            self.peer_address()) {
            write!(f,
                   "(Direct connection {} ({}) -> {} ({}))",
                   self.node_id,
                   local_address,
                   self.peer_node_id,
                   peer_address)
        } else {
            write!(f,
                   "(Direct connection {} (-) -> {} (-))",
                   self.node_id,
                   self.peer_node_id)
        }
    }
}

impl Drop for Connection {
    fn drop(&mut self) {
        self.stream.get_ref().shutdown(net::Shutdown::Both).unwrap();
        self.thread.take().unwrap().join().unwrap();

        if let Some(ref f) = *self.on_drop.lock().unwrap() {
            f(self.peer_node_id);
        }
    }
}

impl From<byteorder::Error> for Error {
    fn from(error: byteorder::Error) -> Self {
        match error {
            byteorder::Error::Io(ref err) if err.kind() == io::ErrorKind::Other &&
                                             format!("{}", error) == "unexpected EOF" => {
                Error::ConnectionLost
            }
            byteorder::Error::Io(err) => Error::Io(err),
            byteorder::Error::UnexpectedEOF => Error::ConnectionLost,
        }
    }
}

impl From<node::IDError> for Error {
    fn from(error: node::IDError) -> Self {
        Error::Id(error)
    }
}

impl From<net::AddrParseError> for Error {
    fn from(error: net::AddrParseError) -> Self {
        Error::AddrParse(error)
    }
}

impl From<protobuf::ProtobufError> for Error {
    fn from(error: protobuf::ProtobufError) -> Self {
        Error::Protobuf(error)
    }
}

impl From<io::Error> for Error {
    fn from(error: io::Error) -> Self {
        Error::Io(error)
    }
}

impl From<cipher::Error> for Error {
    fn from(error: cipher::Error) -> Self {
        Error::Cipher(error)
    }
}

fn write_introduction(w: &mut Write, node_id: ID, public_address: SocketAddr) -> Result<()> {
    let mut buffer = Vec::new();
    let mut introduction = message::Introduction::new();
    introduction.set_id(node_id.to_vec());
    introduction.set_public_address(format!("{}", public_address));
    try!(introduction.write_to_vec(&mut buffer));
    write_container(w, message::Kind::IntroductionMessage, buffer)
}

fn write_peers(w: &mut Write, peers: &[(ID, SocketAddr)]) -> Result<()> {
    let mut buffer = Vec::new();
    let mut peers_packet = message::Peers::new();
    for peer in peers {
        let (peer_node_id, peer_public_address) = *peer;
        let mut peer_packet = message::Peer::new();
        peer_packet.set_id(peer_node_id.to_vec());
        peer_packet.set_public_address(format!("{}", peer_public_address));
        peers_packet.mut_peers().push(peer_packet);
    }
    try!(peers_packet.write_to_vec(&mut buffer));
    write_container(w, message::Kind::PeersMessage, buffer)
}

fn write_add_services(w: &mut Write, service_names: &[String]) -> Result<()> {
    let mut buffer = Vec::new();
    let mut services_packet = message::AddServices::new();
    for service_name in service_names {
        let mut service_packet = message::Service::new();
        service_packet.set_name((*service_name).to_string());
        services_packet.mut_services().push(service_packet);
    }
    try!(services_packet.write_to_vec(&mut buffer));
    write_container(w, message::Kind::AddServicesMessage, buffer)
}

fn write_remove_services(w: &mut Write, service_names: &[String]) -> Result<()> {
    let mut buffer = Vec::new();
    let mut services_packet = message::RemoveServices::new();
    for service_name in service_names {
        let mut service_packet = message::Service::new();
        service_packet.set_name((*service_name).to_string());
        services_packet.mut_services().push(service_packet);
    }
    try!(services_packet.write_to_vec(&mut buffer));
    write_container(w, message::Kind::RemoveServicesMessage, buffer)
}

fn write_request(w: &mut Write, id: u32, name: &str) -> Result<()> {
    let mut buffer = Vec::new();
    let mut request_packet = message::Request::new();
    request_packet.set_id(id);
    request_packet.set_name(name.to_string());
    try!(request_packet.write_to_vec(&mut buffer));
    write_container(w, message::Kind::RequestMessage, buffer)
}

fn write_response(w: &mut Write, request_id: u32, response: request::Response) -> Result<()> {
    let mut buffer = Vec::new();
    let mut response_packet = message::Response::new();
    response_packet.set_request_id(request_id);
    match response {
        Ok(mut reader) => {
            let mut data = Vec::new();
            try!(reader.read_to_end(&mut data));
            debug!("response {:?}", data);
            response_packet.set_kind(message::Response_Kind::OK);
            response_packet.set_data(data);
        }
        Err(request::Error::ServiceDoesNotExists) => {
            response_packet.set_kind(message::Response_Kind::ServiceDoesNotExists);
        }
        Err(request::Error::ServiceUnavailable) => {
            response_packet.set_kind(message::Response_Kind::ServiceUnavailable);
        }
        Err(request::Error::Timeout) => {
            response_packet.set_kind(message::Response_Kind::Timeout);
        }
        Err(request::Error::Internal(message)) => {
            response_packet.set_kind(message::Response_Kind::Internal);
            response_packet.set_message(message);
        }
    }
    try!(response_packet.write_to_vec(&mut buffer));
    write_container(w, message::Kind::ResponseMessage, buffer)
}

fn write_container(w: &mut Write, kind: message::Kind, data: Vec<u8>) -> Result<()> {
    let mut container = message::Container::new();
    container.set_kind(kind);
    container.set_payload(data);

    let bytes = try!(container.write_to_bytes());
    let size = bytes.len() as u64;

    try!(w.write_u64::<byteorder::BigEndian>(size));
    try!(w.write(&bytes));
    try!(w.flush());

    Ok(())
}

fn read_introduction(container: &message::Container) -> Result<(ID, SocketAddr)> {
    let introduction_packet = try!(read_packet::<message::Introduction>(&container));
    Ok((try!(ID::from_vec(introduction_packet.get_id().to_vec())),
        try!(introduction_packet.get_public_address()
                                .parse::<SocketAddr>())))
}

fn read_peers(container: &message::Container) -> Result<Vec<(ID, SocketAddr)>> {
    Ok(try!(read_packet::<message::Peers>(&container))
           .get_peers()
           .iter()
           .map(|peer_packet| {
               (ID::from_vec(peer_packet.get_id().to_vec()).unwrap(),
                peer_packet.get_public_address()
                           .parse::<SocketAddr>()
                           .unwrap())
           })
           .collect())
}

fn read_add_services(container: &message::Container) -> Result<Vec<String>> {
    Ok(try!(read_packet::<message::AddServices>(&container))
           .get_services()
           .to_vec()
           .iter()
           .map(|service_packet| service_packet.get_name().to_string())
           .collect())
}

fn read_remove_services(container: &message::Container) -> Result<Vec<String>> {
    Ok(try!(read_packet::<message::RemoveServices>(&container))
           .get_services()
           .to_vec()
           .iter()
           .map(|service_packet| service_packet.get_name().to_string())
           .collect())
}

fn read_request(container: &message::Container) -> Result<(u32, String)> {
    let request_packet = try!(read_packet::<message::Request>(&container));
    Ok((request_packet.get_id(),
        request_packet.get_name().to_string()))
}

fn read_response(container: &message::Container) -> Result<(u32, request::Response)> {
    let response_packet = try!(read_packet::<message::Response>(&container));
    let result = match response_packet.get_kind() {
        message::Response_Kind::OK => {
            Ok(Box::new(io::Cursor::new(response_packet.get_data().to_vec())) as Box<io::Read + Send>)
        }
        message::Response_Kind::ServiceDoesNotExists => Err(request::Error::ServiceDoesNotExists),
        message::Response_Kind::ServiceUnavailable => Err(request::Error::ServiceUnavailable),
        message::Response_Kind::Timeout => Err(request::Error::Timeout),
        message::Response_Kind::Internal => {
            Err(request::Error::Internal(response_packet.get_message().to_string()))
        }
    };
    Ok((response_packet.get_request_id(), result))
}

fn read_packet<T: protobuf::Message + protobuf::MessageStatic>(container: &message::Container)
                                                               -> Result<T> {
    Ok(try!(protobuf::parse_from_bytes::<T>(container.get_payload())))
}

fn read_container(stream: &mut io::Read) -> Result<message::Container> {
    let size = try!(stream.read_u64::<byteorder::BigEndian>()) as usize;

    let mut bytes = Vec::with_capacity(size);
    unsafe {
        bytes.set_len(size);
    }
    assert_eq!(size, try!(stream.read(&mut bytes)));

    Ok(try!(protobuf::parse_from_bytes::<message::Container>(&bytes)))
}
