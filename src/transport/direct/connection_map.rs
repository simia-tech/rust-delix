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
use std::collections::hash_map::IterMut;
use std::net::SocketAddr;
use std::result;

use node::ID;
use transport::direct::Connection;

pub struct ConnectionMap {
    map: HashMap<ID, Connection>,
}

pub type Result<T> = result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    ConnectionAlreadyExists,
}

impl ConnectionMap {
    pub fn new() -> ConnectionMap {
        ConnectionMap { map: HashMap::new() }
    }

    pub fn add(&mut self, connection: Connection) -> Result<()> {
        if self.map.contains_key(&connection.peer_node_id()) {
            return Err(Error::ConnectionAlreadyExists);
        }
        self.map.insert(connection.peer_node_id(), connection);
        Ok(())
    }

    pub fn contains_key(&self, peer_node_id: &ID) -> bool {
        self.map.contains_key(peer_node_id)
    }

    pub fn get_mut(&mut self, peer_node_id: &ID) -> Option<&mut Connection> {
        self.map.get_mut(peer_node_id)
    }

    pub fn id_public_address_pairs(&mut self) -> Vec<(ID, SocketAddr)> {
        self.map
            .iter_mut()
            .map(|(peer_node_id, peer_connection)| {
                (*peer_node_id, peer_connection.peer_public_address())
            })
            .collect()
    }

    pub fn len(&self) -> usize {
        self.map.len()
    }

    pub fn iter_mut(&mut self) -> IterMut<ID, Connection> {
        self.map.iter_mut()
    }
}
