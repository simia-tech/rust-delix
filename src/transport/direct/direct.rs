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

use time::Duration;

use transport::{Result, Transport};
use transport::direct::{Balancer, Connection, ConnectionMap, Tracker, ServiceMap};
use transport::direct::tracker::{Statistic, Subject};

use node::{ID, request};
use stats::StatCollector;

pub struct Direct {
    join_handle: RwLock<Option<thread::JoinHandle<()>>>,
    running: Arc<AtomicBool>,
    local_address: SocketAddr,
    public_address: SocketAddr,
    connections: Arc<ConnectionMap>,
    services: Arc<ServiceMap>,
    tracker: Arc<Tracker>,
    stat_collector: Arc<Box<StatCollector>>,
}

impl Direct {
    pub fn new(balancer: Box<Balancer>,
               local_address: SocketAddr,
               public_address: Option<SocketAddr>,
               request_timeout: Option<Duration>,
               stat_collector: Box<StatCollector>)
               -> Direct {

        let statistic = Arc::new(Statistic::new());
        balancer.assign_statistic(statistic.clone());

        Direct {
            join_handle: RwLock::new(None),
            running: Arc::new(AtomicBool::new(false)),
            local_address: local_address,
            public_address: public_address.unwrap_or(local_address),
            connections: Arc::new(ConnectionMap::new()),
            services: Arc::new(ServiceMap::new(balancer)),
            tracker: Arc::new(Tracker::new(statistic.clone(), request_timeout)),
            stat_collector: Arc::new(stat_collector),
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
        let connections_clone = self.connections.clone();
        let services_clone = self.services.clone();
        let tracker_clone = self.tracker.clone();
        let stat_collector_clone = self.stat_collector.clone();
        *self.join_handle.write().unwrap() = Some(thread::spawn(move || {
            running_clone.store(true, Ordering::SeqCst);
            for stream in tcp_listener.incoming() {
                if !running_clone.load(Ordering::SeqCst) {
                    break;
                }

                let stream = stream.unwrap();
                let peers = &connections_clone.id_public_address_pairs();
                let mut connection = Connection::new_inbound(stream,
                                                             node_id,
                                                             public_address,
                                                             peers)
                                         .unwrap();

                set_up(&mut connection, &services_clone, &tracker_clone);

                connection.send_services(&services_clone.local_service_names())
                          .unwrap();

                println!("{}: inbound {}", node_id, connection);
                connections_clone.add(connection).unwrap();
                stat_collector_clone.increment(
                    &["transport", "direct", "connections", "inbound"]);
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
                                                                            node_id,
                                                                            self.public_address));
                tx.send(peers).unwrap();

                set_up(&mut connection, &self.services, &self.tracker);

                try!(connection.send_services(&self.services.local_service_names()));

                println!("{}: outbound {}", node_id, connection);
                self.connections.add(connection).unwrap();
                self.stat_collector.increment(
                    &["transport", "direct", "connections", "outbound"]);
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

        self.connections.send_services(&self.services.local_service_names()).unwrap();

        Ok(())
    }

    fn deregister(&self, name: &str) -> Result<()> {
        try!(self.services.remove_local(name));
        Ok(())
    }

    fn service_count(&self) -> usize {
        self.services.len()
    }

    fn request(&self, name: &str, data: &[u8]) -> request::Response {
        self.services.select(name,
                             |handler| {
                                 let (request_id, _) = self.tracker
                                                           .begin(Subject::local(name));
                                 let response = handler(data)
                                                    .map_err(|text| request::Error::Internal(text));
                                 self.tracker.end(request_id, None).unwrap();
                                 response
                             },
                             |peer_node_id| {
                                 let (request_id, repsonse_rx) =
                                     self.tracker
                                         .begin(Subject::remote(name, peer_node_id));
                                 self.connections
                                     .send_request(&peer_node_id, request_id, name, data)
                                     .unwrap();
                                 repsonse_rx.recv().unwrap()
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
    connection.set_on_services(Box::new(move |peer_node_id, services| {
        for service in services {
            services_clone.insert_remote(&service, peer_node_id).unwrap();
        }
    }));

    let services_clone = services.clone();
    connection.set_on_request(Box::new(move |name, data| {
        services_clone.select_local(name, |handler| {
            handler(data).map_err(|text| request::Error::Internal(text))
        })
    }));

    let tracker_clone = tracker.clone();
    connection.set_on_response(Box::new(move |request_id, response| {
        tracker_clone.end(request_id, Some(response)).unwrap();
    }));

    let services_clone = services.clone();
    connection.set_on_drop(Box::new(move |peer_node_id| {
        services_clone.remove_remote(&peer_node_id).unwrap();
    }));
}
