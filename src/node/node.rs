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
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::{JoinHandle, spawn, sleep_ms};

use discovery::Discovery;
use node::{ID, State};
use transport;
use transport::Transport;

pub struct Node {
    id: ID,
    transport: Arc<Mutex<Box<Transport>>>,
    thread: Option<JoinHandle<()>>,
    running: Arc<AtomicBool>,
}

pub type Result<T> = result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    NoSocketAddr,
    IO(io::Error),
    Transport(transport::Error),
}

impl Node {
    pub fn new(d: Box<Discovery>, t: Box<Transport>) -> Result<Node> {
        let node_id = ID::new_random();

        try!(t.bind(node_id));

        let running = Arc::new(AtomicBool::new(true));
        let running_clone = running.clone();

        let discovery = Arc::new(Mutex::new(d));
        let transport = Arc::new(Mutex::new(t));

        let discovery_mutex = discovery.clone();
        let transport_mutex = transport.clone();

        let thread = spawn(move || {
            while running_clone.load(Ordering::SeqCst) &&
                  transport_mutex.lock().unwrap().connection_count() == 0 {
                match discovery_mutex.lock().unwrap().discover() {
                    Some(address) => {
                        if let Err(err) = transport_mutex.lock().unwrap().join(address, node_id) {
                            println!("{}: failed to connect to {}: {:?}", node_id, address, err);
                        }
                        sleep_ms(2000);
                    }
                    None => {
                        println!("nothing to discover");
                        break;
                    }
                }
            }
        });

        Ok(Node {
            id: node_id,
            transport: transport,
            thread: Some(thread),
            running: running,
        })
    }

    pub fn id(&self) -> ID {
        self.id
    }

    pub fn state(&self) -> State {
        if (*self.transport.lock().unwrap()).connection_count() == 0 {
            State::Discovering
        } else {
            State::Joined
        }
    }

    pub fn connection_count(&self) -> usize {
        self.transport.lock().unwrap().connection_count()
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
        self.thread.take().unwrap().join().unwrap();
    }
}

impl From<transport::Error> for Error {
    fn from(error: transport::Error) -> Self {
        Error::Transport(error)
    }
}

impl From<io::Error> for Error {
    fn from(error: io::Error) -> Self {
        Error::IO(error)
    }
}
