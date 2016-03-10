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

use std::net::SocketAddr;
use std::sync::RwLock;

use discovery::Discovery;

pub struct Constant {
    addresses: RwLock<Vec<SocketAddr>>,
    current_index: RwLock<usize>,
}

impl Constant {
    pub fn new(addresses: Vec<SocketAddr>) -> Constant {
        Constant {
            addresses: RwLock::new(addresses),
            current_index: RwLock::new(0),
        }
    }
}

impl Discovery for Constant {
    fn next(&self) -> Option<SocketAddr> {
        let addresses = self.addresses.read().unwrap();
        let mut current_index = self.current_index.write().unwrap();

        let result = addresses.get(*current_index);
        *current_index += 1;
        if *current_index >= addresses.len() {
            *current_index = 0;
        }
        result.map(|address| *address)
    }
}
