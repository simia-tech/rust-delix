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
use std::io::Write;
use std::net::{SocketAddr, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread::{JoinHandle, spawn};

use protobuf::Message as Message_imported_for_functions;
use message::{Container, Kind, NodeAdd};

pub struct Connection {
    stream: Arc<Mutex<TcpStream>>,
    thread: Option<JoinHandle<()>>,
}

impl Connection {

    pub fn new(s: TcpStream) -> Connection {
        let stream_mutex = Arc::new(Mutex::new(s));

        let stream = stream_mutex.clone();
        let thread = spawn(move || {
            let mut buffer = Vec::new();
            let mut node_add = NodeAdd::new();
            node_add.set_address(vec![0, 1, 2, 3]);
            node_add.write_to_vec(&mut buffer).unwrap();

            write_container(&mut *stream.lock().unwrap(), Kind::NodeAddMessage, buffer);
        });

        Connection {
            stream: stream_mutex,
            thread: Some(thread),
        }
    }

    pub fn peer_addr(&self) -> SocketAddr {
        (*self.stream.lock().unwrap()).peer_addr().unwrap()
    }

}

impl fmt::Display for Connection {

    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "(-> {})", self.peer_addr())
    }

}

impl Drop for Connection {

    fn drop(&mut self) {
        self.thread.take().unwrap().join().unwrap();
    }

}

fn write_container(w: &mut Write, kind: Kind, data: Vec<u8>) {
    let mut container = Container::new();
    container.set_kind(kind);
    container.set_payload(data);
    container.write_to_writer(w).unwrap();
}
