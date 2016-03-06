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

use openssl::{ssl, x509};

use delix::logger;
use delix::metric::{self, Metric};
use delix::node::{self, Node};
use delix::discovery::{self, Discovery};
use delix::relay::{self, Relay};
use delix::transport::{self, Transport};
use delix::transport::direct::balancer;
use configuration::Configuration;

const DEFAULT_KEY_LENGTH: u32 = 2048;

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
    Relay(relay::Error),
    Resolve(io::Error),
    Ssl(ssl::error::SslError),
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
        let discovery = try!(self.load_discovery());

        let transport = try!(self.load_transport(metric.clone()));

        Ok(Arc::new(try!(Node::new(discovery, transport, metric.clone()))))
    }

    fn load_discovery(&self) -> Result<Box<Discovery>> {
        let discovery_type = try!(self.configuration
                                      .string_at("discovery.type")
                                      .ok_or(Error::MissingField("discovery.type")));

        match discovery_type.as_ref() {
            "constant" => {
                let addresses = try!(self.configuration
                                         .strings_at("discovery.addresses")
                                         .ok_or(Error::MissingField("discovery.addresses")));
                let addresses = try!(resolve_socket_addresses(&addresses));
                let constant = discovery::Constant::new(addresses);
                info!("loaded constant discovery");
                Ok(Box::new(constant))
            }
            _ => {
                Err(Error::InvalidValue("discovery.type",
                                        discovery_type.to_string(),
                                        vec!["constant"]))
            }
        }
    }

    fn load_transport(&self, metric: Arc<Metric>) -> Result<Box<Transport>> {
        let transport_type = try!(self.configuration
                                      .string_at("transport.type")
                                      .ok_or(Error::MissingField("transport.type")));

        match transport_type.as_ref() {
            "direct" => {
                let local_address = try!(self.configuration
                                             .string_at("transport.local_address")
                                             .ok_or(Error::MissingField("transport.\
                                                                         local_address")));
                let local_address = try!(resolve_socket_address(&local_address));

                let public_address = match self.configuration
                                               .string_at("transport.public_address") {
                    Some(ref value) => Some(try!(resolve_socket_address(value))),
                    None => None,
                };

                let ca_file_name = self.configuration.string_at("transport.ca_file");
                let cert_file_name = self.configuration.string_at("transport.cert_file");
                let key_file_name = self.configuration.string_at("transport.key_file");

                let mut ssl_context = try!(ssl::SslContext::new(ssl::SslMethod::Tlsv1_2));
                if let (Some(ca_file_name),
                        Some(cert_file_name),
                        Some(key_file_name)) = (ca_file_name, cert_file_name, key_file_name) {

                    try!(ssl_context.set_CA_file(&ca_file_name));
                    try!(ssl_context.set_certificate_file(&cert_file_name,
                                                          x509::X509FileType::PEM));
                    try!(ssl_context.set_private_key_file(&key_file_name, x509::X509FileType::PEM));
                    ssl_context.set_verify(ssl::SSL_VERIFY_PEER, None);
                    try!(ssl_context.check_private_key());
                } else {
                    info!("generating default certificate with a key length of {} ...",
                          DEFAULT_KEY_LENGTH);
                    let generator = x509::X509Generator::new().set_bitlength(DEFAULT_KEY_LENGTH);
                    let (certificate, private_key) = generator.generate().unwrap();

                    try!(ssl_context.set_certificate(&certificate));
                    try!(ssl_context.set_private_key(&private_key));
                }

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

                Ok(Box::new(transport::Direct::new(ssl_context,
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

impl From<ssl::error::SslError> for Error {
    fn from(error: ssl::error::SslError) -> Self {
        Error::Ssl(error)
    }
}

fn load_relay(configuration: &Configuration, node: &Arc<Node>) -> Result<Box<Relay>> {
    let relay_type = try!(configuration.string_at("type")
                                       .ok_or(Error::MissingField("relay.type")));

    match relay_type.as_ref() {
        "http_static" => {
            let address = configuration.string_at("address");
            let header_field = configuration.string_at("header_field")
                                            .unwrap_or("X-Delix-Service".to_string());
            let read_timeout = configuration.i64_at("read_timeout_ms")
                                            .map(|value| Duration::milliseconds(value));
            let write_timeout = configuration.i64_at("write_timeout_ms")
                                             .map(|value| Duration::milliseconds(value));

            let http_static = relay::HttpStatic::new(node.clone(),
                                                     &header_field,
                                                     read_timeout,
                                                     write_timeout);

            if let Some(configurations) = configuration.configurations_at("service") {
                for configuration in configurations {
                    let name = try!(configuration.string_at("name")
                                                 .ok_or(Error::MissingField("relay.service.name")));
                    let address = try!(configuration.string_at("address")
                                                    .ok_or(Error::MissingField("relay.service.\
                                                                                address")));
                    http_static.add_service(&name, &address);
                }
            }

            if let Some(ref address) = address {
                try!(http_static.bind(try!(resolve_socket_address(address))));
                info!("loaded http static relay - listening at {}", address);
            } else {
                info!("loaded http static relay");
            }

            Ok(Box::new(http_static))
        }
        _ => Err(Error::InvalidValue("relay.type", relay_type.to_string(), vec!["http_static"])),
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
