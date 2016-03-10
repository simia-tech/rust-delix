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
use std::result;
use std::sync::{Arc, mpsc};

use discovery::Discovery;
use metric::{self, Metric};
use node::{ID, Service, request, response};
use transport;
use transport::Transport;

pub struct Node {
    pub id: ID,
    discovery: Box<Discovery>,
    transport: Box<Transport>,
    request_counter: metric::item::Counter,
}

pub type Result<T> = result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    NoSocketAddr,
    Transport(transport::Error),
    Request(request::Error),
}

impl Node {
    pub fn new(discovery: Box<Discovery>,
               transport: Box<Transport>,
               metric: Arc<Metric>)
               -> Result<Self> {
        let node_id = ID::new_random();

        try!(transport.bind(node_id));

        Ok(Node {
            id: node_id,
            discovery: discovery,
            transport: transport,
            request_counter: metric.counter("requests"),
        })
    }

    pub fn join(&self) {
        while let Some(address) = self.discovery.next() {
            match self.transport.join(address, self.id) {
                Ok(()) => break,
                Err(error) => {
                    error!("{}: failed to connect to {}: {:?}", self.id, address, error);
                }
            }
        }
    }

    pub fn register(&self, name: &str, f: Box<Service>) -> Result<()> {
        try!(self.transport.register(name, f));
        Ok(())
    }

    pub fn deregister(&self, name: &str) -> Result<()> {
        try!(self.transport.deregister(name));
        Ok(())
    }

    pub fn request_bytes(&self, name: &str, request: &[u8]) -> request::Result<Vec<u8>> {
        let (tx, rx) = mpsc::channel();

        try!(self.request(name,
                          Box::new(io::Cursor::new(request.to_vec())),
                          Box::new(move |mut reader| {
                              let mut response = Vec::new();
                              io::copy(&mut reader, &mut response).unwrap();
                              tx.send(response).unwrap();
                          })));

        Ok(rx.recv().unwrap())
    }

    pub fn request(&self,
                   name: &str,
                   reader: Box<request::Reader>,
                   response_handler: Box<response::Handler>)
                   -> request::Result<()> {
        self.request_counter.increment();
        Ok(try!(self.transport.request(name, reader, response_handler)))
    }
}

impl fmt::Debug for Node {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "(Node {})", self.id)
    }
}

impl From<transport::Error> for Error {
    fn from(error: transport::Error) -> Self {
        Error::Transport(error)
    }
}

impl From<request::Error> for Error {
    fn from(error: request::Error) -> Self {
        Error::Request(error)
    }
}
