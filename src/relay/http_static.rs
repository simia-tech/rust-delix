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
use std::sync::{Arc, RwLock, atomic};
use std::thread;

use node::{Node, request};
use relay::{Relay, Result};
use util::reader;

pub struct HttpStatic {
    node: Arc<Node>,
    header_field: String,
    join_handle: RwLock<Option<(thread::JoinHandle<()>, SocketAddr)>>,
    running: Arc<atomic::AtomicBool>,
}

enum StatusCode {
    InternalServerError,
    BadGateway,
    ServiceUnavailable,
}

impl HttpStatic {
    pub fn new(node: Arc<Node>, header_field: &str) -> HttpStatic {
        HttpStatic {
            node: node,
            header_field: header_field.to_string(),
            join_handle: RwLock::new(None),
            running: Arc::new(atomic::AtomicBool::new(false)),
        }
    }

    pub fn add_service(&self, name: &str, address: SocketAddr) {
        let name_clone = name.to_string();
        self.node
            .register(name,
                      Box::new(move |mut request| {
                          let mut stream = try!(net::TcpStream::connect(address));
                          io::copy(&mut request, &mut stream).unwrap();

                          debug!("handled request to {}", name_clone);

                          Ok(Box::new(reader::Http::new(stream)))
                      }))
            .unwrap();
    }
}

impl Relay for HttpStatic {
    fn bind(&self, address: SocketAddr) -> Result<()> {
        let tcp_listener = try!(net::TcpListener::bind(address));

        let node_clone = self.node.clone();
        let running_clone = self.running.clone();
        let header_field = self.header_field.to_lowercase().trim().to_string();
        *self.join_handle.write().unwrap() = Some((thread::spawn(move || {
            running_clone.store(true, atomic::Ordering::SeqCst);
            for stream in tcp_listener.incoming() {
                if !running_clone.load(atomic::Ordering::SeqCst) {
                    break;
                }

                let mut stream = stream.unwrap();

                let mut http_reader = reader::Http::new(stream.try_clone().unwrap());
                let mut service_name = String::new();
                http_reader.read_header(|name, value| {
                               if name == header_field {
                                   service_name = value.to_string();
                               }
                           })
                           .unwrap();

                let response = node_clone.request(&service_name,
                                                  Box::new(http_reader),
                                                  Box::new(stream.try_clone()
                                                                 .unwrap()));

                let response = match response {
                    Ok(_) => Vec::new(),
                    Err(request::Error::ServiceDoesNotExists) => {
                        build_text_response(StatusCode::BadGateway,
                                            &format!("service [{}] not found", service_name))
                    }
                    Err(request::Error::ServiceUnavailable) => {
                        build_text_response(StatusCode::ServiceUnavailable,
                                            &format!("service [{}] is unavailable", service_name))
                    }
                    Err(error) => {
                        build_text_response(StatusCode::InternalServerError,
                                            &format!("error [{:?}]", error))
                    }
                };
                stream.write_all(&response).unwrap();
                stream.flush().unwrap();
            }
        }),
                                                   address));

        Ok(())
    }

    fn unbind(&self) -> Result<()> {
        self.running.store(false, atomic::Ordering::SeqCst);
        if let Some((join_handle, address)) = self.join_handle.write().unwrap().take() {
            // connect to local address to enable the thread to escape the accept loop.
            try!(net::TcpStream::connect(address));
            join_handle.join().unwrap();
        }
        Ok(())
    }
}

impl Drop for HttpStatic {
    fn drop(&mut self) {
        self.unbind().unwrap();
    }
}

impl From<io::Error> for request::Error {
    fn from(error: io::Error) -> Self {
        match error.kind() {
            io::ErrorKind::ConnectionRefused => request::Error::ServiceUnavailable,
            _ => request::Error::Internal(format!("{:?}", error.kind())),
        }
    }
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
