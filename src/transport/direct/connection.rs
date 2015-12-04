/*
Copyright 2015 The Delix Project Authors. See the AUTHORS file at the top level directory.

Licensed under the Apache License, Version 2.0 (the "License");
you may not use this file except in compliance with the License.
You may obtain a copy of the License at

http://www.apache.org/licenses/LICENSE-2.0

Unless required by applicable law or agreed to in writing, software
distributed under the License is distributed on an "AS IS" BASIS,
WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
See the License for the specific language governing permissions and
limitations under the License.
*/

use std::fmt;
use std::io::{Cursor, Read, Write};
use std::net::{SocketAddr, TcpStream};
use std::sync::mpsc;
use std::thread::{JoinHandle, spawn};

use protobuf::Message as Message_imported_for_functions;
use protobuf::error::ProtobufError;
use protobuf::parse_from_reader;
use message::{Container, Kind, Introduction};
use byteorder::{BigEndian, WriteBytesExt, ReadBytesExt};

use node::ID;

pub struct Connection {
    stream: TcpStream,
    thread: Option<JoinHandle<()>>,
    peer_node_id_receiver: mpsc::Receiver<ID>,
    peer_node_id: Option<ID>,
}

impl Connection {

    pub fn new(s: TcpStream, node_id: ID) -> Connection {
        let mut stream = s.try_clone().unwrap();
        let (sender, receiver) = mpsc::channel();
        let thread = spawn(move || {
            write_introduction(&mut stream, node_id).unwrap();
            println!("send introduction {}", node_id);

            let mut introduction = read_introduction(&mut stream).unwrap();
            let peer_node_id = ID::new(introduction.take_id()).unwrap();
            println!("got introduction {}", peer_node_id);

            sender.send(peer_node_id).unwrap();
        });

        Connection {
            stream: s,
            thread: Some(thread),
            peer_node_id_receiver: receiver,
            peer_node_id: None,
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

    pub fn peer_addr(&self) -> SocketAddr {
        self.stream.peer_addr().unwrap()
    }

    pub fn local_addr(&self) -> SocketAddr {
        self.stream.local_addr().unwrap()
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

fn write_introduction(w: &mut Write, node_id: ID) -> Result<(), ProtobufError> {
    let mut buffer = Vec::new();
    let mut introduction = Introduction::new();
    introduction.set_id(node_id.to_vec());
    try!(introduction.write_to_vec(&mut buffer));
    write_container(w, Kind::IntroductionMessage, buffer)
}

fn write_container(w: &mut Write, kind: Kind, data: Vec<u8>) -> Result<(), ProtobufError> {
    let mut container = Container::new();
    container.set_kind(kind);
    container.set_payload(data);

    let container_bytes = try!(container.write_to_bytes());
    let container_size = container_bytes.len() as u64;

    w.write_u64::<BigEndian>(container_size).unwrap();
    w.write(&container_bytes).unwrap();

    Ok(())
}

fn read_introduction(r: &mut Read) -> Result<Introduction, ProtobufError> {
    let mut container = try!(read_container(r));
    let mut payload_cursor = Cursor::new(container.take_payload());
    parse_from_reader::<Introduction>(&mut payload_cursor)
}

fn read_container(r: &mut Read) -> Result<Container, ProtobufError> {
    let container_size = r.read_u64::<BigEndian>().unwrap() as usize;

    let mut buffer = Vec::with_capacity(container_size);
    unsafe { buffer.set_len(container_size); }
    r.read(&mut buffer).unwrap();

    parse_from_reader::<Container>(&mut Cursor::new(buffer))
}
