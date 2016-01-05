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
use std::sync::Arc;
use time::Duration;

use delix::logger;
use delix::node::{self, Node};
use delix::discovery::{Constant, Discovery};
use delix::relay::{self, Relay};
use delix::transport::{Direct, Transport, cipher};
use delix::transport::direct::{Balancer, balancer};
use configuration::Configuration;

#[derive(Debug)]
pub struct Loader {
    configuration: Configuration,
}

pub type Result<T> = result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    NoLogType,
    UnknownLogType(String),
    NoDiscoveryType,
    UnknownDiscoveryType(String),
    NoCipherType,
    UnknownCipherType(String),
    NoTransportType,
    UnknownTransportType(String),
    NoBalancerType,
    UnknownBalancerType(String),
    NoRelayType,
    UnknownRelayType(String),
    NoKey,
    NoAddress,
    NoName,
    NoLocalAddress,
    NodeError(node::Error),
    AddrParseError(AddrParseError),
    Cipher(cipher::Error),
    Relay(relay::Error),
}

impl Loader {
    pub fn new(configuration: Configuration) -> Loader {
        Loader { configuration: configuration }
    }

    pub fn load_log(&self) -> Result<()> {
        let log_type = match self.configuration.string_at("log.type") {
            Some(log_type) => log_type,
            None => return Err(Error::NoLogType),
        };

        match log_type.as_ref() {
            "console" => {
                logger::Console::init().unwrap();
                info!("loaded console log");
            }
            _ => return Err(Error::UnknownLogType(log_type)),
        }

        Ok(())
    }

    pub fn load_node(&self) -> Result<Arc<Node>> {
        let discovery_type = match self.configuration.string_at("discovery.type") {
            Some(discovery_type) => discovery_type,
            None => return Err(Error::NoDiscoveryType),
        };

        let discovery: Box<Discovery> = match discovery_type.as_ref() {
            "constant" => {
                let constant = Constant::new(try!(self.configuration
                                                      .strings_at("discovery.addresses")
                                                      .unwrap_or(vec![])
                                                      .iter()
                                                      .map(|s| s.parse::<SocketAddr>())
                                                      .collect()));
                info!("loaded constant discovery");
                Box::new(constant)
            }
            _ => return Err(Error::UnknownDiscoveryType(discovery_type)),
        };

        let cipher_type = match self.configuration.string_at("cipher.type") {
            Some(cipher_type) => cipher_type,
            None => return Err(Error::NoCipherType),
        };

        let cipher: Box<cipher::Cipher> = match cipher_type.as_ref() {
            "symmetric" => {
                let key = match self.configuration.bytes_at("cipher.key") {
                    Some(key) => key,
                    None => return Err(Error::NoKey),
                };
                let cipher = try!(cipher::Symmetric::new(&key, None));
                info!("loaded symmetric cipher");
                Box::new(cipher)
            }
            _ => return Err(Error::UnknownCipherType(cipher_type)),
        };

        let transport_type = match self.configuration.string_at("transport.type") {
            Some(transport_type) => transport_type,
            None => return Err(Error::NoTransportType),
        };

        let transport: Box<Transport> = match transport_type.as_ref() {
            "direct" => {
                let local_address = match self.configuration.string_at("transport.local_address") {
                    Some(local_address) => try!(local_address.parse::<SocketAddr>()),
                    None => return Err(Error::NoLocalAddress),
                };
                let public_address = match self.configuration
                                               .string_at("transport.public_address") {
                    Some(public_address) => Some(try!(public_address.parse::<SocketAddr>())),
                    None => None,
                };
                let request_timeout = self.configuration
                                          .i64_at("transport.request_timeout_ms")
                                          .map(|value| Duration::milliseconds(value));

                let balancer_type = match self.configuration.string_at("transport.balancer.type") {
                    Some(balancer_type) => balancer_type,
                    None => return Err(Error::NoBalancerType),
                };

                let balancer: Box<Balancer> = match balancer_type.as_ref() {
                    "dynamic_round_robin" => Box::new(balancer::DynamicRoundRobin::new()),
                    _ => return Err(Error::UnknownBalancerType(balancer_type)),
                };

                info!("loaded and bound direct transport to {}", local_address);

                Box::new(Direct::new(cipher,
                                     balancer,
                                     local_address,
                                     public_address,
                                     request_timeout))
            }
            _ => return Err(Error::UnknownTransportType(transport_type)),
        };

        Ok(Arc::new(try!(Node::new(discovery, transport))))
    }

    pub fn load_relays(&self, node: &Arc<Node>) -> Result<Vec<Box<Relay>>> {
        let mut relays = Vec::new();
        if let Some(configurations) = self.configuration.configurations_at("relay") {
            for configuration in configurations {
                let relay_type = match configuration.string_at("type") {
                    Some(relay_type) => relay_type,
                    None => return Err(Error::NoRelayType),
                };

                let relay: Box<Relay> = match relay_type.as_ref() {
                    "http_static" => {
                        let http_static = relay::HttpStatic::new(node.clone());
                        let address = match configuration.string_at("address") {
                            Some(address) => try!(address.parse::<SocketAddr>()),
                            None => return Err(Error::NoAddress),
                        };

                        if let Some(configurations) = configuration.configurations_at("service") {
                            for configuration in configurations {
                                let name = match configuration.string_at("name") {
                                    Some(name) => name,
                                    None => return Err(Error::NoName),
                                };
                                let address = match configuration.string_at("address") {
                                    Some(address) => try!(address.parse::<SocketAddr>()),
                                    None => return Err(Error::NoAddress),
                                };
                                http_static.add_service(&name, address);
                            }
                        }

                        try!(http_static.bind(address));

                        info!("loaded and bound http static relay to {}", address);

                        Box::new(http_static)
                    }
                    _ => return Err(Error::UnknownRelayType(relay_type)),
                };

                relays.push(relay);
            }

        }
        Ok(relays)
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

impl From<cipher::Error> for Error {
    fn from(error: cipher::Error) -> Self {
        Error::Cipher(error)
    }
}

impl From<relay::Error> for Error {
    fn from(error: relay::Error) -> Self {
        Error::Relay(error)
    }
}
