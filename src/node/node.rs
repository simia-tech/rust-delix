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
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;

use discovery::Discovery;
use node::{ID, State, request};
use transport;
use transport::Transport;

pub struct Node {
    id: ID,
    transport: Arc<Box<Transport>>,
    join_handle: Option<thread::JoinHandle<()>>,
    running: Arc<AtomicBool>,
}

pub type Result<T> = result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    NoSocketAddr,
    Transport(transport::Error),
    Request(request::Error),
}

impl Node {
    pub fn new(d: Box<Discovery>, t: Box<Transport>) -> Result<Node> {
        let node_id = ID::new_random();

        try!(t.bind(node_id));

        let running = Arc::new(AtomicBool::new(true));
        let running_clone = running.clone();

        let discovery = Arc::new(d);
        let transport = Arc::new(t);

        let discovery_clone = discovery.clone();
        let transport_clone = transport.clone();

        let join_handle = Some(thread::spawn(move || {
            while running_clone.load(Ordering::SeqCst) {
                if transport_clone.connection_count() == 0 {
                    if let Some(address) = discovery_clone.discover() {
                        if let Err(err) = transport_clone.join(address, node_id) {
                            error!("{}: failed to connect to {}: {:?}", node_id, address, err);
                        }
                    }
                }
                thread::sleep_ms(2000);
            }
        }));

        Ok(Node {
            id: node_id,
            transport: transport,
            join_handle: join_handle,
            running: running,
        })
    }

    pub fn id(&self) -> ID {
        self.id
    }

    pub fn state(&self) -> State {
        if self.transport.connection_count() == 0 {
            State::Discovering
        } else {
            State::Joined
        }
    }

    pub fn connection_count(&self) -> usize {
        self.transport.connection_count()
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
        let mut response = try!(self.request(name, Box::new(io::Cursor::new(request.to_vec()))));
        let mut response_bytes = Vec::new();
        response.read_to_end(&mut response_bytes).unwrap();
        Ok(response_bytes)
    }

    pub fn request(&self, name: &str, reader: Box<request::Reader>) -> request::Result {
        Ok(try!(self.transport.request(name, Box::new(reader))))
    }
}

impl fmt::Display for Node {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f,
               "(Node {} {} {} connections)",
               self.id,
               self.state(),
               self.connection_count())
    }
}

impl Drop for Node {
    fn drop(&mut self) {
        self.running.store(false, Ordering::SeqCst);
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
