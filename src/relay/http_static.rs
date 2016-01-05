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
use std::sync::{Arc, RwLock, atomic};
use std::thread;

use node::Node;
use relay::{Relay, Result};

pub struct HttpStatic {
    node: Arc<Node>,
    join_handle: RwLock<Option<(thread::JoinHandle<()>, SocketAddr)>>,
    running: Arc<atomic::AtomicBool>,
}

impl HttpStatic {
    pub fn new(node: Arc<Node>) -> HttpStatic {
        HttpStatic {
            node: node,
            join_handle: RwLock::new(None),
            running: Arc::new(atomic::AtomicBool::new(false)),
        }
    }

    pub fn add_service(&self, name: &str, address: SocketAddr) {
        let name_clone = name.to_string();
        self.node
            .register(name,
                      Box::new(move |request| {
                          let mut stream = net::TcpStream::connect(address).unwrap();
                          stream.write_all(request).unwrap();

                          let response = read_header_and_body(&stream, |_, _| {});
                          info!("handled request to {}", name_clone);
                          Ok(response)
                      }))
            .unwrap();
    }
}

impl Relay for HttpStatic {
    fn bind(&self, address: SocketAddr) -> Result<()> {
        let tcp_listener = try!(net::TcpListener::bind(address));

        let node_clone = self.node.clone();
        let running_clone = self.running.clone();
        *self.join_handle.write().unwrap() = Some((thread::spawn(move || {
            running_clone.store(true, atomic::Ordering::SeqCst);
            for stream in tcp_listener.incoming() {
                if !running_clone.load(atomic::Ordering::SeqCst) {
                    break;
                }

                let mut stream = stream.unwrap();

                let mut name = String::new();
                let request = read_header_and_body(&stream, |key, value| {
                    if key == "x-delix-service" {
                        name = value.to_string();
                    }
                });

                let response = match node_clone.request(&name, &request) {
                    Ok(response) => response,
                    Err(error) => {
                        println!("request error: {:?}", error);
                        return;
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

fn read_header_and_body<R: Read, F: FnMut(&str, &str)>(r: R, mut f: F) -> Vec<u8> {
    let mut header = Vec::new();
    let mut content_length = 0;
    let mut reader = io::BufReader::new(r);
    loop {
        let mut line = String::new();
        reader.read_line(&mut line).unwrap();

        if line.trim().len() == 0 {
            header.append(&mut line.into_bytes());
            break;
        }

        {
            let parts = line.split(':').collect::<Vec<_>>();
            if parts.len() == 2 {
                let key = parts[0].to_lowercase().trim().to_string();
                let value = parts[1].to_string().trim().to_string();

                if key == "content-length" {
                    content_length = value.parse::<usize>().unwrap();
                }

                f(&key, &value);
            }
        }

        header.append(&mut line.into_bytes());
    }

    if content_length > 0 {
        let mut body = Vec::with_capacity(content_length);
        unsafe {
            body.set_len(content_length);
        }
        reader.read(&mut body).unwrap();

        let mut request = Vec::with_capacity(header.len() + body.len());
        request.append(&mut header);
        request.append(&mut body);
        request
    } else {
        header
    }
}
