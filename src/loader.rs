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
use delix::transport::direct::balancer;
use delix::util::resolve;
use configuration::Configuration;

#[derive(Debug)]
pub struct Loader {
    configuration: Configuration,
}

pub type Result<T> = result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    MissingField(&'static str),
    InvalidValue(&'static str, String, Vec<&'static str>),
    NodeError(node::Error),
    Cipher(cipher::Error),
    Relay(relay::Error),
    Resolve(io::Error),
}

impl Loader {
    pub fn new(configuration: Configuration) -> Loader {
        Loader { configuration: configuration }
    }

    pub fn load_metric(&self) -> Result<Arc<metric::Metric>> {
        let metric_type = try!(self.configuration
                                   .string_at("metric.type")
                                   .ok_or(Error::MissingField("metric.type")));

        match metric_type.as_ref() {
            "console" => {
                info!("loaded console metric");
                Ok(Arc::new(metric::Memory::new()))
            }
            "terminal" => {
                info!("loaded terminal metric");
                let refresh_interval_ms = self.configuration
                                              .i64_at("metric.refresh_interval_ms")
                                              .unwrap_or(100);
                Ok(Arc::new(metric::Terminal::new(refresh_interval_ms as u64)))
            }
            _ => {
                Err(Error::InvalidValue("metric.type",
                                        metric_type.to_string(),
                                        vec!["console", "terminal"]))
            }
        }
    }

    pub fn load_log(&self, metric: &Arc<Metric>) -> Result<()> {
        let log_type = try!(self.configuration
                                .string_at("log.type")
                                .ok_or(Error::MissingField("log.type")));

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
                logger::Console::init(log_level_filter, "delix", metric).unwrap();
                info!("loaded console log");
                Ok(())
            }
            _ => Err(Error::InvalidValue("log.type", log_type.to_string(), vec!["console"])),
        }
    }

    pub fn load_node(&self, metric: &Arc<metric::Metric>) -> Result<Arc<Node>> {
        let cipher = try!(self.load_cipher());
        let transport = try!(self.load_transport(cipher, metric.clone()));
        let discovery = try!(self.load_discovery(transport.public_address()));

        Ok(Arc::new(try!(Node::new(discovery, transport, metric.clone()))))
    }

    fn load_cipher(&self) -> Result<Box<Cipher>> {
        let cipher_type = try!(self.configuration
                                   .string_at("cipher.type")
                                   .ok_or(Error::MissingField("cipher.type")));

        match cipher_type.as_ref() {
            "symmetric" => {
                let key = try!(self.configuration
                                   .bytes_at("cipher.key")
                                   .ok_or(Error::MissingField("cipher.key")));
                let cipher = try!(cipher::Symmetric::new(&key, None));
                info!("loaded symmetric cipher");
                Ok(Box::new(cipher))
            }
            _ => {
                Err(Error::InvalidValue("cipher.type", cipher_type.to_string(), vec!["symmetric"]))
            }
        }
    }

    fn load_discovery(&self, public_address: SocketAddr) -> Result<Box<Discovery>> {
        let discovery_type = try!(self.configuration
                                      .string_at("discovery.type")
                                      .ok_or(Error::MissingField("discovery.type")));

        match discovery_type.as_ref() {
            "constant" => {
                let addresses = try!(self.configuration
                                         .strings_at("discovery.addresses")
                                         .ok_or(Error::MissingField("discovery.addresses")));
                let addresses = try!(resolve::socket_addresses(&addresses));
                let discovery = discovery::Constant::new(addresses);
                info!("loaded constant discovery");
                Ok(Box::new(discovery))
            }
            "multicast" => {
                let interface_address = try!(self.configuration
                                                 .string_at("discovery.interface_address")
                                                 .ok_or(Error::MissingField("discovery.\
                                                                             interface_address")));
                let interface_address = try!(resolve::socket_address(&interface_address));
                let multicast_address = try!(self.configuration
                                                 .string_at("discovery.multicast_address")
                                                 .ok_or(Error::MissingField("discovery.\
                                                                             multicast_address")));
                let multicast_address = try!(resolve::socket_address(&multicast_address));

                let reply_timeout = Duration::milliseconds(self.configuration
                                                               .i64_at("transport.\
                                                                        reply_timeout_ms")
                                                               .unwrap_or(500));

                let discovery = try!(discovery::Multicast::new(interface_address,
                                                               multicast_address,
                                                               public_address,
                                                               reply_timeout));
                info!("loaded multicast discovery");
                Ok(Box::new(discovery))

            }
            _ => {
                Err(Error::InvalidValue("discovery.type",
                                        discovery_type.to_string(),
                                        vec!["constant"]))
            }
        }
    }

    fn load_transport(&self, cipher: Box<Cipher>, metric: Arc<Metric>) -> Result<Box<Transport>> {
        let transport_type = try!(self.configuration
                                      .string_at("transport.type")
                                      .ok_or(Error::MissingField("transport.type")));

        match transport_type.as_ref() {
            "direct" => {
                let local_address = try!(self.configuration
                                             .string_at("transport.local_address")
                                             .ok_or(Error::MissingField("transport.\
                                                                         local_address")));
                let local_address = try!(resolve::socket_address(&local_address));

                let public_address = match self.configuration
                                               .string_at("transport.public_address") {
                    Some(ref value) => Some(try!(resolve::socket_address(value))),
                    None => None,
                };

                let request_timeout = self.configuration
                                          .i64_at("transport.request_timeout_ms")
                                          .map(|value| Duration::milliseconds(value));

                let balancer_type = try!(self.configuration
                                             .string_at("transport.balancer.type")
                                             .ok_or(Error::MissingField("transport.balancer.\
                                                                         type")));

                let balancer_factory = match balancer_type.as_ref() {
                    "dynamic_round_robin" => Box::new(balancer::DynamicRoundRobinFactory::new()),
                    _ => {
                        return Err(Error::InvalidValue("transport.balancer.type",
                                                       balancer_type.to_string(),
                                                       vec!["dynamic_round_robin"]))
                    }
                };

                info!("loaded direct transport - listening at {}", local_address);

                Ok(Box::new(transport::Direct::new(cipher,
                                                   balancer_factory,
                                                   metric,
                                                   local_address,
                                                   public_address,
                                                   request_timeout)))
            }
            _ => {
                Err(Error::InvalidValue("transport.type",
                                        transport_type.to_string(),
                                        vec!["direct"]))
            }
        }
    }

    pub fn load_relays(&self, node: &Arc<Node>) -> Result<Vec<Box<Relay>>> {
        let mut relays = Vec::new();
        if let Some(configurations) = self.configuration.configurations_at("relay") {
            for configuration in configurations {
                relays.push(try!(load_relay(&configuration, node)));
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
        Error::Resolve(error)
    }
}

fn load_relay(configuration: &Configuration, node: &Arc<Node>) -> Result<Box<Relay>> {
    let relay_type = try!(configuration.string_at("type")
                                       .ok_or(Error::MissingField("relay.type")));

    match relay_type.as_ref() {
        "http" => {
            let address = match configuration.string_at("address") {
                Some(address) => Some(try!(resolve::socket_address(&address))),
                None => None,
            };
            let header_field = configuration.string_at("header_field")
                                            .unwrap_or("X-Delix-Service".to_string());
            let read_timeout = configuration.i64_at("read_timeout_ms")
                                            .map(|value| Duration::milliseconds(value));
            let write_timeout = configuration.i64_at("write_timeout_ms")
                                             .map(|value| Duration::milliseconds(value));
            let services_path = configuration.string_at("services_path");

            let api_address = match configuration.string_at("api.address") {
                Some(address) => Some(try!(resolve::socket_address(&address))),
                None => None,
            };

            let http = try!(relay::Http::bind(node.clone(),
                                              address,
                                              api_address,
                                              &header_field,
                                              read_timeout,
                                              write_timeout,
                                              services_path));

            try!(http.load());

            info!("loaded http relay");

            Ok(Box::new(http))
        }
        _ => Err(Error::InvalidValue("relay.type", relay_type.to_string(), vec!["http"])),
    }
}
