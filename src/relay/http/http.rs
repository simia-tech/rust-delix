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

use std::io::{self, Write};
use std::net::{self, SocketAddr};
use std::sync::Arc;

use time::Duration;

use node::{Node, request, service};
use util::net::TcpServer;
use util::reader;
use util::time::to_std_duration;
use super::api::Api;
use super::logic::Logic;
use super::super::{Relay, Result};

pub struct Http {
    logic: Arc<Logic>,

    #[allow(dead_code)]
    server: Option<TcpServer>,

    #[allow(dead_code)]
    api: Option<Api>,
}

enum StatusCode {
    InternalServerError,
    BadGateway,
    ServiceUnavailable,
}

impl Http {
    pub fn bind(node: Arc<Node>,
                address: Option<SocketAddr>,
                api_address: Option<SocketAddr>,
                header_field: &str,
                read_timeout: Option<Duration>,
                write_timeout: Option<Duration>,
                services_path: Option<String>)
                -> Result<Self> {

        let logic = Arc::new(Logic::new(node.clone(), services_path));

        let server = if let Some(address) = address {
            let node = node.clone();
            let header_field = header_field.to_string();
            Some(try!(TcpServer::bind(address, move |mut stream| {
                stream.set_read_timeout(read_timeout.map(|value| to_std_duration(value))).unwrap();
                stream.set_write_timeout(write_timeout.map(|value| to_std_duration(value)))
                      .unwrap();

                let node = node.clone();
                let header_field = header_field.clone();
                Box::new(move || {
                    if let Err(error) = handle_connection(&mut stream, &node, &header_field) {
                        error!("http error: {:?}", error);
                    }
                })
            })))
        } else {
            None
        };

        let api = if let Some(api_address) = api_address {
            Some(Api::bind(logic.clone(), api_address).unwrap())
        } else {
            None
        };

        Ok(Http {
            logic: logic,
            server: server,
            api: api,
        })
    }

    pub fn add_service(&self, name: &str, address: &str) {
        self.logic.add_service(name, address);
    }
}

impl Relay for Http {
    fn load(&self) -> Result<()> {
        try!(self.logic.load_services());
        Ok(())
    }
}

impl From<io::Error> for service::Error {
    fn from(error: io::Error) -> Self {
        match error.kind() {
            io::ErrorKind::ConnectionRefused => service::Error::Unavailable,
            _ => service::Error::Internal(format!("{:?}", error.kind())),
        }
    }
}

fn handle_connection(stream: &mut net::TcpStream,
                     node: &Arc<Node>,
                     header_field: &str)
                     -> io::Result<()> {
    let header_field = header_field.to_lowercase();
    let mut http_reader = reader::Http::new(stream.try_clone().unwrap());
    let mut service_name = String::new();
    try!(http_reader.read_header(|name, value| {
        if name == header_field {
            service_name = value.to_string();
        }
    }));

    let mut stream_clone = stream.try_clone().unwrap();
    let response_handler = move |mut reader| {
        if let Err(e) = io::copy(&mut reader, &mut stream_clone) {
            error!("response error: {:?}", e);
        }
    };

    let result = node.request(&service_name,
                              Box::new(http_reader),
                              Box::new(response_handler));

    let response = match result {
        Ok(_) => Vec::new(),
        Err(request::Error::NoService) => {
            build_text_response(StatusCode::BadGateway,
                                &format!("service [{}] not found", service_name))
        }
        Err(request::Error::Service(service::Error::Unavailable)) => {
            build_text_response(StatusCode::ServiceUnavailable,
                                &format!("service [{}] is unavailable", service_name))
        }
        Err(error) => {
            build_text_response(StatusCode::InternalServerError,
                                &format!("error [{:?}]", error))
        }
    };
    try!(stream.write_all(&response));
    Ok(())
}

fn build_text_response(status_code: StatusCode, message: &str) -> Vec<u8> {
    match status_code {
        StatusCode::InternalServerError => {
            format!("HTTP/1.1 500 Internal Server Error\r\n\r\n{}", message).into_bytes()
        }
        StatusCode::BadGateway => {
            format!("HTTP/1.1 502 Bad Gateway\r\n\r\n{}", message).into_bytes()
        }
        StatusCode::ServiceUnavailable => {
            format!("HTTP/1.1 503 Service Unavailable\r\n\r\n{}", message).into_bytes()
        }
    }
}
