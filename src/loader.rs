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

use std::net::{SocketAddr, ToSocketAddrs};
use std::io;
use std::result;
use std::sync::Arc;
use time::Duration;
use log;

use delix::logger;
use delix::metric::{self, Metric};
use delix::node::{self, Node};
use delix::discovery::{self, Discovery};
use delix::relay::{self, Relay};
use delix::transport::{self, Transport};
use delix::transport::cipher::{self, Cipher};
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
    NoAddresses,
    NoName,
    NoLocalAddress,
    NodeError(node::Error),
    Cipher(cipher::Error),
    Relay(relay::Error),
    Io(io::Error),
}

impl Loader {
    pub fn new(configuration: Configuration) -> Loader {
        Loader { configuration: configuration }
    }

    pub fn load_log(&self) -> Result<()> {
        let log_type = try!(self.configuration.string_at("log.type").ok_or(Error::NoLogType));

        let log_level_filter = match self.configuration
                                         .string_at("log.level")
                                         .unwrap_or("off".to_string())
                                         .as_ref() {
            "off" => log::LogLevelFilter::Off,
            "error" => log::LogLevelFilter::Error,
            "warn" => log::LogLevelFilter::Warn,
            "info" => log::LogLevelFilter::Info,
            "debug" => log::LogLevelFilter::Debug,
            "trace" => log::LogLevelFilter::Trace,
            _ => log::LogLevelFilter::Off,
        };

        match log_type.as_ref() {
            "console" => {
                logger::Console::init(log_level_filter, "delix").unwrap();
                info!("loaded console log");
            }
            _ => return Err(Error::UnknownLogType(log_type)),
        }

        Ok(())
    }

    pub fn load_node(&self) -> Result<(Arc<Node>, Arc<metric::Memory>)> {
        let discovery = try!(self.load_discovery());

        let cipher = try!(self.load_cipher());

        let metric = Arc::new(metric::Memory::new());

        let transport = try!(self.load_transport(cipher, metric.clone()));

        Ok((Arc::new(try!(Node::new(discovery, transport, metric.clone()))),
            metric))
    }

    fn load_discovery(&self) -> Result<Box<Discovery>> {
        let discovery_type = try!(self.configuration
                                      .string_at("discovery.type")
                                      .ok_or(Error::NoDiscoveryType));

        match discovery_type.as_ref() {
            "constant" => {
                let addresses = try!(self.configuration
                                         .strings_at("discovery.addresses")
                                         .ok_or(Error::NoAddresses));
                let addresses = try!(resolve_socket_addresses(&addresses));
                let constant = discovery::Constant::new(addresses);
                info!("loaded constant discovery");
                Ok(Box::new(constant))
            }
            _ => Err(Error::UnknownDiscoveryType(discovery_type)),
        }
    }

    fn load_cipher(&self) -> Result<Box<Cipher>> {
        let cipher_type = try!(self.configuration
                                   .string_at("cipher.type")
                                   .ok_or(Error::NoCipherType));

        match cipher_type.as_ref() {
            "symmetric" => {
                let key = match self.configuration.bytes_at("cipher.key") {
                    Some(key) => key,
                    None => return Err(Error::NoKey),
                };
                let cipher = try!(cipher::Symmetric::new(&key, None));
                info!("loaded symmetric cipher");
                Ok(Box::new(cipher))
            }
            _ => Err(Error::UnknownCipherType(cipher_type)),
        }
    }

    fn load_transport(&self, cipher: Box<Cipher>, metric: Arc<Metric>) -> Result<Box<Transport>> {
        let transport_type = try!(self.configuration
                                      .string_at("transport.type")
                                      .ok_or(Error::NoTransportType));

        match transport_type.as_ref() {
            "direct" => {
                let local_address = try!(self.configuration
                                             .string_at("transport.local_address")
                                             .ok_or(Error::NoLocalAddress));
                let local_address = try!(resolve_socket_address(&local_address));

                let public_address = match self.configuration
                                               .string_at("transport.public_address") {
                    Some(ref value) => Some(try!(resolve_socket_address(value))),
                    None => None,
                };

                let request_timeout = self.configuration
                                          .i64_at("transport.request_timeout_ms")
                                          .map(|value| Duration::milliseconds(value));

                let balancer_type = try!(self.configuration
                                             .string_at("transport.balancer.type")
                                             .ok_or(Error::NoBalancerType));

                let balancer: Box<Balancer> = match balancer_type.as_ref() {
                    "dynamic_round_robin" => Box::new(balancer::DynamicRoundRobin::new()),
                    _ => return Err(Error::UnknownBalancerType(balancer_type)),
                };

                info!("loaded direct transport - listening at {}", local_address);

                Ok(Box::new(transport::Direct::new(cipher,
                                                   balancer,
                                                   metric,
                                                   local_address,
                                                   public_address,
                                                   request_timeout)))
            }
            _ => Err(Error::UnknownTransportType(transport_type)),
        }
    }

    pub fn load_relays(&self, node: &Arc<Node>) -> Result<Vec<Box<Relay>>> {
        let mut relays = Vec::new();
        if let Some(configurations) = self.configuration.configurations_at("relay") {
            for configuration in configurations {
                let relay_type = try!(configuration.string_at("type").ok_or(Error::NoRelayType));

                let relay: Box<Relay> = match relay_type.as_ref() {
                    "http_static" => {
                        let address = configuration.string_at("address");
                        let header_field = configuration.string_at("header_field")
                                                        .unwrap_or("X-Delix-Service".to_string());

                        let http_static = relay::HttpStatic::new(node.clone(), &header_field);

                        if let Some(configurations) = configuration.configurations_at("service") {
                            for configuration in configurations {
                                let name = try!(configuration.string_at("name")
                                                             .ok_or(Error::NoName));
                                let address = try!(configuration.string_at("address")
                                                                .ok_or(Error::NoAddress));
                                let address = try!(resolve_socket_address(&address));
                                http_static.add_service(&name, address);
                            }
                        }

                        if let Some(ref address) = address {
                            try!(http_static.bind(try!(resolve_socket_address(address))));
                            info!("loaded http static relay - listening at {}", address);
                        } else {
                            info!("loaded http static relay");
                        }

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

impl From<io::Error> for Error {
    fn from(error: io::Error) -> Self {
        Error::Io(error)
    }
}

fn resolve_socket_address(address: &str) -> io::Result<SocketAddr> {
    Ok(try!(try!(address.to_socket_addrs())
                .next()
                .ok_or(io::Error::new(io::ErrorKind::Other,
                                      format!("could not resolve address [{}]", address)))))
}

fn resolve_socket_addresses(addresses: &[String]) -> io::Result<Vec<SocketAddr>> {
    let mut result = Vec::new();
    for address in addresses {
        result.append(&mut try!(address.to_socket_addrs()).collect::<Vec<SocketAddr>>());
    }
    Ok(result)
}
