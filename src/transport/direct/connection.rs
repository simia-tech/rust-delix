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

use protobuf::{self, Message, MessageStatic, parse_from_bytes};
use message::{Container, Kind, Introduction, Peers, Peer, Request, Response, Response_Kind,
              Services, Service};
use byteorder::{self, WriteBytesExt, ReadBytesExt};

use node::{self, ID};

pub struct Connection {
    stream: net::TcpStream,
    thread: Option<thread::JoinHandle<()>>,

    node_id: ID,
    peer_node_id: ID,
    peer_public_address: SocketAddr,

    on_services: Arc<Mutex<Option<Box<Fn(ID, Vec<String>) + Send>>>>,
    on_request: Arc<Mutex<Option<Box<Fn(&str, &[u8]) -> Result<Vec<u8>> + Send>>>>,
    on_response: Arc<Mutex<Option<Box<Fn(u32, Result<Vec<u8>>) + Send>>>>,
    on_shutdown: Arc<Mutex<Option<mpsc::Sender<ID>>>>,
}

pub type Result<T> = result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    ConnectionLost,
    ServiceDoesNotExists,
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

        let on_request: Arc<Mutex<Option<Box<Fn(&str, &[u8]) -> Result<Vec<u8>> + Send>>>> =
            Arc::new(Mutex::new(None));
        let on_request_clone = on_request.clone();

        let on_response: Arc<Mutex<Option<Box<Fn(u32, Result<Vec<u8>>) + Send>>>> =
            Arc::new(Mutex::new(None));
        let on_response_clone = on_response.clone();

        let on_shutdown: Arc<Mutex<Option<mpsc::Sender<ID>>>> = Arc::new(Mutex::new(None));
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
                        if let Some(ref sender) = *on_shutdown_clone.lock().unwrap() {
                            sender.send(peer_node_id).unwrap();
                        }
                        break;
                    }
                    Err(err) => {
                        println!("{}: error reading connection: {:?}", node_id, err);
                        break;
                    }
                };
                match container.get_kind() {
                    Kind::ServicesMessage => {
                        if let Some(ref f) = *on_services_clone.lock().unwrap() {
                            f(peer_node_id, read_services(&container).unwrap());
                        }
                    }
                    Kind::RequestMessage => {
                        if let Some(ref f) = *on_request_clone.lock().unwrap() {
                            let (request_id, name, data) = read_request(&container).unwrap();
                            write_response(&mut stream, request_id, f(&name, &data)).unwrap();
                        }
                    }
                    Kind::ResponseMessage => {
                        if let Some(ref f) = *on_response_clone.lock().unwrap() {
                            let (request_id, result) = read_response(&container).unwrap();
                            f(request_id, result);
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

    pub fn send_services(&mut self, service_names: &[&str]) -> Result<()> {
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

    pub fn set_on_request(&mut self, f: Box<Fn(&str, &[u8]) -> Result<Vec<u8>> + Send>) {
        *self.on_request.lock().unwrap() = Some(f);
    }

    pub fn set_on_response(&mut self, f: Box<Fn(u32, Result<Vec<u8>>) + Send>) {
        *self.on_response.lock().unwrap() = Some(f);
    }

    pub fn set_on_shutdown(&mut self, sender: mpsc::Sender<ID>) {
        *self.on_shutdown.lock().unwrap() = Some(sender);
    }

    pub fn clear_on_shutdown(&mut self) {
        *self.on_shutdown.lock().unwrap() = None;
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
    let mut introduction = Introduction::new();
    introduction.set_id(node_id.to_vec());
    introduction.set_public_address(format!("{}", public_address));
    try!(introduction.write_to_vec(&mut buffer));
    write_container(w, Kind::IntroductionMessage, buffer)
}

fn write_peers(w: &mut Write, peers: &[(ID, SocketAddr)]) -> Result<()> {
    let mut buffer = Vec::new();
    let mut peers_packet = Peers::new();
    for peer in peers {
        let (peer_node_id, peer_public_address) = *peer;
        let mut peer_packet = Peer::new();
        peer_packet.set_id(peer_node_id.to_vec());
        peer_packet.set_public_address(format!("{}", peer_public_address));
        peers_packet.mut_peers().push(peer_packet);
    }
    try!(peers_packet.write_to_vec(&mut buffer));
    write_container(w, Kind::PeersMessage, buffer)
}

fn write_services(w: &mut Write, service_names: &[&str]) -> Result<()> {
    let mut buffer = Vec::new();
    let mut services_packet = Services::new();
    for service_name in service_names {
        let mut service_packet = Service::new();
        service_packet.set_name((*service_name).to_string());
        services_packet.mut_services().push(service_packet);
    }
    try!(services_packet.write_to_vec(&mut buffer));
    write_container(w, Kind::ServicesMessage, buffer)
}

fn write_request(w: &mut Write, id: u32, name: &str, data: &[u8]) -> Result<()> {
    let mut buffer = Vec::new();
    let mut request_packet = Request::new();
    request_packet.set_id(id);
    request_packet.set_name(name.to_string());
    request_packet.set_data(data.to_vec());
    try!(request_packet.write_to_vec(&mut buffer));
    write_container(w, Kind::RequestMessage, buffer)
}

fn write_response(w: &mut Write, request_id: u32, result: Result<Vec<u8>>) -> Result<()> {
    let mut buffer = Vec::new();
    let mut response_packet = Response::new();
    response_packet.set_request_id(request_id);
    match result {
        Ok(data) => {
            response_packet.set_kind(Response_Kind::OK);
            response_packet.set_data(data);
        }
        Err(Error::ServiceDoesNotExists) => {
            response_packet.set_kind(Response_Kind::ServiceDoesNotExists);
        }
        Err(_) => {
            response_packet.set_kind(Response_Kind::UnknownError);
        }
    }
    try!(response_packet.write_to_vec(&mut buffer));
    write_container(w, Kind::ResponseMessage, buffer)
}

fn write_container(w: &mut Write, kind: Kind, data: Vec<u8>) -> Result<()> {
    let mut container = Container::new();
    container.set_kind(kind);
    container.set_payload(data);

    let container_bytes = try!(container.write_to_bytes());
    let container_size = container_bytes.len() as u64;

    w.write_u64::<byteorder::BigEndian>(container_size).unwrap();
    w.write(&container_bytes).unwrap();
    w.flush().unwrap();

    Ok(())
}

fn read_introduction(container: &Container) -> Result<(ID, SocketAddr)> {
    let introduction_packet = try!(read_packet::<Introduction>(&container));
    Ok((try!(ID::new(introduction_packet.get_id().to_vec())),
        try!(introduction_packet.get_public_address()
                                .parse::<SocketAddr>())))
}

fn read_peers(container: &Container) -> Result<Vec<(ID, SocketAddr)>> {
    Ok(try!(read_packet::<Peers>(&container))
           .get_peers()
           .iter()
           .map(|peer_packet| {
               (ID::new(peer_packet.get_id().to_vec()).unwrap(),
                peer_packet.get_public_address()
                           .parse::<SocketAddr>()
                           .unwrap())
           })
           .collect())
}

fn read_services(container: &Container) -> Result<Vec<String>> {
    Ok(try!(read_packet::<Services>(&container))
           .get_services()
           .to_vec()
           .iter()
           .map(|service_packet| service_packet.get_name().to_string())
           .collect())
}

fn read_request(container: &Container) -> Result<(u32, String, Vec<u8>)> {
    let request_packet = try!(read_packet::<Request>(&container));
    Ok((request_packet.get_id(),
        request_packet.get_name().to_string(),
        request_packet.get_data().to_vec()))
}

fn read_response(container: &Container) -> Result<(u32, Result<Vec<u8>>)> {
    let response_packet = try!(read_packet::<Response>(&container));
    let result = match response_packet.get_kind() {
        Response_Kind::OK => Ok(response_packet.get_data().to_vec()),
        Response_Kind::ServiceDoesNotExists => Err(Error::ServiceDoesNotExists),
        Response_Kind::UnknownError => Err(Error::ServiceDoesNotExists),
    };
    Ok((response_packet.get_request_id(), result))
}

fn read_packet<T: Message + MessageStatic>(container: &Container) -> Result<T> {
    Ok(try!(parse_from_bytes::<T>(container.get_payload())))
}

fn read_container(r: &mut Read) -> Result<Container> {
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

    Ok(try!(parse_from_bytes::<Container>(&buffer)))
}
