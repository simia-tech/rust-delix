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

extern crate time;

use std::net::SocketAddr;
use std::sync::Arc;

use self::time::Duration;

use delix::discovery::Constant;
use delix::metric;
use delix::node::Node;
use delix::transport::{Direct, cipher};
use delix::transport::direct::balancer;

pub fn build_node(local_address: &str,
                  discover_addresses: &[&str],
                  request_timeout: Option<i64>)
                  -> Arc<Node> {

    let cipher = Box::new(cipher::Symmetric::new(b"test keytest key", None).unwrap());
    let balancer = Box::new(balancer::DynamicRoundRobin::new());
    let discovery = Box::new(Constant::new(discover_addresses.to_vec()
                                                             .iter()
                                                             .map(|s| {
                                                                 s.parse::<SocketAddr>().unwrap()
                                                             })
                                                             .collect()));

    let metric = Arc::new(metric::Memory::new());
    let transport = Box::new(Direct::new(cipher,
                                         balancer,
                                         metric.clone(),
                                         local_address.parse::<SocketAddr>().unwrap(),
                                         None,
                                         request_timeout.map(|value| {
                                             Duration::milliseconds(value)
                                         })));

    let node = Arc::new(Node::new(discovery, transport, metric).unwrap());
    node.join();
    node
}

pub fn wait_for_joined(nodes: &[&Arc<Node>]) {
    // let required_connections = nodes.len() as isize - 1;
    // for node in nodes {
    //     node.metric().watch_gauge("connections", move |_, value| value < required_connections);
    // }
}

pub fn wait_for_discovering(node: &Arc<Node>) {
    // node.metric().watch_gauge("connections", |_, value| value > 0);
}

pub fn wait_for_services(nodes: &[&Arc<Node>], count: isize) {
    // for node in nodes {
    //     node.metric().watch_gauge("services", move |_, value| value != count);
    // }
}
