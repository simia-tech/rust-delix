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

pub struct Connection {
    stream: net::TcpStream,
    thread: Option<thread::JoinHandle<()>>,

    node_id: ID,
    peer_node_id: ID,
    peer_public_address: SocketAddr,

    on_services: Arc<Mutex<Option<Box<Fn(ID, Vec<String>) + Send>>>>,
    on_request: Arc<Mutex<Option<Box<Fn(&str, &[u8]) -> request::Response + Send>>>>,
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
}

impl Connection {
    pub fn new_inbound(s: net::TcpStream,
                       node_id: ID,
                       public_address: SocketAddr,
                       peers: &[(ID, SocketAddr)])
                       -> Result<Connection> {

        let (mut connection, sender) = try!(Self::new(s, node_id, public_address));

        try!(write_peers(&mut connection.stream, peers));
        sender.send(true).unwrap();

        Ok(connection)
    }

    pub fn new_outbound(s: net::TcpStream,
                        node_id: ID,
                        public_address: SocketAddr)
                        -> Result<(Connection, Vec<(ID, SocketAddr)>)> {

        let (mut connection, sender) = try!(Self::new(s, node_id, public_address));

        let container = try!(read_container(&mut connection.stream));
        let peers = try!(read_peers(&container));
        sender.send(true).unwrap();

        Ok((connection, peers))
    }

    fn new(s: net::TcpStream,
           node_id: ID,
           public_address: SocketAddr)
           -> Result<(Connection, mpsc::Sender<bool>)> {

        let mut stream = s.try_clone().unwrap();

        let on_services: Arc<Mutex<Option<Box<Fn(ID, Vec<String>) + Send>>>> =
            Arc::new(Mutex::new(None));
        let on_services_clone = on_services.clone();

        let on_request: Arc<Mutex<Option<Box<Fn(&str, &[u8]) -> request::Response + Send>>>> =
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
                let container = match read_container(&mut stream) {
                    Ok(container) => container,
                    Err(Error::ConnectionLost) => {
                        if let Some(ref f) = *on_shutdown_clone.lock().unwrap() {
                            f(peer_node_id);
                        }
                        break;
                    }
                    Err(err) => {
                        println!("{}: error reading connection: {:?}", node_id, err);
                        break;
                    }
                };
                match container.get_kind() {
                    message::Kind::ServicesMessage => {
                        if let Some(ref f) = *on_services_clone.lock().unwrap() {
                            f(peer_node_id, read_services(&container).unwrap());
                        }
                    }
                    message::Kind::RequestMessage => {
                        if let Some(ref f) = *on_request_clone.lock().unwrap() {
                            let (request_id, name, data) = read_request(&container).unwrap();
                            write_response(&mut stream, request_id, f(&name, &data)).unwrap();
                        }
                    }
                    message::Kind::ResponseMessage => {
                        if let Some(ref f) = *on_response_clone.lock().unwrap() {
                            let (request_id, response) = read_response(&container).unwrap();
                            let response = match response {
                                Err(request::Error::ServiceDoesNotExists) => Err(request::Error::ServiceDoesNotExistsOnPeer(peer_node_id.clone())),
                                _ => response,
                            };
                            f(request_id, response);
                        }
                    }
                    _ => {
                        println!("{}: got unexpected container {:?}", node_id, container);
                    }
                }
            }
        }));

        Ok((Connection {
            stream: s,
            thread: thread,
            node_id: node_id,
            peer_node_id: peer_node_id,
            peer_public_address: peer_public_address,
            on_services: on_services,
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
        self.stream.peer_addr().ok()
    }

    pub fn local_address(&self) -> Option<SocketAddr> {
        self.stream.local_addr().ok()
    }

    pub fn send_services(&mut self, service_names: &[String]) -> Result<()> {
        try!(write_services(&mut self.stream, service_names));
        Ok(())
    }

    pub fn set_on_services(&mut self, f: Box<Fn(ID, Vec<String>) + Send>) {
        *self.on_services.lock().unwrap() = Some(f);
    }

    pub fn send_request(&mut self, id: u32, name: &str, data: &[u8]) -> Result<()> {
        try!(write_request(&mut self.stream, id, name, data));
        Ok(())
    }

    pub fn set_on_request(&mut self, f: Box<Fn(&str, &[u8]) -> request::Response + Send>) {
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
        self.stream.shutdown(net::Shutdown::Both).unwrap();
        self.thread.take().unwrap().join().unwrap();

        if let Some(ref f) = *self.on_drop.lock().unwrap() {
            f(self.peer_node_id);
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

fn write_services(w: &mut Write, service_names: &[String]) -> Result<()> {
    let mut buffer = Vec::new();
    let mut services_packet = message::Services::new();
    for service_name in service_names {
        let mut service_packet = message::Service::new();
        service_packet.set_name((*service_name).to_string());
        services_packet.mut_services().push(service_packet);
    }
    try!(services_packet.write_to_vec(&mut buffer));
    write_container(w, message::Kind::ServicesMessage, buffer)
}

fn write_request(w: &mut Write, id: u32, name: &str, data: &[u8]) -> Result<()> {
    let mut buffer = Vec::new();
    let mut request_packet = message::Request::new();
    request_packet.set_id(id);
    request_packet.set_name(name.to_string());
    request_packet.set_data(data.to_vec());
    try!(request_packet.write_to_vec(&mut buffer));
    write_container(w, message::Kind::RequestMessage, buffer)
}

fn write_response(w: &mut Write, request_id: u32, response: request::Response) -> Result<()> {
    let mut buffer = Vec::new();
    let mut response_packet = message::Response::new();
    response_packet.set_request_id(request_id);
    match response {
        Ok(data) => {
            response_packet.set_kind(message::Response_Kind::OK);
            response_packet.set_data(data);
        }
        Err(request::Error::ServiceDoesNotExists) => {
            response_packet.set_kind(message::Response_Kind::ServiceDoesNotExists);
        }
        Err(request::Error::ServiceDoesNotExistsOnPeer(id)) => {
            response_packet.set_kind(message::Response_Kind::ServiceDoesNotExistsOnPeer);
            response_packet.set_data(id.to_vec());
        }
        Err(request::Error::Timeout) => {
            response_packet.set_kind(message::Response_Kind::Timeout);
        }
        Err(request::Error::Internal(message)) => {
            response_packet.set_kind(message::Response_Kind::Internal);
            response_packet.set_data(message.bytes().collect());
        }
    }
    try!(response_packet.write_to_vec(&mut buffer));
    write_container(w, message::Kind::ResponseMessage, buffer)
}

fn write_container(w: &mut Write, kind: message::Kind, data: Vec<u8>) -> Result<()> {
    let mut container = message::Container::new();
    container.set_kind(kind);
    container.set_payload(data);

    let container_bytes = try!(container.write_to_bytes());
    let container_size = container_bytes.len() as u64;

    w.write_u64::<byteorder::BigEndian>(container_size).unwrap();
    w.write(&container_bytes).unwrap();
    w.flush().unwrap();

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

fn read_services(container: &message::Container) -> Result<Vec<String>> {
    Ok(try!(read_packet::<message::Services>(&container))
           .get_services()
           .to_vec()
           .iter()
           .map(|service_packet| service_packet.get_name().to_string())
           .collect())
}

fn read_request(container: &message::Container) -> Result<(u32, String, Vec<u8>)> {
    let request_packet = try!(read_packet::<message::Request>(&container));
    Ok((request_packet.get_id(),
        request_packet.get_name().to_string(),
        request_packet.get_data().to_vec()))
}

fn read_response(container: &message::Container) -> Result<(u32, request::Response)> {
    let response_packet = try!(read_packet::<message::Response>(&container));
    let result = match response_packet.get_kind() {
        message::Response_Kind::OK => Ok(response_packet.get_data().to_vec()),
        message::Response_Kind::ServiceDoesNotExists => Err(request::Error::ServiceDoesNotExists),
        message::Response_Kind::ServiceDoesNotExistsOnPeer => Err(request::Error::ServiceDoesNotExistsOnPeer(ID::from_vec(response_packet.get_data().to_vec()).unwrap())),
        message::Response_Kind::Timeout => Err(request::Error::Timeout),
        message::Response_Kind::Internal => {
            Err(request::Error::Internal(String::from_utf8(response_packet.get_data().to_vec())
                                             .unwrap()))
        }
    };
    Ok((response_packet.get_request_id(), result))
}

fn read_packet<T: protobuf::Message + protobuf::MessageStatic>(container: &message::Container)
                                                               -> Result<T> {
    Ok(try!(protobuf::parse_from_bytes::<T>(container.get_payload())))
}

fn read_container(r: &mut Read) -> Result<message::Container> {
    let container_size = match r.read_u64::<byteorder::BigEndian>() {
        Ok(number) => number as usize,
        Err(byteorder::Error::UnexpectedEOF) => return Err(Error::ConnectionLost),
        Err(byteorder::Error::Io(error)) => return Err(Error::Io(error)),
    };

    let mut buffer = Vec::with_capacity(container_size);
    unsafe {
        buffer.set_len(container_size);
    }
    r.read(&mut buffer).unwrap();

    Ok(try!(protobuf::parse_from_bytes::<message::Container>(&buffer)))
}
