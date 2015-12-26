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

use std::collections::HashMap;
use std::fmt;
use std::net::SocketAddr;
use std::result;
use std::sync::{Arc, RwLock, mpsc};
use std::thread;

use node::ID;
use transport::direct::{self, Connection};

pub struct ConnectionMap {
    map: Arc<RwLock<HashMap<ID, Connection>>>,
    sender: mpsc::Sender<ID>,
}

pub type Result<T> = result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    ConnectionAlreadyExists,
    Connection(direct::ConnectionError),
}

impl ConnectionMap {
    pub fn new() -> ConnectionMap {
        let map = Arc::new(RwLock::new(HashMap::new()));
        let map_clone = map.clone();

        let (sender, receiver) = mpsc::channel::<ID>();
        thread::spawn(move || {
            for peer_node_id in receiver {
                map_clone.write().unwrap().remove(&peer_node_id);
            }
        });
        ConnectionMap {
            map: map,
            sender: sender,
        }
    }

    pub fn add(&self, mut connection: Connection) -> Result<()> {
        let mut map = self.map.write().unwrap();
        if map.contains_key(&connection.peer_node_id()) {
            return Err(Error::ConnectionAlreadyExists);
        }

        let sender = self.sender.clone();
        connection.set_on_shutdown(Box::new(move |peer_node_id| {
            sender.send(peer_node_id).unwrap()
        }));

        map.insert(connection.peer_node_id(), connection);
        Ok(())
    }

    pub fn contains_key(&self, peer_node_id: &ID) -> bool {
        self.map.read().unwrap().contains_key(peer_node_id)
    }

    pub fn id_public_address_pairs(&self) -> Vec<(ID, SocketAddr)> {
        self.map
            .read()
            .unwrap()
            .iter()
            .map(|(peer_node_id, peer_connection)| {
                (*peer_node_id, peer_connection.peer_public_address())
            })
            .collect()
    }

    pub fn len(&self) -> usize {
        self.map.read().unwrap().len()
    }

    pub fn send_services(&self, services: &[String]) -> Result<()> {
        let mut map = self.map.write().unwrap();
        for (_, connection) in map.iter_mut() {
            try!(connection.send_services(services));
        }
        Ok(())
    }

    pub fn send_request(&self, peer_node_id: &ID, id: u32, name: &str, data: &[u8]) -> Result<()> {
        let mut map = self.map.write().unwrap();
        let mut connection = map.get_mut(peer_node_id).unwrap();
        Ok(try!(connection.send_request(id, name, data)))
    }

    pub fn clear_on_shutdown(&self) {
        let mut map = self.map.write().unwrap();
        for (_, connection) in map.iter_mut() {
            connection.clear_on_shutdown();
        }
    }
}

unsafe impl Send for ConnectionMap {}

unsafe impl Sync for ConnectionMap {}

impl fmt::Display for ConnectionMap {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "(Direct connection map {} entries)", self.len())
    }
}

impl Drop for ConnectionMap {
    fn drop(&mut self) {
        self.clear_on_shutdown();
    }
}

impl From<direct::ConnectionError> for Error {
    fn from(error: direct::ConnectionError) -> Self {
        Error::Connection(error)
    }
}
