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
use std::io::{Cursor, Read, Write};
use std::net::{SocketAddr, TcpStream};
use std::sync::{Arc, Mutex, mpsc};
use std::thread::{JoinHandle, spawn};

use protobuf::{Message, MessageStatic};
use protobuf::{parse_from_reader, parse_from_bytes};
use message::{Container, Kind, Introduction, Peers, Peer, Request, Response, Response_Kind,
              Services, Service};
use byteorder::{BigEndian, WriteBytesExt, ReadBytesExt};

use node::ID;
use transport::{Error, Result};

pub struct Connection {
    stream: TcpStream,
    thread: Option<JoinHandle<()>>,

    peer_node_id: ID,
    peer_public_address: SocketAddr,

    on_peers: Arc<Mutex<Option<Box<Fn(Vec<(ID, SocketAddr)>) + Send>>>>,
    on_services: Arc<Mutex<Option<Box<Fn(ID, Vec<String>) + Send>>>>,
    on_request: Arc<Mutex<Option<Box<Fn(String, &[u8]) -> Result<Vec<u8>> + Send>>>>,
    on_response: Arc<Mutex<Option<Box<Fn(Result<Vec<u8>>) + Send>>>>,
}

impl Connection {
    pub fn new(s: TcpStream, node_id: ID, public_address: SocketAddr) -> Connection {
        let mut stream = s.try_clone().unwrap();

        let on_peers: Arc<Mutex<Option<Box<Fn(Vec<(ID, SocketAddr)>) + Send>>>> =
            Arc::new(Mutex::new(None));
        let on_peers_clone = on_peers.clone();

        let on_services: Arc<Mutex<Option<Box<Fn(ID, Vec<String>) + Send>>>> =
            Arc::new(Mutex::new(None));
        let on_services_clone = on_services.clone();

        let on_request: Arc<Mutex<Option<Box<Fn(String, &[u8]) -> Result<Vec<u8>> + Send>>>> =
            Arc::new(Mutex::new(None));
        let on_request_clone = on_request.clone();

        let on_response: Arc<Mutex<Option<Box<Fn(Result<Vec<u8>>) + Send>>>> =
            Arc::new(Mutex::new(None));
        let on_response_clone = on_response.clone();

        let (peer_node_id_sender, peer_node_id_receiver) = mpsc::channel();
        let (peer_public_address_sender, peer_public_address_receiver) = mpsc::channel();
        let thread = spawn(move || {
            write_introduction(&mut stream, node_id, public_address).unwrap();
            // println!("send introduction {}", node_id);

            let container = read_container(&mut stream).unwrap();
            let introduction = read_packet::<Introduction>(&container).unwrap();
            let peer_node_id = ID::new(introduction.get_id().to_vec()).unwrap();
            let peer_public_address = introduction.get_public_address()
                                                  .parse::<SocketAddr>()
                                                  .unwrap();
            // println!("got introduction {} {}", peer_node_id, peer_public_address);

            peer_node_id_sender.send(peer_node_id).unwrap();
            peer_public_address_sender.send(peer_public_address).unwrap();

            loop {
                let container = read_container(&mut stream).unwrap();
                match container.get_kind() {
                    Kind::PeersMessage => {
                        if let Some(ref f) = *on_peers_clone.lock().unwrap() {
                            let peers_packet = read_packet::<Peers>(&container).unwrap();
                            f(peers_packet.get_peers()
                                          .iter()
                                          .map(|peer_packet| {
                                              (ID::new(peer_packet.get_id().to_vec()).unwrap(),
                                               peer_packet.get_public_address()
                                                          .parse::<SocketAddr>()
                                                          .unwrap())
                                          })
                                          .collect());
                        }
                    }
                    Kind::ServicesMessage => {
                        if let Some(ref f) = *on_services_clone.lock().unwrap() {
                            let services_packet = read_packet::<Services>(&container).unwrap();
                            f(peer_node_id,
                              services_packet.get_services()
                                             .to_vec()
                                             .iter()
                                             .map(|service_packet| {
                                                 service_packet.get_name().to_string()
                                             })
                                             .collect());
                        }
                    }
                    Kind::RequestMessage => {
                        if let Some(ref f) = *on_request_clone.lock().unwrap() {
                            let request_packet = read_packet::<Request>(&container).unwrap();
                            write_response(&mut stream,
                                           f(request_packet.get_name().to_string(),
                                             request_packet.get_data()))
                                .unwrap();
                        }
                    }
                    Kind::ResponseMessage => {
                        if let Some(ref f) = *on_response_clone.lock().unwrap() {
                            let response_packet = read_packet::<Response>(&container).unwrap();
                            let result = match response_packet.get_kind() {
                                Response_Kind::OK => Ok(response_packet.get_data().to_vec()),
                                Response_Kind::ServiceDoesNotExists => {
                                    Err(Error::ServiceDoesNotExists)
                                }
                                Response_Kind::UnknownError => Err(Error::ServiceDoesNotExists),
                            };
                            f(result);
                        }
                    }
                    _ => {
                        println!("{}: got unexpected container {:?}", node_id, container);
                    }
                }
            }
        });

        Connection {
            stream: s,
            thread: Some(thread),
            peer_node_id: peer_node_id_receiver.recv().unwrap(),
            peer_public_address: peer_public_address_receiver.recv().unwrap(),
            on_peers: on_peers,
            on_services: on_services,
            on_request: on_request,
            on_response: on_response,
        }
    }

    pub fn peer_node_id(&self) -> ID {
        self.peer_node_id
    }

    pub fn peer_public_address(&self) -> SocketAddr {
        self.peer_public_address
    }

    pub fn peer_addr(&self) -> SocketAddr {
        self.stream.peer_addr().unwrap()
    }

    pub fn local_addr(&self) -> SocketAddr {
        self.stream.local_addr().unwrap()
    }

    pub fn send_peers(&mut self, peers: &[(ID, SocketAddr)]) -> Result<()> {
        try!(write_peers(&mut self.stream, peers));
        Ok(())
    }

    pub fn set_on_peers(&mut self, f: Box<Fn(Vec<(ID, SocketAddr)>) + Send>) {
        *self.on_peers.lock().unwrap() = Some(f);
    }

    pub fn send_services(&mut self, service_names: &[&str]) -> Result<()> {
        try!(write_services(&mut self.stream, service_names));
        Ok(())
    }

    pub fn set_on_services(&mut self, f: Box<Fn(ID, Vec<String>) + Send>) {
        *self.on_services.lock().unwrap() = Some(f);
    }

    pub fn send_request(&mut self, name: &str, data: &[u8]) -> Result<()> {
        try!(write_request(&mut self.stream, name, data));
        Ok(())
    }

    pub fn set_on_request(&mut self, f: Box<Fn(String, &[u8]) -> Result<Vec<u8>> + Send>) {
        *self.on_request.lock().unwrap() = Some(f);
    }

    pub fn set_on_response(&mut self, f: Box<Fn(Result<Vec<u8>>) + Send>) {
        *self.on_response.lock().unwrap() = Some(f);
    }
}

impl fmt::Display for Connection {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "({} -> {})", self.local_addr(), self.peer_addr())
    }
}

impl Drop for Connection {
    fn drop(&mut self) {
        self.thread.take().unwrap().join().unwrap();
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

fn write_request(w: &mut Write, name: &str, data: &[u8]) -> Result<()> {
    let mut buffer = Vec::new();
    let mut request_packet = Request::new();
    request_packet.set_name(name.to_string());
    request_packet.set_data(data.to_vec());
    try!(request_packet.write_to_vec(&mut buffer));
    write_container(w, Kind::RequestMessage, buffer)
}

fn write_response(w: &mut Write, result: Result<Vec<u8>>) -> Result<()> {
    let mut buffer = Vec::new();
    let mut response_packet = Response::new();
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

    w.write_u64::<BigEndian>(container_size).unwrap();
    w.write(&container_bytes).unwrap();
    w.flush().unwrap();

    Ok(())
}

fn read_packet<T: Message + MessageStatic>(container: &Container) -> Result<T> {
    Ok(try!(parse_from_reader::<T>(&mut Cursor::new(container.get_payload()))))
}

fn read_container(r: &mut Read) -> Result<Container> {
    let container_size = r.read_u64::<BigEndian>().unwrap() as usize;

    let mut buffer = Vec::with_capacity(container_size);
    unsafe {
        buffer.set_len(container_size);
    }
    r.read(&mut buffer).unwrap();

    Ok(try!(parse_from_bytes::<Container>(&buffer)))
}
