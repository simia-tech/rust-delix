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

use std::io::{self, Read, Write};
use std::net::{self, SocketAddr};
use std::sync::{Arc, RwLock};
use std::thread;
use time::Duration;

use node::{Node, request, service};
use relay::{Relay, Result};
use util::reader;
use util::time::to_std_duration;

pub struct Http {
    node: Arc<Node>,
    header_field: String,
    read_timeout: Option<Duration>,
    write_timeout: Option<Duration>,
    join_handle: RwLock<Option<(thread::JoinHandle<()>, SocketAddr)>>,
    running: Arc<RwLock<bool>>,
}

enum StatusCode {
    InternalServerError,
    BadGateway,
    ServiceUnavailable,
}

impl Http {
    pub fn new(node: Arc<Node>,
               header_field: &str,
               read_timeout: Option<Duration>,
               write_timeout: Option<Duration>)
               -> Self {
        Http {
            node: node,
            header_field: header_field.to_string(),
            read_timeout: read_timeout,
            write_timeout: write_timeout,
            join_handle: RwLock::new(None),
            running: Arc::new(RwLock::new(false)),
        }
    }

    pub fn add_service(&self, name: &str, address: &str) {
        let name_clone = name.to_string();
        let address_clone = address.to_string();
        let read_timeout = self.read_timeout.map(|value| to_std_duration(value));
        let write_timeout = self.write_timeout.map(|value| to_std_duration(value));
        self.node
            .register(name,
                      Box::new(move |mut request| {
                          let mut stream = try!(net::TcpStream::connect(&*address_clone));
                          try!(stream.set_read_timeout(read_timeout));
                          try!(stream.set_write_timeout(write_timeout));

                          try!(io::copy(&mut request, &mut stream));
                          debug!("handled request to {}", name_clone);

                          Ok(Box::new(reader::Http::new(stream)))
                      }))
            .unwrap();
    }
}

impl Relay for Http {
    fn bind(&self, address: SocketAddr) -> Result<()> {
        let tcp_listener = try!(net::TcpListener::bind(address));

        *self.running.write().unwrap() = true;

        let node_clone = self.node.clone();
        let running_clone = self.running.clone();
        let header_field = self.header_field.to_lowercase().trim().to_string();
        *self.join_handle.write().unwrap() = Some((thread::spawn(move || {
            for stream in tcp_listener.incoming() {
                if !*running_clone.read().unwrap() {
                    break;
                }

                let node_clone = node_clone.clone();
                let header_field = header_field.clone();

                thread::spawn(move || {
                    if let Err(error) = handle_connection(stream.unwrap(),
                                                          node_clone,
                                                          &header_field) {
                        error!("http error: {:?}", error);
                    }
                });
            }
        }),
                                                   address));

        Ok(())
    }

    fn unbind(&self) -> Result<()> {
        *self.running.write().unwrap() = false;
        if let Some((join_handle, address)) = self.join_handle.write().unwrap().take() {
            // connect to local address to enable the thread to escape the accept loop.
            try!(net::TcpStream::connect(address));
            join_handle.join().unwrap();
        }
        Ok(())
    }
}

impl Drop for Http {
    fn drop(&mut self) {
        self.unbind().unwrap();
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

fn handle_connection(mut stream: net::TcpStream,
                     node: Arc<Node>,
                     header_field: &str)
                     -> io::Result<()> {
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
