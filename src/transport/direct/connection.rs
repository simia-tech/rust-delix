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

use protobuf::Message as Message_imported_for_functions;
use protobuf::error::ProtobufError;
use protobuf::{parse_from_reader, parse_from_bytes};
use message::{Container, Kind, Introduction, Peers, Peer};
use byteorder::{BigEndian, WriteBytesExt, ReadBytesExt};

use node::ID;

pub struct Connection {
    stream: TcpStream,
    thread: Option<JoinHandle<()>>,

    peer_node_id_receiver: mpsc::Receiver<ID>,
    peer_node_id: Option<ID>,

    peer_public_address_receiver: mpsc::Receiver<SocketAddr>,
    peer_public_address: Option<SocketAddr>,

    on_peers: Arc<Mutex<Option<Box<Fn(Vec<(ID, SocketAddr)>) + Send>>>>,
}

impl Connection {
    pub fn new(s: TcpStream, node_id: ID, public_address: SocketAddr) -> Connection {
        let mut stream = s.try_clone().unwrap();

        let on_peers: Arc<Mutex<Option<Box<Fn(Vec<(ID, SocketAddr)>) + Send>>>> =
            Arc::new(Mutex::new(None));
        let on_peers_mutex = on_peers.clone();

        let (peer_node_id_sender, peer_node_id_receiver) = mpsc::channel();
        let (peer_public_address_sender, peer_public_address_receiver) = mpsc::channel();
        let thread = spawn(move || {
            write_introduction(&mut stream, node_id, public_address).unwrap();
            // println!("send introduction {}", node_id);

            let mut introduction = read_introduction(&mut stream).unwrap();
            let peer_node_id = ID::new(introduction.take_id()).unwrap();
            let peer_public_address = introduction.take_public_address()
                                                  .parse::<SocketAddr>()
                                                  .unwrap();
            // println!("got introduction {} {}", peer_node_id, peer_public_address);

            peer_node_id_sender.send(peer_node_id).unwrap();
            peer_public_address_sender.send(peer_public_address).unwrap();

            loop {
                let mut container = read_container(&mut stream).unwrap();
                match container.get_kind() {
                    Kind::PeersMessage => {
                        let mut peers_packet = read_peers(&mut container).unwrap();
                        let mut peers = Vec::new();
                        for peer_packet in peers_packet.mut_peers().iter_mut() {
                            let peer_node_id = ID::new(peer_packet.take_id()).unwrap();
                            let peer_public_address = peer_packet.take_public_address()
                                                                 .parse::<SocketAddr>()
                                                                 .unwrap();

                            peers.push((peer_node_id, peer_public_address));
                        }

                        if let Some(ref f) = *on_peers_mutex.lock().unwrap() {
                            f(peers);
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
            peer_node_id_receiver: peer_node_id_receiver,
            peer_node_id: None,
            peer_public_address_receiver: peer_public_address_receiver,
            peer_public_address: None,
            on_peers: on_peers,
        }
    }

    pub fn peer_node_id(&mut self) -> ID {
        if let Some(node_id) = self.peer_node_id {
            return node_id;
        }

        let peer_node_id = self.peer_node_id_receiver.recv().unwrap();
        self.peer_node_id = Some(peer_node_id);
        peer_node_id
    }

    pub fn peer_public_address(&mut self) -> SocketAddr {
        if let Some(public_address) = self.peer_public_address {
            return public_address;
        }

        let peer_public_address = self.peer_public_address_receiver.recv().unwrap();
        self.peer_public_address = Some(peer_public_address);
        peer_public_address
    }

    pub fn peer_addr(&self) -> SocketAddr {
        self.stream.peer_addr().unwrap()
    }

    pub fn local_addr(&self) -> SocketAddr {
        self.stream.local_addr().unwrap()
    }

    pub fn send_peers(&mut self, peers: &[(ID, SocketAddr)]) {
        // WORKAROUND: process peer node id first before sending peers. This ensures, that
        // that the introduction sequence has been finished.
        self.peer_node_id();

        write_peers(&mut self.stream, peers).unwrap();
    }

    pub fn set_on_peers(&mut self, f: Box<Fn(Vec<(ID, SocketAddr)>) + Send>) {
        *self.on_peers.lock().unwrap() = Some(f);
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

fn write_introduction(w: &mut Write,
                      node_id: ID,
                      public_address: SocketAddr)
                      -> Result<(), ProtobufError> {
    let mut buffer = Vec::new();
    let mut introduction = Introduction::new();
    introduction.set_id(node_id.to_vec());
    introduction.set_public_address(format!("{}", public_address));
    try!(introduction.write_to_vec(&mut buffer));
    write_container(w, Kind::IntroductionMessage, buffer)
}

fn write_peers(w: &mut Write, peers: &[(ID, SocketAddr)]) -> Result<(), ProtobufError> {
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

fn write_container(w: &mut Write, kind: Kind, data: Vec<u8>) -> Result<(), ProtobufError> {
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

fn read_introduction(r: &mut Read) -> Result<Introduction, ProtobufError> {
    let mut container = try!(read_container(r));
    let mut payload_cursor = Cursor::new(container.take_payload());
    parse_from_reader::<Introduction>(&mut payload_cursor)
}

fn read_peers(container: &mut Container) -> Result<Peers, ProtobufError> {
    parse_from_reader::<Peers>(&mut Cursor::new(container.take_payload()))
}

fn read_container(r: &mut Read) -> Result<Container, ProtobufError> {
    let container_size = r.read_u64::<BigEndian>().unwrap() as usize;

    let mut buffer = Vec::with_capacity(container_size);
    unsafe {
        buffer.set_len(container_size);
    }
    r.read(&mut buffer).unwrap();

    parse_from_bytes::<Container>(&buffer)
}
