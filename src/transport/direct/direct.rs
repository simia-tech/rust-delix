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

use std::collections::HashMap;
use std::net::{TcpListener, TcpStream, SocketAddr};
use std::sync::{Arc, Mutex};
use std::thread::spawn;

use transport::{Result, Transport};
use transport::direct::Connection;

use node::ID;

pub struct Direct {
    connections: Arc<Mutex<HashMap<ID, Connection>>>,
}

impl Direct {

    pub fn new() -> Direct {
        Direct {
            connections: Arc::new(Mutex::new(HashMap::new())),
        }
    }

}

impl Transport for Direct {

    fn bind(&self, address: SocketAddr, node_id: ID) -> Result<()> {
        let tcp_listener = try!(TcpListener::bind(address));
        println!("bound to address {:?}", address);

        let connections = self.connections.clone();
        spawn(move || {
            for stream in tcp_listener.incoming() {
                let stream = stream.unwrap();
                let mut connection = Connection::new(stream, node_id);
                println!("inbound connection {}", connection);
                connections.lock().unwrap().insert(connection.peer_node_id(), connection);
            }
        });

        Ok(())
    }

    fn join(&mut self, address: SocketAddr, node_id: ID) -> Result<()> {
        let stream = try!(TcpStream::connect(address));
        let mut connection = Connection::new(stream, node_id);
        println!("outbound connection {}", connection);
        self.connections.lock().unwrap().insert(connection.peer_node_id(), connection);
        Ok(())
    }

    fn connection_count(&self) -> usize {
        self.connections.lock().unwrap().len()
    }

}
