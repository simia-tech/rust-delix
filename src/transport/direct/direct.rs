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
use std::io;
use std::net::{TcpListener, TcpStream, SocketAddr};
use std::sync::{Arc, Mutex, RwLock, mpsc};
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;

use time::Duration;

use transport::{Result, Transport};
use transport::cipher::Cipher;
use transport::direct::{Balancer, Connection, ConnectionMap, Link, Tracker, ServiceMap};
use transport::direct::tracker::{self, Statistic};

use node::{ID, request};

pub struct Direct {
    join_handle: RwLock<Option<thread::JoinHandle<()>>>,
    running: Arc<AtomicBool>,
    local_address: SocketAddr,
    public_address: SocketAddr,
    cipher: Arc<Box<Cipher>>,
    connections: Arc<ConnectionMap>,
    services: Arc<ServiceMap>,
    tracker: Arc<Tracker>,
}

impl Direct {
    pub fn new(cipher: Box<Cipher>,
               balancer: Box<Balancer>,
               local_address: SocketAddr,
               public_address: Option<SocketAddr>,
               request_timeout: Option<Duration>)
               -> Direct {

        let statistic = Arc::new(Statistic::new());
        balancer.assign_statistic(statistic.clone());

        Direct {
            join_handle: RwLock::new(None),
            running: Arc::new(AtomicBool::new(false)),
            local_address: local_address,
            public_address: public_address.unwrap_or(local_address),
            cipher: Arc::new(cipher),
            connections: Arc::new(ConnectionMap::new()),
            services: Arc::new(ServiceMap::new(balancer)),
            tracker: Arc::new(Tracker::new(statistic.clone(), request_timeout)),
        }
    }

    fn unbind(&mut self) -> Result<()> {
        self.running.store(false, Ordering::SeqCst);
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

        let public_address = self.public_address;
        let running_clone = self.running.clone();
        let cipher_clone = self.cipher.clone();
        let connections_clone = self.connections.clone();
        let services_clone = self.services.clone();
        let tracker_clone = self.tracker.clone();
        *self.join_handle.write().unwrap() = Some(thread::spawn(move || {
            running_clone.store(true, Ordering::SeqCst);
            for stream in tcp_listener.incoming() {
                if !running_clone.load(Ordering::SeqCst) {
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

    fn connection_count(&self) -> usize {
        self.connections.len()
    }

    fn register(&self, name: &str, f: Box<request::Handler>) -> Result<()> {
        try!(self.services.insert_local(name, f));

        self.connections.send_add_services(&vec![name.to_string()]).unwrap();

        Ok(())
    }

    fn deregister(&self, name: &str) -> Result<()> {
        self.connections.send_remove_services(&vec![name.to_string()]).unwrap();

        try!(self.services.remove_local(name));

        Ok(())
    }

    fn service_count(&self) -> usize {
        self.services.len()
    }

    fn request(&self,
               name: &str,
               reader: Box<request::Reader>,
               response_writer: Box<request::ResponseWriter>)
               -> request::Response {
        let reader = Arc::new(Mutex::new(Some(reader)));
        let response_writer = Arc::new(Mutex::new(Some(response_writer)));
        self.services.select(name,
                             |handler| {
                                 let response_writer = response_writer.lock()
                                                                      .unwrap()
                                                                      .take()
                                                                      .unwrap();
                                 let (request_id, response_rx) = self.tracker
                                                                     .begin(name,
                                                                            &Link::Local,
                                                                            response_writer);
                                 let reader_clone = reader.clone();
                                 let handler_clone = handler.clone();
                                 let tracker_clone = self.tracker.clone();
                                 thread::spawn(move || {
                                     let reader = reader_clone.lock()
                                                              .unwrap()
                                                              .take()
                                                              .unwrap();

                                     let mut response = (*handler_clone.lock()
                                                                       .unwrap())(reader);

                                     if let Ok(ref mut reader) = response {
                                         if let Some(ref mut response_writer) =
                                                tracker_clone.take_response_writer(request_id) {
                                             io::copy(reader, response_writer).unwrap();
                                         }
                                     }

                                     if let Err(tracker::Error::AlreadyEnded) =
                                            tracker_clone.end(request_id, response) {
                                         debug!("got response for request ({}) that already \
                                                 timed out",
                                                request_id);
                                     }
                                 });
                                 response_rx.recv().unwrap()
                             },
                             |peer_node_id| {
                                 let response_writer = response_writer.lock()
                                                                      .unwrap()
                                                                      .take()
                                                                      .unwrap();

                                 let (request_id, response_rx) =
                                     self.tracker
                                         .begin(name, &Link::Remote(peer_node_id), response_writer);
                                 self.connections
                                     .send_request(&peer_node_id,
                                                   request_id,
                                                   name,
                                                   reader.lock()
                                                         .unwrap()
                                                         .as_mut()
                                                         .unwrap())
                                     .unwrap();
                                 response_rx.recv().unwrap()
                             })
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

fn set_up(connection: &mut Connection, services: &Arc<ServiceMap>, tracker: &Arc<Tracker>) {
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
    connection.set_on_response(Box::new(move |request_id, mut response| {
        if let Ok(ref mut reader) = response {
            if let Some(ref mut response_writer) = tracker_clone.take_response_writer(request_id) {
                io::copy(reader, response_writer).unwrap();
            } else {
                io::copy(reader, &mut io::sink()).unwrap();
            }
        }

        if let Err(tracker::Error::AlreadyEnded) = tracker_clone.end(request_id, response) {
            debug!("got response for request ({}) that already timed out",
                   request_id);
        }
    }));

    let services_clone = services.clone();
    connection.set_on_drop(Box::new(move |peer_node_id| {
        services_clone.remove_all_remotes(&peer_node_id);
    }));
}
