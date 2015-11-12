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

pub struct Direct {
    connections: Arc<Mutex<HashMap<SocketAddr, Connection>>>,
}

impl Direct {

    pub fn new() -> Direct {
        Direct { connections: Arc::new(Mutex::new(HashMap::new())) }
    }

}

impl Transport for Direct {

    fn bind(&self, address: SocketAddr) -> Result<()> {
        let tcp_listener = try!(TcpListener::bind(address));
        println!("bound to address {:?}", address);

        let connections = self.connections.clone();
        spawn(move || {
            for stream in tcp_listener.incoming() {
                let stream = stream.unwrap();
                let connection = Connection::new(stream);
                println!("got connection {}", connection);

                connections.lock().unwrap().insert(connection.peer_addr(), connection);
            }
        });

        Ok(())
    }

    fn join(&mut self, address: SocketAddr) -> Result<()> {
        println!("join address {:?}", address);
        let stream = try!(TcpStream::connect(address));
        let connection = Connection::new(stream);
        self.connections.lock().unwrap().insert(connection.peer_addr(), connection);
        Ok(())
    }

    fn connection_count(&self) -> usize {
        self.connections.lock().unwrap().len()
    }

}
