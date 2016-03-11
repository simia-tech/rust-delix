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
use std::net::{self, SocketAddr};
use std::sync::{Arc, Mutex, RwLock, mpsc};
use std::thread;
use time::Duration;

use openssl::ssl;

use transport::{Result, Transport};
use metric::Metric;
use node::{ID, Service, request, response};
use super::{Connection, ConnectionMap, Handlers, Link, Tracker, ServiceMap, balancer};
use super::tracker::Statistic;

pub struct Direct {
    join_handle: RwLock<Option<thread::JoinHandle<()>>>,
    running: Arc<RwLock<bool>>,
    local_address: SocketAddr,
    public_address: SocketAddr,
    ssl_context: Arc<RwLock<ssl::SslContext>>,
    connections: Arc<ConnectionMap>,
    services: Arc<ServiceMap>,
    tracker: Arc<Tracker<Mutex<Box<response::Handler>>, request::Result<()>>>,
}

impl Direct {
    pub fn new(ssl_context: ssl::SslContext,
               mut balancer_factory: Box<balancer::Factory>,
               metric: Arc<Metric>,
               local_address: SocketAddr,
               public_address: Option<SocketAddr>,
               request_timeout: Option<Duration>)
               -> Self {

        let statistic = Arc::new(Statistic::new());
        balancer_factory.set_statistic(statistic.clone());

        Direct {
            join_handle: RwLock::new(None),
            running: Arc::new(RwLock::new(false)),
            local_address: local_address,
            public_address: public_address.unwrap_or(local_address),
            ssl_context: Arc::new(RwLock::new(ssl_context)),
            connections: Arc::new(ConnectionMap::new(metric.clone())),
            services: Arc::new(ServiceMap::new(balancer_factory, metric.clone())),
            tracker: Arc::new(Tracker::new(statistic.clone(), request_timeout)),
        }
    }

    fn unbind(&self) -> Result<()> {
        *self.running.write().unwrap() = false;
        if let Some(join_handle) = self.join_handle.write().unwrap().take() {
            // connect to local address to enable the thread to escape the accept loop.
            try!(net::TcpStream::connect(self.local_address));
            join_handle.join().unwrap();
        }
        Ok(())
    }
}

impl Transport for Direct {
    fn public_address(&self) -> SocketAddr {
        self.public_address
    }

    fn bind(&self, node_id: ID) -> Result<()> {
        let tcp_listener = try!(net::TcpListener::bind(self.local_address));

        *self.running.write().unwrap() = true;

        let public_address = self.public_address;
        let running_clone = self.running.clone();
        let ssl_context_clone = self.ssl_context.clone();
        let connections_clone = self.connections.clone();
        let services_clone = self.services.clone();
        let tracker_clone = self.tracker.clone();
        *self.join_handle.write().unwrap() = Some(thread::spawn(move || {
            for stream in tcp_listener.incoming() {
                if !*running_clone.read().unwrap() {
                    break;
                }

                if let Err(error) = accept(stream.unwrap(),
                                           &ssl_context_clone,
                                           node_id,
                                           public_address,
                                           &connections_clone,
                                           &services_clone,
                                           &tracker_clone) {
                    error!("error accepting connection: {:?}", error);
                }
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

                let tcp_stream = try!(net::TcpStream::connect(peer_public_address));
                let ssl_stream = try!(ssl::SslStream::connect(&*self.ssl_context.read().unwrap(),
                                                              tcp_stream));
                let handlers = build_handlers(&self.connections, &self.services, &self.tracker);
                let (connection, peers) = try!(Connection::new_outbound(ssl_stream,
                                                                        node_id,
                                                                        self.public_address,
                                                                        handlers));
                let peer_node_id = connection.peer_node_id();
                info!("{}: outbound {}", node_id, connection);
                try!(self.connections.add(connection));

                tx.send(peers).unwrap();

                try!(try!(self.connections
                              .select(&peer_node_id, |connection| -> io::Result<()> {
                                  Ok(try!(connection.send_add_services(&self.services
                                                                       .local_service_names())))
                              })));
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
               mut reader: Box<request::Reader>,
               response_handler: Box<response::Handler>)
               -> request::Result<()> {

        let (link, local_handler) = try!(self.services.get(name));

        match link {
            Link::Local => {
                let (request_id, response_rx) = self.tracker
                                                    .begin(name,
                                                           &Link::Local,
                                                           Mutex::new(response_handler));
                let tracker_clone = self.tracker.clone();
                thread::spawn(move || {
                    let service_result = local_handler.unwrap()(reader);

                    let timed_out = !tracker_clone.end(request_id, |response_handler| {
                        let service_result = service_result;
                        match service_result {
                            Ok(reader) => {
                                (&mut **response_handler.lock()
                                                        .unwrap())(reader);
                                Ok(())
                            }
                            Err(error) => Err(request::Error::Service(error)),
                        }
                    });

                    if timed_out {
                        debug!("got response for request ({}) that already timed out",
                               request_id);
                    }
                });
                try!(response_rx.recv().unwrap())
            }
            Link::Remote(peer_node_id) => {
                let (request_id, response_rx) = self.tracker
                                                    .begin(name,
                                                           &Link::Remote(peer_node_id),
                                                           Mutex::new(response_handler));
                try!(self.connections
                         .send_request(&peer_node_id, request_id, name, &mut reader));
                try!(response_rx.recv().unwrap())
            }
        }
    }
}

impl Drop for Direct {
    fn drop(&mut self) {
        self.unbind().unwrap();
        self.connections.shutdown();
    }
}

fn accept(tcp_stream: net::TcpStream,
          ssl_context: &Arc<RwLock<ssl::SslContext>>,
          node_id: ID,
          public_address: SocketAddr,
          connections: &Arc<ConnectionMap>,
          services: &Arc<ServiceMap>,
          tracker: &Arc<Tracker<Mutex<Box<response::Handler>>, request::Result<()>>>)
          -> Result<()> {

    let ssl_stream = try!(ssl::SslStream::accept(&*ssl_context.read().unwrap(), tcp_stream));

    let peers = &connections.id_public_address_pairs();
    let handlers = build_handlers(connections, services, tracker);
    let connection = try!(Connection::new_inbound(ssl_stream,
                                                  node_id,
                                                  public_address,
                                                  peers,
                                                  handlers));
    let peer_node_id = connection.peer_node_id();
    info!("{}: inbound {}", node_id, connection);
    try!(connections.add(connection));

    try!(try!(connections.select(&peer_node_id, |connection| -> io::Result<()> {
        Ok(try!(connection.send_add_services(&services.local_service_names())))
    })));

    Ok(())
}

fn build_handlers(connections: &Arc<ConnectionMap>,
                  services: &Arc<ServiceMap>,
                  tracker: &Arc<Tracker<Mutex<Box<response::Handler>>, request::Result<()>>>)
                  -> Handlers {

    let connections_request_clone = connections.clone();
    let services_add_clone = services.clone();
    let services_remove_clone = services.clone();
    let services_request_clone = services.clone();
    let services_drop_clone = services.clone();
    let tracker_response_clone = tracker.clone();
    let tracker_drop_clone = tracker.clone();

    Handlers {
        add_services: Box::new(move |peer_node_id, services| {
            services_add_clone.insert_remotes(&services, peer_node_id);
        }),
        remove_services: Box::new(move |peer_node_id, services| {
            services_remove_clone.remove_remotes(&services, &peer_node_id);
        }),
        request: Box::new(move |peer_node_id, request_id, name, reader| {
            let connections_clone = connections_request_clone.clone();
            let services_clone = services_request_clone.clone();
            let name = name.to_string();
            thread::spawn(move || {
                let handler = services_clone.get_local(&name).unwrap();
                let service_result = handler(reader);
                if let Err(error) = connections_clone.send_response(&peer_node_id,
                                                                    request_id,
                                                                    service_result) {
                    error!("error while sending response: {:?}", error);
                }
            });
        }),
        response: Box::new(move |request_id, service_result| {
            let success = tracker_response_clone.end(request_id, |response_handler| {
                let service_result = service_result;
                match service_result {
                    Ok(reader) => {
                        thread::spawn(move || {
                            (&mut **response_handler.lock().unwrap())(reader);
                        });
                        Ok(())
                    }
                    Err(error) => Err(request::Error::Service(error)),
                }
            });

            if !success {
                debug!("got response for request ({}) that already timed out",
                       request_id);
            }

            Ok(())
        }),
        drop: Box::new(move |peer_node_id| {
            tracker_drop_clone.cancel(&peer_node_id);
            services_drop_clone.remove_all_remotes(&peer_node_id);
        }),
    }
}
