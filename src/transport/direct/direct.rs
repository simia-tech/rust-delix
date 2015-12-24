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
use std::sync::{Arc, Mutex, mpsc};
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;

use transport::{Error, Result, Transport};
use transport::direct::{Connection, ConnectionMap, Link, Tracker, ServiceMap};

use node::{ID, ServiceHandler};

pub struct Direct {
    thread: Option<thread::JoinHandle<()>>,
    running: Arc<AtomicBool>,
    local_address: SocketAddr,
    public_address: SocketAddr,
    connections: Arc<Mutex<ConnectionMap>>,
    services: Arc<Mutex<ServiceMap>>,
    tracker: Arc<Mutex<Tracker<Result<Vec<u8>>>>>,
}

impl Direct {
    pub fn new(local_address: SocketAddr, public_address: Option<SocketAddr>) -> Direct {
        Direct {
            thread: None,
            running: Arc::new(AtomicBool::new(false)),
            local_address: local_address,
            public_address: public_address.unwrap_or(local_address),
            connections: Arc::new(Mutex::new(ConnectionMap::new())),
            services: Arc::new(Mutex::new(ServiceMap::new())),
            tracker: Arc::new(Mutex::new(Tracker::new())),
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
                let mut connection = Connection::new(stream, node_id, public_address);

                set_up(&mut connection,
                       &connections_clone,
                       &services_clone,
                       &tracker_clone);

                connection.send_peers(&connections_clone.lock().unwrap().id_public_address_pairs())
                          .unwrap();

                connection.send_services(&services_clone.lock().unwrap().local_service_names())
                          .unwrap();

                println!("{}: inbound {}", node_id, connection);
                connections_clone.lock().unwrap().add(connection).unwrap();
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
                if self.connections.lock().unwrap().contains_key(&peer_node_id) {
                    continue;
                }

                pending_peers_count += 1;

                let stream = try!(TcpStream::connect(peer_public_address));
                let mut connection = Connection::new(stream, node_id, self.public_address);

                let tx = tx.clone();
                connection.set_on_peers(Box::new(move |peers| {
                    tx.send(peers).unwrap();
                }));

                set_up(&mut connection,
                       &self.connections,
                       &self.services,
                       &self.tracker);

                try!(connection.send_services(&self.services
                                                   .lock()
                                                   .unwrap()
                                                   .local_service_names()));

                println!("{}: outbound {}", node_id, connection);
                self.connections.lock().unwrap().add(connection).unwrap();
            }

            pending_peers_count -= 1;
        }

        Ok(())
    }

    fn connection_count(&self) -> usize {
        self.connections.lock().unwrap().len()
    }

    fn register(&mut self, name: &str, f: Box<ServiceHandler>) -> Result<()> {
        try!(self.services.lock().unwrap().insert_local(name, f));

        for (_, connection) in self.connections.lock().unwrap().iter_mut() {
            try!(connection.send_services(&self.services.lock().unwrap().local_service_names()));
        }

        Ok(())
    }

    fn deregister(&mut self, name: &str) -> Result<()> {
        try!(self.services.lock().unwrap().remove(name));
        Ok(())
    }

    fn service_count(&self) -> usize {
        self.services.lock().unwrap().len()
    }

    fn request(&mut self, name: &str, data: &[u8]) -> Result<Vec<u8>> {
        let services = self.services.lock().unwrap();
        let link = match services.get_link(name) {
            Some(link) => link,
            None => return Err(Error::ServiceDoesNotExists),
        };

        match *link {
            Link::Local(ref service_handler) => Ok(service_handler(data)),
            Link::Remote(ref peer_node_id) => {
                let mut connections = self.connections.lock().unwrap();
                let mut connection = connections.get_mut(peer_node_id).unwrap();

                let (request_id, result_channel) = self.tracker.lock().unwrap().begin();
                try!(connection.send_request(request_id, name, data));

                result_channel.recv().unwrap()
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
        self.connections.lock().unwrap().shutdown_all().unwrap();
    }
}

fn set_up(connection: &mut Connection,
          connections: &Arc<Mutex<ConnectionMap>>,
          services: &Arc<Mutex<ServiceMap>>,
          tracker: &Arc<Mutex<Tracker<Result<Vec<u8>>>>>) {

    let services_clone = services.clone();
    connection.set_on_services(Box::new(move |peer_node_id, services| {
        for service in services {
            services_clone.lock().unwrap().insert_remote(&service, peer_node_id).unwrap();
        }
    }));

    let services_clone = services.clone();
    connection.set_on_request(Box::new(move |name, data| {
        let services = services_clone.lock().unwrap();
        let link = match services.get_link(name) {
            Some(link) => link,
            None => return Err(Error::ServiceDoesNotExists),
        };

        match *link {
            Link::Local(ref service_handler) => Ok(service_handler(data)),
            Link::Remote(_) => unimplemented!(),
        }
    }));

    let tracker_clone = tracker.clone();
    connection.set_on_response(Box::new(move |request_id, result| {
        tracker_clone.lock().unwrap().end(request_id, result).unwrap();
    }));

    let connections = connections.clone();
    connection.set_on_shutdown(Box::new(move |peer_node_id| {
        // remove the connection in a helper thread, otherwise the removed connection will
        // drop and tries to join its own thread, which results in a panic.
        let connections = connections.clone();
        thread::spawn(move || {
            connections.lock().unwrap().remove(&peer_node_id).unwrap();
        });
    }));
}
