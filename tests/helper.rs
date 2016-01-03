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

extern crate delix;

use std::net::SocketAddr;

use delix::discovery::Constant;
use delix::node::{Node, State};
use delix::transport::{Direct, cipher};
use delix::transport::direct::balancer;

pub fn build_node(local_address: &str, discover_addresses: &[&str]) -> Node {
    let cipher = Box::new(cipher::Symmetric::new(b"test keytest key", None).unwrap());
    let balancer = Box::new(balancer::DynamicRoundRobin::new());
    let discovery = Box::new(Constant::new(discover_addresses.to_vec()
                                                             .iter()
                                                             .map(|s| {
                                                                 s.parse::<SocketAddr>().unwrap()
                                                             })
                                                             .collect()));
    let transport = Box::new(Direct::new(cipher,
                                         balancer,
                                         local_address.parse::<SocketAddr>().unwrap(),
                                         None,
                                         None));
    Node::new(discovery, transport).unwrap()
}

pub fn assert_node(node: &Node, expected_state: State, expected_connection_count: usize) {
    assert_eq!(expected_state, node.state());
    assert_eq!(expected_connection_count, node.connection_count());
}
