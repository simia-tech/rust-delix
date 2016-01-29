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
extern crate time;

use std::io::Read;
use std::net::SocketAddr;
use std::sync::{self, Arc, mpsc};

use self::hyper::client::response::Response;
use self::hyper::status::StatusCode;
use self::time::Duration;

use delix::discovery::Constant;
use delix::logger;
use delix::metric::{self, Metric};
use delix::node::Node;
use delix::transport::{Direct, cipher};
use delix::transport::direct::balancer;
use delix::relay::{self, Relay};

static START: sync::Once = sync::ONCE_INIT;

pub fn set_up() {
    START.call_once(|| {
        logger::Console::init(log::LogLevelFilter::Trace, "delix").unwrap();
    });
}

pub fn build_node(local_address: &str,
                  discover_addresses: &[&str],
                  request_timeout: Option<i64>)
                  -> Arc<Node<metric::Memory>> {

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

    Arc::new(Node::new(discovery, transport, metric).unwrap())
}

pub fn build_http_static_relay<M>(node: &Arc<Node<M>>,
                                  address: Option<&str>)
                                  -> Arc<relay::HttpStatic<M>>
    where M: Metric
{
    let relay = relay::HttpStatic::new(node.clone(), "X-Delix-Service");
    if let Some(address) = address {
        relay.bind(address.parse::<SocketAddr>().unwrap()).unwrap();
    }
    Arc::new(relay)
}

pub fn wait_for_joined(nodes: &[&Arc<Node<metric::Memory>>]) {
    let required_connections = nodes.len() as isize - 1;
    for node in nodes {
        node.metric().watch_gauge("connections", move |_, value| value < required_connections);
    }
}

pub fn wait_for_discovering(node: &Arc<Node<metric::Memory>>) {
    node.metric().watch_gauge("connections", |_, value| value > 0);
}

pub fn recv_all<T>(rx: &mpsc::Receiver<T>) -> Vec<T> {
    let mut result = Vec::new();
    loop {
        result.push(match rx.try_recv() {
            Ok(value) => value,
            Err(mpsc::TryRecvError::Empty) => break,
            Err(error) => panic!(error),
        });
    }
    result
}

pub fn assert_response(expected_status_code: StatusCode,
                       expected_body: &[u8],
                       response: &mut Response) {
    assert_eq!(expected_status_code, response.status);
    let mut response_body = String::new();
    response.read_to_string(&mut response_body).unwrap();
    assert_eq!(String::from_utf8_lossy(expected_body), response_body);
}

pub fn assert_contains_all<T: PartialEq>(expected: &[T], actual: &Vec<T>) {
    for e in expected {
        assert!(actual.contains(e));
    }
}
