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

use std::io;
use std::net::{TcpListener, TcpStream, SocketAddr};
use std::sync::{Arc, RwLock, mpsc};
use std::thread;

use time::Duration;

use transport::{Result, Transport};
use transport::cipher::Cipher;
use metric::Metric;
use node::{ID, Service, request, response};

use super::{Balancer, Connection, ConnectionMap, Link, Tracker, ServiceMap};
use super::tracker::Statistic;

pub struct Direct {
    join_handle: RwLock<Option<thread::JoinHandle<()>>>,
    running: Arc<RwLock<bool>>,
    local_address: SocketAddr,
    public_address: SocketAddr,
    cipher: Arc<Box<Cipher>>,
    connections: Arc<ConnectionMap>,
    services: Arc<ServiceMap>,
    tracker: Arc<Tracker<Box<response::Writer>, request::Result>>,
}

impl Direct {
    pub fn new(cipher: Box<Cipher>,
               balancer: Box<Balancer>,
               metric: Arc<Metric>,
               local_address: SocketAddr,
               public_address: Option<SocketAddr>,
               request_timeout: Option<Duration>)
               -> Self {

        let statistic = Arc::new(Statistic::new());
        balancer.assign_statistic(statistic.clone());

        Direct {
            join_handle: RwLock::new(None),
            running: Arc::new(RwLock::new(false)),
            local_address: local_address,
            public_address: public_address.unwrap_or(local_address),
            cipher: Arc::new(cipher),
            connections: Arc::new(ConnectionMap::new(metric.clone())),
            services: Arc::new(ServiceMap::new(balancer, metric.clone())),
            tracker: Arc::new(Tracker::new(statistic.clone(), request_timeout)),
        }
    }

    fn unbind(&mut self) -> Result<()> {
        *self.running.write().unwrap() = false;
        if let Some(join_handle) = self.join_handle.write().unwrap().take() {
            // connect to local address to enable the thread to escape the accept loop.
            try!(TcpStream::connect(self.local_address));
            join_handle.join().unwrap();
        }
        Ok(())
    }
}

impl Transport for Direct {
    fn bind(&self, node_id: ID) -> Result<()> {
        let tcp_listener = try!(TcpListener::bind(self.local_address));

        *self.running.write().unwrap() = true;

        let public_address = self.public_address;
        let running_clone = self.running.clone();
        let cipher_clone = self.cipher.clone();
        let connections_clone = self.connections.clone();
        let services_clone = self.services.clone();
        let tracker_clone = self.tracker.clone();
        *self.join_handle.write().unwrap() = Some(thread::spawn(move || {
            for stream in tcp_listener.incoming() {
                if !*running_clone.read().unwrap() {
                    break;
                }

                let stream = stream.unwrap();
                let peers = &connections_clone.id_public_address_pairs();
                let mut connection = Connection::new_inbound(stream,
                                                             cipher_clone.clone(),
                                                             node_id,
                                                             public_address,
                                                             peers)
                                         .unwrap();

                set_up(&mut connection, &services_clone, &tracker_clone);

                connection.send_add_services(&services_clone.local_service_names())
                          .unwrap();

                info!("{}: inbound {}", node_id, connection);
                connections_clone.add(connection).unwrap();
            }
        }));

        Ok(())
    }

    fn join(&self, address: SocketAddr, node_id: ID) -> Result<()> {
        let (tx, rx) = mpsc::channel();
        let mut pending_peers_count = 1;
        tx.send(vec![(ID::new_random(), address)]).unwrap();

        while pending_peers_count > 0 {
            let peers = rx.recv().unwrap();

            for peer in peers {
                let (peer_node_id, peer_public_address) = peer;
                if self.connections.contains_key(&peer_node_id) {
                    continue;
                }

                pending_peers_count += 1;

                let stream = try!(TcpStream::connect(peer_public_address));
                let (mut connection, peers) = try!(Connection::new_outbound(stream,
                                                                            self.cipher.clone(),
                                                                            node_id,
                                                                            self.public_address));
                tx.send(peers).unwrap();

                set_up(&mut connection, &self.services, &self.tracker);

                try!(connection.send_add_services(&self.services.local_service_names()));

                info!("{}: outbound {}", node_id, connection);
                self.connections.add(connection).unwrap();
            }

            pending_peers_count -= 1;
        }

        Ok(())
    }

    fn register(&self, name: &str, f: Box<Service>) -> Result<()> {
        try!(self.services.insert_local(name, f));

        self.connections.send_add_services(&vec![name.to_string()]).unwrap();

        Ok(())
    }

    fn deregister(&self, name: &str) -> Result<()> {
        self.connections.send_remove_services(&vec![name.to_string()]).unwrap();

        try!(self.services.remove_local(name));

        Ok(())
    }

    fn request(&self,
               name: &str,
               reader: Box<request::Reader>,
               response_writer: Box<response::Writer>)
               -> request::Result {

        self.services.select(name,
                             reader,
                             response_writer,
                             |reader, response_writer, handler| {
                                 let (request_id, response_rx) = self.tracker
                                                                     .begin(name,
                                                                            &Link::Local,
                                                                            response_writer);
                                 let handler_clone = handler.clone();
                                 let tracker_clone = self.tracker.clone();
                                 thread::spawn(move || {
                                     let mut service_result = (*handler_clone.lock()
                                                                             .unwrap())(reader);

                                     let timed_out =
                                         !tracker_clone.end(request_id, |mut response_writer| {
                                             match service_result {
                                                 Ok(ref mut reader) => {
                                                     if let Err(error) =
                                                            io::copy(reader, &mut response_writer) {
                                                         Err(request::Error::from(error))
                                                     } else {
                                                         Ok(response_writer)
                                                     }
                                                 }
                                                 Err(ref error) => {
                                                     Err(request::Error::Service(error.clone()))
                                                 }
                                             }
                                         });

                                     if timed_out {
                                         debug!("got response for request ({}) that already \
                                                 timed out",
                                                request_id);
                                     }
                                 });
                                 try!(response_rx.recv().unwrap())
                             },
                             |mut reader, response_writer, peer_node_id| {
                                 let (request_id, response_rx) =
                                     self.tracker
                                         .begin(name, &Link::Remote(peer_node_id), response_writer);
                                 try!(self.connections
                                          .send_request(&peer_node_id,
                                                        request_id,
                                                        name,
                                                        &mut reader));
                                 try!(response_rx.recv().unwrap())
                             })
    }
}

impl Drop for Direct {
    fn drop(&mut self) {
        self.unbind().unwrap();
    }
}

fn set_up(connection: &mut Connection,
          services: &Arc<ServiceMap>,
          tracker: &Arc<Tracker<Box<response::Writer>, request::Result>>) {
    let services_clone = services.clone();
    connection.set_on_add_services(Box::new(move |peer_node_id, services| {
        services_clone.insert_remotes(&services, peer_node_id);
    }));

    let services_clone = services.clone();
    connection.set_on_remove_services(Box::new(move |peer_node_id, services| {
        services_clone.remove_remotes(&services, &peer_node_id);
    }));

    let services_clone = services.clone();
    connection.set_on_request(Box::new(move |name, reader| {
        services_clone.select_local(name, |handler| handler(reader))
    }));

    let tracker_clone = tracker.clone();
    connection.set_on_response(Box::new(move |request_id, mut service_result| {
        let timed_out = !tracker_clone.end(request_id, |mut response_writer| {
            match service_result {
                Ok(ref mut reader) => {
                    try!(io::copy(reader, &mut response_writer));
                    Ok(response_writer)
                }
                Err(ref error) => Err(request::Error::Service(error.clone())),
            }
        });

        if timed_out {
            debug!("got response for request ({}) that already timed out",
                   request_id);
        }

        Ok(())
    }));

    let services_clone = services.clone();
    connection.set_on_drop(Box::new(move |peer_node_id| {
        services_clone.remove_all_remotes(&peer_node_id);
    }));
}
