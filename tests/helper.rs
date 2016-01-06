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
extern crate hyper;
extern crate log;

use std::io::Read;
use std::net::SocketAddr;
use std::sync::{self, Arc};

use self::hyper::client::response::Response;
use self::hyper::status::StatusCode;

use delix::discovery::Constant;
use delix::logger;
use delix::node::{Node, State};
use delix::transport::{Direct, cipher};
use delix::transport::direct::balancer;
use delix::relay::{self, Relay};

static START: sync::Once = sync::ONCE_INIT;

pub fn set_up() {
    START.call_once(|| {
        logger::Console::init(log::LogLevelFilter::Trace, "delix").unwrap();
    });
}

pub fn build_node(local_address: &str, discover_addresses: &[&str]) -> Arc<Node> {
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
    Arc::new(Node::new(discovery, transport).unwrap())
}

pub fn build_http_static_relay(node: &Arc<Node>, address: Option<&str>) -> Arc<relay::HttpStatic> {
    let relay = relay::HttpStatic::new(node.clone());
    if let Some(address) = address {
        relay.bind(address.parse::<SocketAddr>().unwrap()).unwrap();
    }
    Arc::new(relay)
}

pub fn assert_node(node: &Arc<Node>, expected_state: State, expected_connection_count: usize) {
    assert_eq!(expected_state, node.state());
    assert_eq!(expected_connection_count, node.connection_count());
}

pub fn assert_response(expected_status_code: StatusCode,
                       expected_body: &[u8],
                       response: &mut Response) {
    assert_eq!(expected_status_code, response.status);
    let mut response_body = String::new();
    response.read_to_string(&mut response_body).unwrap();
    assert_eq!(String::from_utf8_lossy(expected_body), response_body);
}
