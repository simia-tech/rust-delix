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

use std::io::{self, BufRead, Read, Write};
use std::net::{self, SocketAddr};
use std::sync::{Arc, RwLock};

use hyper::{header, method, server, version};
use protobuf::{self, Message};

use message;
use node::Node;
use relay::{Relay, Result};

pub struct HttpStatic {
    node: Arc<Node>,
    listening: RwLock<Option<server::Listening>>,
}

impl HttpStatic {
    pub fn new(node: Arc<Node>) -> HttpStatic {
        HttpStatic {
            node: node,
            listening: RwLock::new(None),
        }
    }

    pub fn add_service(&self, name: &str, address: SocketAddr) {
        let name_clone = name.to_string();
        self.node
            .register(name,
                      Box::new(move |encoded_request| {
                          println!("got request");

                          let request = decode_request(encoded_request);

                          println!("request: {:?}", request);

                          Ok(Vec::new())
                      }))
            .unwrap();
    }
}

impl Relay for HttpStatic {
    fn bind(&self, address: SocketAddr) -> Result<()> {
        let node_clone = self.node.clone();
        let handler = move |mut request: server::Request, response: server::Response| {
            // let name = match request.headers.get_raw("x-delix-service") {
            //    Some(values) => String::from_utf8_lossy(&values[0]),
            //    None => panic!("did not found address header"),
            // };
            let encoded_request = encode_request(&mut request);
            let encoded_response = node_clone.request("echo", &encoded_request);
            println!("got response {:?}", encoded_response);

            response.send(b"test message").unwrap();
        };

        *self.listening.write().unwrap() = Some(server::Server::http(address)
                                                    .unwrap()
                                                    .handle(handler)
                                                    .unwrap());

        Ok(())
    }

    fn unbind(&self) -> Result<()> {
        if let Some(mut listening) = self.listening.write().unwrap().take() {
            listening.close().unwrap();
        }
        Ok(())
    }
}

impl Drop for HttpStatic {
    fn drop(&mut self) {
        self.unbind().unwrap();
    }
}

fn encode_request(request: &mut server::Request) -> Vec<u8> {
    let mut http_request = message::HttpRequest::new();

    http_request.set_method(match request.method {
        method::Method::Options => message::HttpRequest_Method::OPTIONS,
        method::Method::Get => message::HttpRequest_Method::GET,
        method::Method::Post => message::HttpRequest_Method::POST,
        method::Method::Put => message::HttpRequest_Method::PUT,
        method::Method::Delete => message::HttpRequest_Method::DELETE,
        method::Method::Head => message::HttpRequest_Method::HEAD,
        method::Method::Trace => message::HttpRequest_Method::TRACE,
        method::Method::Connect => message::HttpRequest_Method::CONNECT,
        method::Method::Patch => message::HttpRequest_Method::PATCH,
        method::Method::Extension(_) => unimplemented!(),
    });

    http_request.set_path(format!("{}", request.uri));

    http_request.set_version(match request.version {
        version::HttpVersion::Http09 => message::HttpRequest_Version::V09,
        version::HttpVersion::Http10 => message::HttpRequest_Version::V10,
        version::HttpVersion::Http11 => message::HttpRequest_Version::V11,
        version::HttpVersion::Http20 => message::HttpRequest_Version::V20,
    });

    for item in request.headers.iter() {
        let mut header = message::HttpRequest_Header::new();
        header.set_name(item.name().to_string());
        header.set_value(item.value_string());
        http_request.mut_headers().push(header);
    }

    let mut body = Vec::new();
    request.read_to_end(&mut body).unwrap();
    http_request.set_body(body);

    http_request.write_to_bytes().unwrap()
}

fn decode_request(encoded_request: &[u8])
                  -> (method::Method,
                      String,
                      version::HttpVersion,
                      header::Headers,
                      Vec<u8>) {
    let mut http_request = protobuf::parse_from_bytes::<message::HttpRequest>(encoded_request)
                               .unwrap();

    let method = match http_request.get_method() {
        message::HttpRequest_Method::OPTIONS => method::Method::Options,
        message::HttpRequest_Method::GET => method::Method::Get,
        message::HttpRequest_Method::POST => method::Method::Post,
        message::HttpRequest_Method::PUT => method::Method::Put,
        message::HttpRequest_Method::DELETE => method::Method::Delete,
        message::HttpRequest_Method::HEAD => method::Method::Head,
        message::HttpRequest_Method::TRACE => method::Method::Trace,
        message::HttpRequest_Method::CONNECT => method::Method::Connect,
        message::HttpRequest_Method::PATCH => method::Method::Patch,
    };

    let version = match http_request.get_version() {
        message::HttpRequest_Version::V09 => version::HttpVersion::Http09,
        message::HttpRequest_Version::V10 => version::HttpVersion::Http10,
        message::HttpRequest_Version::V11 => version::HttpVersion::Http11,
        message::HttpRequest_Version::V20 => version::HttpVersion::Http20,
    };

    let mut headers = header::Headers::new();
    for header in http_request.mut_headers().iter_mut() {
        headers.set_raw(header.take_name(), vec![header.take_value().into_bytes()]);
    }

    (method,
     http_request.take_path(),
     version,
     headers,
     http_request.take_body())
}
