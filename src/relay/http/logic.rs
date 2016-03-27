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

extern crate rustc_serialize;

use std::fs;
use std::io::{self, Read};
use std::net;
use std::path::Path;
use std::sync::Arc;

use rustc_serialize::json;

use node::Node;
use util::reader;

pub struct Logic {
    node: Arc<Node>,
    services_path: Option<String>,
}

#[derive(Debug, RustcDecodable, RustcEncodable)]
pub struct Service {
    pub address: String,
}

impl Logic {
    pub fn new(node: Arc<Node>, services_path: Option<String>) -> Self {
        Logic {
            node: node,
            services_path: services_path,
        }
    }

    pub fn load_services(&self) -> io::Result<()> {
        if let Some(ref services_path) = self.services_path {
            let services_path = Path::new(services_path);
            for entry in try!(fs::read_dir(services_path)) {
                let entry = try!(entry);
                if let Some(name) = entry.path().file_stem().and_then(|name| name.to_str()) {
                    let mut file = try!(fs::File::open(entry.path()));
                    let mut content = String::new();
                    try!(file.read_to_string(&mut content));

                    let service = json::decode::<Service>(&content).unwrap();

                    self.add_service(name, &service.address)
                }
            }
        }
        Ok(())
    }

    pub fn add_service(&self, name: &str, address: &str) {
        let name_clone = name.to_string();
        let address_clone = address.to_string();
        self.node
            .register(name,
                      Box::new(move |mut request| {
                          let mut stream = try!(net::TcpStream::connect(&*address_clone));

                          try!(io::copy(&mut request, &mut stream));
                          debug!("handled request to {}", name_clone);

                          Ok(Box::new(reader::Http::new(stream)))
                      }))
            .unwrap();
    }
}
