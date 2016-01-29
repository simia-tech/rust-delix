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
use std::sync::{Arc, RwLock};
use std::thread;

use discovery::Discovery;
use metric::Metric;
use node::{ID, request};
use transport;
use transport::Transport;
use util::writer;

pub struct Node<M>
    where M: Metric
{
    id: ID,
    transport: Arc<Box<Transport>>,
    metric: Arc<M>,
    join_handle: Option<thread::JoinHandle<()>>,
    running: Arc<RwLock<bool>>,
}

pub type Result<T> = result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    NoSocketAddr,
    Transport(transport::Error),
    Request(request::Error),
}

impl<M> Node<M> where M: Metric
{
    pub fn new(d: Box<Discovery>, t: Box<Transport>, metric: Arc<M>) -> Result<Self> {
        let node_id = ID::new_random();

        try!(t.bind(node_id));

        let running = Arc::new(RwLock::new(true));
        let running_clone = running.clone();

        let discovery = Arc::new(d);
        let transport = Arc::new(t);

        let discovery_clone = discovery.clone();
        let transport_clone = transport.clone();

        let join_handle = Some(thread::spawn(move || {
            while *running_clone.read().unwrap() {
                if let Some(address) = discovery_clone.discover() {
                    match transport_clone.join(address, node_id) {
                        Ok(()) => break,
                        Err(error) => {
                            error!("{}: failed to connect to {}: {:?}", node_id, address, error);
                        }
                    }
                }
                thread::sleep(::std::time::Duration::from_millis(2000));
            }
        }));

        Ok(Node {
            id: node_id,
            transport: transport,
            metric: metric,
            join_handle: join_handle,
            running: running,
        })
    }

    pub fn id(&self) -> ID {
        self.id
    }

    pub fn metric(&self) -> &Arc<M> {
        &self.metric
    }

    pub fn register(&self, name: &str, f: Box<request::Handler>) -> Result<()> {
        try!(self.transport.register(name, f));
        Ok(())
    }

    pub fn deregister(&self, name: &str) -> Result<()> {
        try!(self.transport.deregister(name));
        Ok(())
    }

    pub fn service_count(&self) -> usize {
        self.transport.service_count()
    }

    pub fn request_bytes(&self,
                         name: &str,
                         request: &[u8])
                         -> result::Result<Vec<u8>, request::Error> {
        let response_writer = writer::Collector::new();

        try!(self.request(name,
                          Box::new(io::Cursor::new(request.to_vec())),
                          Box::new(response_writer.clone())));

        Ok(response_writer.vec().unwrap())
    }

    pub fn request(&self,
                   name: &str,
                   reader: Box<request::Reader>,
                   response_writer: Box<request::ResponseWriter>)
                   -> request::Response {
        Ok(try!(self.transport.request(name, reader, response_writer)))
    }
}

impl<M> fmt::Display for Node<M> where M: Metric
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "(Node {})", self.id)
    }
}

impl<M> Drop for Node<M> where M: Metric
{
    fn drop(&mut self) {
        *self.running.write().unwrap() = false;
        self.join_handle.take().unwrap().join().unwrap();
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
