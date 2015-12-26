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
use std::net::{TcpListener, TcpStream, SocketAddr};
use std::sync::{Arc, RwLock, mpsc};
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;

use transport::{self, Result, Transport};
use transport::direct::{self, Connection, ConnectionMap, Link, Tracker, ServiceMap};

use node::{ID, ServiceHandler};

pub struct Direct {
    thread: Option<thread::JoinHandle<()>>,
    running: Arc<AtomicBool>,
    local_address: SocketAddr,
    public_address: SocketAddr,
    connections: Arc<RwLock<ConnectionMap>>,
    services: Arc<RwLock<ServiceMap>>,
    tracker: Arc<RwLock<Tracker<direct::ConnectionResult<Vec<u8>>>>>,
}

impl Direct {
    pub fn new(local_address: SocketAddr, public_address: Option<SocketAddr>) -> Direct {
        Direct {
            thread: None,
            running: Arc::new(AtomicBool::new(false)),
            local_address: local_address,
            public_address: public_address.unwrap_or(local_address),
            connections: Arc::new(RwLock::new(ConnectionMap::new())),
            services: Arc::new(RwLock::new(ServiceMap::new())),
            tracker: Arc::new(RwLock::new(Tracker::new())),
        }
    }

    fn unbind(&mut self) -> Result<()> {
        self.running.store(false, Ordering::SeqCst);
        if let Some(thread) = self.thread.take() {
            // connect to local address to enable thread to escape the accept loop.
            try!(TcpStream::connect(self.local_address));
            thread.join().unwrap();
        }
        Ok(())
    }
}

impl Transport for Direct {
    fn bind(&mut self, node_id: ID) -> Result<()> {
        let tcp_listener = try!(TcpListener::bind(self.local_address));

        let public_address = self.public_address;
        let running_clone = self.running.clone();
        let connections_clone = self.connections.clone();
        let services_clone = self.services.clone();
        let tracker_clone = self.tracker.clone();
        self.thread = Some(thread::spawn(move || {
            running_clone.store(true, Ordering::SeqCst);
            for stream in tcp_listener.incoming() {
                if !running_clone.load(Ordering::SeqCst) {
                    break;
                }

                let stream = stream.unwrap();
                let peers = &connections_clone.read()
                                              .unwrap()
                                              .id_public_address_pairs();
                let mut connection = Connection::new_inbound(stream,
                                                             node_id,
                                                             public_address,
                                                             peers)
                                         .unwrap();

                set_up(&mut connection, &services_clone, &tracker_clone);

                connection.send_services(&services_clone.read().unwrap().local_service_names())
                          .unwrap();

                println!("{}: inbound {}", node_id, connection);
                connections_clone.write().unwrap().add(connection).unwrap();
            }
        }));

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
                let mut connections = self.connections.write().unwrap();
                if connections.contains_key(&peer_node_id) {
                    continue;
                }

                pending_peers_count += 1;

                let stream = try!(TcpStream::connect(peer_public_address));
                let (mut connection, peers) = try!(Connection::new_outbound(stream,
                                                                            node_id,
                                                                            self.public_address));
                tx.send(peers).unwrap();

                set_up(&mut connection, &self.services, &self.tracker);

                try!(connection.send_services(&self.services
                                                   .read()
                                                   .unwrap()
                                                   .local_service_names()));

                println!("{}: outbound {}", node_id, connection);
                connections.add(connection).unwrap();
            }

            pending_peers_count -= 1;
        }

        Ok(())
    }

    fn connection_count(&self) -> usize {
        self.connections.read().unwrap().len()
    }

    fn register(&mut self, name: &str, f: Box<ServiceHandler>) -> Result<()> {
        try!(self.services.write().unwrap().insert_local(name, f));

        self.connections
            .write()
            .unwrap()
            .send_services(&self.services.read().unwrap().local_service_names())
            .unwrap();

        Ok(())
    }

    fn deregister(&mut self, name: &str) -> Result<()> {
        try!(self.services.write().unwrap().remove(name));
        Ok(())
    }

    fn service_count(&self) -> usize {
        self.services.read().unwrap().len()
    }

    fn request(&mut self, name: &str, data: &[u8]) -> Result<Vec<u8>> {
        let services = self.services.read().unwrap();
        let link = match services.get_link(name) {
            Some(link) => link,
            None => return Err(transport::Error::Connection(direct::ConnectionError::ServiceDoesNotExists)),
        };

        match *link {
            Link::Local(ref service_handler) => Ok(service_handler(data)),
            Link::Remote(ref peer_node_id) => {
                let (request_id, result_channel) = self.tracker.write().unwrap().begin();
                try!(self.connections
                         .write()
                         .unwrap()
                         .send_request(peer_node_id, request_id, name, data));
                Ok(try!(result_channel.recv().unwrap()))
            }
        }
    }
}

impl fmt::Display for Direct {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f,
               "(Direct transport {} connections {} services)",
               self.connection_count(),
               self.service_count())
    }
}

impl Drop for Direct {
    fn drop(&mut self) {
        self.unbind().unwrap();
    }
}

fn set_up(connection: &mut Connection,
          services: &Arc<RwLock<ServiceMap>>,
          tracker: &Arc<RwLock<Tracker<direct::ConnectionResult<Vec<u8>>>>>) {

    let services_clone = services.clone();
    connection.set_on_services(Box::new(move |peer_node_id, services| {
        for service in services {
            services_clone.write().unwrap().insert_remote(&service, peer_node_id).unwrap();
        }
    }));

    let services_clone = services.clone();
    connection.set_on_request(Box::new(move |name, data| {
        let services = services_clone.read().unwrap();
        let link = match services.get_link(name) {
            Some(link) => link,
            None => return Err(direct::ConnectionError::ServiceDoesNotExists),
        };

        match *link {
            Link::Local(ref service_handler) => Ok(service_handler(data)),
            Link::Remote(_) => unimplemented!(),
        }
    }));

    let tracker_clone = tracker.clone();
    connection.set_on_response(Box::new(move |request_id, result| {
        tracker_clone.write().unwrap().end(request_id, result).unwrap();
    }));
}
