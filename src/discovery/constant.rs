/*
Copyright 2015 The Delix Project Authors. See the AUTHORS file at the top level directory.

Licensed under the Apache License, Version 2.0 (the "License");
you may not use this file except in compliance with the License.
You may obtain a copy of the License at

http://www.apache.org/licenses/LICENSE-2.0

Unless required by applicable law or agreed to in writing, software
distributed under the License is distributed on an "AS IS" BASIS,
WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
See the License for the specific language governing permissions and
limitations under the License.
*/

use std::net::{SocketAddr, ToSocketAddrs};

use discovery::Discovery;

pub struct Constant {
    addresses: Vec<SocketAddr>,
    current_index: usize,
}

impl Constant {

    pub fn new<T: ToSocketAddrs>(inputs: &[T]) -> Constant {
        let mut addresses = Vec::new();
        for input in inputs {
            for socket_addr in input.to_socket_addrs().unwrap() {
                addresses.push(socket_addr);
            }
        }
        Constant {
            addresses: addresses,
            current_index: 0,
        }
    }

}

impl Discovery for Constant {

    fn discover(&mut self) -> Option<SocketAddr> {
        let result = self.addresses.get(self.current_index);
        self.current_index += 1;
        if self.current_index >= self.addresses.len() {
            self.current_index = 0;
        }
        match result {
            None => None,
            Some(result) => Some(*result),
        }
    }

}
