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
use std::net::AddrParseError;
use std::result;
use time::Duration;

use delix::node;
use delix::node::Node;
use delix::discovery::Constant;
use delix::discovery::Discovery;
use delix::transport::Transport;
use delix::transport::Direct;
use delix::transport::direct::{Balancer, balancer};
use delix::stats::MultiStatCollector;
use delix::stats::DebugStatCollector;
use delix::stats::StatCollector;
use configuration::Configuration;

#[derive(Debug)]
pub struct Loader;

pub type Result<T> = result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    NoDiscoveryType,
    UnknownDiscoveryType(String),
    NoTransportType,
    UnknownTransportType(String),
    NoBalancerType,
    UnknownBalancerType(String),
    NoLocalAddress,
    NodeError(node::Error),
    AddrParseError(AddrParseError),
}

impl Loader {
    pub fn load_node(configuration: &Configuration) -> Result<Node> {
        let discovery_type = match configuration.string_at("discovery.type") {
            Some(discovery_type) => discovery_type,
            None => return Err(Error::NoDiscoveryType),
        };

        let discovery: Box<Discovery> = match discovery_type.as_ref() {
            "constant" => {
                Box::new(Constant::new(try!(configuration.strings_at("discovery.addresses")
                                                         .unwrap_or(vec![])
                                                         .iter()
                                                         .map(|s| s.parse::<SocketAddr>())
                                                         .collect())))
            }
            _ => return Err(Error::UnknownDiscoveryType(discovery_type)),
        };

        let stat_collectors: Vec<Box<StatCollector>> = configuration.strings_at("stats.collectors")
            .unwrap_or(vec![])
            .iter()
            .filter_map(|c| -> Option<Box<StatCollector>> {
                match c.as_ref() {
                    "debug" => Some(Box::new(DebugStatCollector)),
                    _ => None,
                }})
            .collect();

        let stat_collector = Box::new(MultiStatCollector::new(stat_collectors));

        let transport_type = match configuration.string_at("transport.type") {
            Some(transport_type) => transport_type,
            None => return Err(Error::NoTransportType),
        };

        let transport: Box<Transport> = match transport_type.as_ref() {
            "direct" => {
                let local_address = match configuration.string_at("transport.local_address") {
                    Some(local_address) => try!(local_address.parse::<SocketAddr>()),
                    None => return Err(Error::NoLocalAddress),
                };
                let public_address = match configuration.string_at("transport.public_address") {
                    Some(public_address) => Some(try!(public_address.parse::<SocketAddr>())),
                    None => None,
                };
                let request_timeout = configuration.i64_at("transport.request_timeout_ms")
                                                   .map(|value| Duration::milliseconds(value));

                let balancer_type = match configuration.string_at("transport.balancer.type") {
                    Some(balancer_type) => balancer_type,
                    None => return Err(Error::NoBalancerType),
                };

                let balancer: Box<Balancer> = match balancer_type.as_ref() {
                    "dynamic_round_robin" => Box::new(balancer::DynamicRoundRobin::new()),
                    _ => return Err(Error::UnknownBalancerType(balancer_type)),
                };

                Box::new(Direct::new(balancer, local_address, public_address, request_timeout, stat_collector))
            }
            _ => return Err(Error::UnknownTransportType(transport_type)),
        };

        let node = try!(Node::new(discovery, transport));
        Ok(node)
    }
}

impl From<node::Error> for Error {
    fn from(error: node::Error) -> Self {
        Error::NodeError(error)
    }
}

impl From<AddrParseError> for Error {
    fn from(error: AddrParseError) -> Self {
        Error::AddrParseError(error)
    }
}
