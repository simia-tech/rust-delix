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
use std::net::{TcpListener, TcpStream, SocketAddr};
use std::sync::{Arc, Mutex, mpsc};
use std::thread::spawn;

use transport::{Result, Transport};
use transport::direct::{Connection, ServiceMap};

use node::{ID, ServiceHandler};

pub struct Direct {
    local_address: SocketAddr,
    public_address: SocketAddr,
    connections: Arc<Mutex<HashMap<ID, Connection>>>,
    services: Arc<Mutex<ServiceMap>>,
}

impl Direct {
    pub fn new(local_address: SocketAddr, public_address: Option<SocketAddr>) -> Direct {
        Direct {
            local_address: local_address,
            public_address: public_address.unwrap_or(local_address),
            connections: Arc::new(Mutex::new(HashMap::new())),
            services: Arc::new(Mutex::new(ServiceMap::new())),
        }
    }
}

impl Transport for Direct {
    fn bind(&self, node_id: ID) -> Result<()> {
        let tcp_listener = try!(TcpListener::bind(self.local_address));
        println!("bound to address {:?}", self.local_address);

        let public_address = self.public_address;
        let connections_clone = self.connections.clone();
        let services_clone = self.services.clone();
        spawn(move || {
            for stream in tcp_listener.incoming() {
                let stream = stream.unwrap();
                let mut connection = Connection::new(stream, node_id, public_address);

                let peer_node_id = connection.peer_node_id();
                let services_clone_clone = services_clone.clone();
                connection.set_on_services(Box::new(move |services| {
                    println!("{}: received {} services", node_id, services.len());
                    for service in services {
                        services_clone_clone.lock()
                                            .unwrap()
                                            .insert_remote(&service, peer_node_id)
                                            .unwrap();
                    }
                }));

                connection.send_peers(&connection_pairs(&mut *connections_clone.lock().unwrap()))
                          .unwrap();

                connection.send_services(&services_clone.lock().unwrap().local_service_names())
                          .unwrap();

                println!("{}: inbound connection {}", node_id, connection);
                connections_clone.lock().unwrap().insert(connection.peer_node_id(), connection);
            }
        });

        Ok(())
    }

    fn join(&mut self, address: SocketAddr, node_id: ID) -> Result<()> {
        let (tx, rx) = mpsc::channel();
        let mut pending_peers_count = 1;
        tx.send(vec![(ID::new_random(), address)]).unwrap();

        while pending_peers_count > 0 {
            let peers = rx.recv().unwrap();

            for peer in peers {
                let (peer_node_id, peer_public_address) = peer;
                if self.connections.lock().unwrap().contains_key(&peer_node_id) {
                    continue;
                }

                pending_peers_count += 1;

                let stream = try!(TcpStream::connect(peer_public_address));
                let mut connection = Connection::new(stream, node_id, self.public_address);

                let tx = tx.clone();
                connection.set_on_peers(Box::new(move |peers| {
                    println!("{}: received {} peers", node_id, peers.len());
                    tx.send(peers).unwrap();
                }));

                let services_clone = self.services.clone();
                connection.set_on_services(Box::new(move |services| {
                    println!("{}: received {} services", node_id, services.len());
                    for service in services {
                        services_clone.lock()
                                      .unwrap()
                                      .insert_remote(&service, peer_node_id)
                                      .unwrap();
                    }
                }));

                try!(connection.send_services(&self.services
                                                   .lock()
                                                   .unwrap()
                                                   .local_service_names()));

                println!("{}: outbound connection {}", node_id, connection);
                self.connections.lock().unwrap().insert(connection.peer_node_id(), connection);
            }

            pending_peers_count -= 1;
        }

        Ok(())
    }

    fn connection_count(&self) -> usize {
        self.connections.lock().unwrap().len()
    }

    fn register_service(&mut self, name: &str, f: Box<ServiceHandler>) -> Result<()> {
        try!(self.services.lock().unwrap().insert_local(name, f));

        for (_, connection) in self.connections.lock().unwrap().iter_mut() {
            try!(connection.send_services(&self.services.lock().unwrap().local_service_names()));
        }

        Ok(())
    }

    fn service_count(&self) -> usize {
        self.services.lock().unwrap().len()
    }
}

fn connection_pairs(connections: &mut HashMap<ID, Connection>) -> Vec<(ID, SocketAddr)> {
    connections.iter_mut()
               .map(|(peer_node_id, peer_connection)| {
                   (*peer_node_id, peer_connection.peer_public_address())
               })
               .collect::<Vec<(ID, SocketAddr)>>()
}
