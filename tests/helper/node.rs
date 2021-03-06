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

use std::net::ToSocketAddrs;
use std::sync::Arc;

use self::time::Duration;

use delix::discovery::Constant;
use delix::metric::{self, Query};
use delix::node::Node;
use delix::transport::Direct;
use delix::transport::cipher;
use delix::transport::direct::balancer;

pub fn build_node(local_address: &str,
                  discover_addresses: &[&str],
                  request_timeout: Option<i64>)
                  -> (Arc<Node>, Arc<metric::Memory>) {

    let cipher = Box::new(cipher::Symmetric::new(b"000102030405060708090a0b0c0d0e0f", None)
                              .unwrap());

    let balancer_factory = Box::new(balancer::DynamicRoundRobinFactory::new());
    let discovery = Box::new(Constant::new(discover_addresses.to_vec()
                                                             .iter()
                                                             .map(|s| {
                                                                 s.to_socket_addrs()
                                                                  .unwrap()
                                                                  .next()
                                                                  .unwrap()
                                                             })
                                                             .collect()));

    let metric = Arc::new(metric::Memory::new());
    let transport = Box::new(Direct::new(cipher,
                                         balancer_factory,
                                         metric.clone(),
                                         local_address.to_socket_addrs().unwrap().next().unwrap(),
                                         None,
                                         request_timeout.map(|value| {
                                             Duration::milliseconds(value)
                                         })));

    let node = Arc::new(Node::new(discovery, transport, metric.clone()).unwrap());
    node.join();
    (node, metric)
}

pub fn wait_for_joined(queries: &[&Arc<metric::Memory>]) {
    let required_connections = queries.len() as isize - 1;
    for &query in queries {
        query.watch("connections",
                    move |_, value| *value < metric::Value::Gauge(required_connections));
    }
}

pub fn wait_for_discovering(query: &Arc<metric::Memory>) {
    query.watch("connections", |_, value| *value > metric::Value::Gauge(0));
}

pub fn wait_for_services(queries: &[&Arc<metric::Memory>], count: isize) {
    for &query in queries {
        query.watch("services",
                    move |_, value| *value != metric::Value::Gauge(count));
    }
}

pub fn wait_for_endpoints(queries: &[&Arc<metric::Memory>], count: isize) {
    for &query in queries {
        query.watch("endpoints",
                    move |_, value| *value != metric::Value::Gauge(count));
    }
}

pub fn wait_for_requests(queries: &[&Arc<metric::Memory>], minimum: usize) {
    for &query in queries {
        query.watch("requests",
                    move |_, value| *value < metric::Value::Counter(minimum));
    }
}
