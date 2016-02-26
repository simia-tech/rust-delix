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
use std::io;
use std::net::SocketAddr;
use std::result;
use std::sync::{Arc, Mutex, RwLock, mpsc};
use std::thread;

use metric::{self, Metric};
use node::{ID, request, service};
use transport::direct::Connection;

pub struct ConnectionMap {
    map: Arc<RwLock<HashMap<ID, Connection>>>,
    tx: Mutex<mpsc::Sender<ID>>,
    connections_gauge: Arc<metric::item::Gauge>,
}

pub type Result<T> = result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    AlreadyExists,
    DoesNotExists,
}

impl ConnectionMap {
    pub fn new(metric: Arc<Metric>) -> Self {
        let map = Arc::new(RwLock::new(HashMap::new()));
        let map_clone = map.clone();

        let connections_gauge = Arc::new(metric.gauge("connections"));
        let connections_gauge_clone = connections_gauge.clone();

        let (tx, rx) = mpsc::channel::<ID>();
        thread::spawn(move || {
            for peer_node_id in rx {
                map_clone.write().unwrap().remove(&peer_node_id);
                connections_gauge_clone.change(-1);
            }
        });
        ConnectionMap {
            map: map,
            tx: Mutex::new(tx),
            connections_gauge: connections_gauge,
        }
    }

    pub fn add(&self, connection: Connection) -> Result<()> {
        let mut map = self.map.write().unwrap();
        if map.contains_key(&connection.peer_node_id()) {
            return Err(Error::AlreadyExists);
        }

        let tx = self.tx.lock().unwrap().clone();
        connection.set_error_handler(Box::new(move |peer_node_id, error| {
            if error.kind() != io::ErrorKind::ConnectionAborted {
                error!("got connection error: {:?}", error);
            }
            tx.send(peer_node_id).unwrap();
        }));

        map.insert(connection.peer_node_id(), connection);
        self.connections_gauge.change(1);
        Ok(())
    }

    pub fn contains_key(&self, peer_node_id: &ID) -> bool {
        self.map.read().unwrap().contains_key(peer_node_id)
    }

    pub fn select<F, T>(&self, peer_node_id: &ID, f: F) -> Result<T>
        where F: FnOnce(&Connection) -> T
    {
        let map = self.map.read().unwrap();
        match map.get(peer_node_id) {
            Some(ref connection) => Ok(f(connection)),
            None => Err(Error::DoesNotExists),
        }
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

    pub fn send_add_services(&self, services: &[String]) -> io::Result<()> {
        let mut map = self.map.write().unwrap();
        for (_, connection) in map.iter_mut() {
            try!(connection.send_add_services(services));
        }
        Ok(())
    }

    pub fn send_remove_services(&self, services: &[String]) -> io::Result<()> {
        let mut map = self.map.write().unwrap();
        for (_, connection) in map.iter_mut() {
            try!(connection.send_remove_services(services));
        }
        Ok(())
    }

    pub fn send_request(&self,
                        peer_node_id: &ID,
                        id: u32,
                        name: &str,
                        reader: &mut request::Reader)
                        -> io::Result<()> {
        let map = self.map.read().unwrap();
        let connection = map.get(peer_node_id).unwrap();
        Ok(try!(connection.send_request(id, name, reader)))
    }

    pub fn send_response(&self,
                         peer_node_id: &ID,
                         request_id: u32,
                         service_result: service::Result)
                         -> io::Result<()> {
        let map = self.map.read().unwrap();
        let connection = match map.get(peer_node_id) {
            Some(connection) => connection,
            None => {
                return Err(io::Error::new(io::ErrorKind::ConnectionAborted, "connection aborted"))
            }
        };
        Ok(try!(connection.send_response(request_id, service_result)))
    }

    pub fn shutdown(&self) {
        let map = self.map.read().unwrap();
        for (_, connection) in map.iter() {
            connection.clear_error_handler();
            connection.shutdown();
        }
    }
}
