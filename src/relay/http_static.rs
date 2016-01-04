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

use std::net::{SocketAddr, TcpListener};
use std::sync::{Arc, RwLock};
use std::thread;

use node::Node;
use relay::{Relay, Result};

pub struct HttpStatic {
    node: Arc<Node>,
    join_handle: RwLock<Option<thread::JoinHandle<()>>>,
}

impl HttpStatic {
    pub fn new(node: Arc<Node>) -> HttpStatic {
        HttpStatic {
            node: node,
            join_handle: RwLock::new(None),
        }
    }

    pub fn add_service(&self, name: &str, address: &str) {}
}

impl Relay for HttpStatic {
    fn bind(&self, address: SocketAddr) -> Result<()> {
        let tcp_listener = try!(TcpListener::bind(address));

        *self.join_handle.write().unwrap() = Some(thread::spawn(move || {
            for stream in tcp_listener.incoming() {
                println!("got stream: {:?}", stream);
            }
        }));

        Ok(())
    }
}
