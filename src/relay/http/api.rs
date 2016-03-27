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

extern crate hyper;
extern crate rustc_serialize;

use std::io::Read;
use std::net::SocketAddr;
use std::result;
use std::sync::Arc;

use self::hyper::method::Method;
use self::hyper::server::{Listening, Request, Response};
use self::hyper::status::StatusCode;
use self::hyper::uri::RequestUri::AbsolutePath;
use rustc_serialize::json;

use super::logic::{Logic, Service};

pub struct Api {
    #[allow(dead_code)]
    listening: Listening,
}

pub type Result<T> = result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    Hyper(hyper::Error),
}

impl Api {
    pub fn bind(logic: Arc<Logic>, address: SocketAddr) -> Result<Self> {
        let listening = try!(try!(hyper::Server::http(address))
                                 .handle(move |request: Request, response: Response| {
                                     handle(&logic, request, response).unwrap();
                                 }));

        Ok(Api { listening: listening })
    }
}

impl Drop for Api {
    fn drop(&mut self) {
        self.listening.close().unwrap();
    }
}

impl From<hyper::Error> for Error {
    fn from(error: hyper::Error) -> Self {
        Error::Hyper(error)
    }
}

fn handle(logic: &Arc<Logic>, request: Request, mut response: Response) -> Result<()> {
    let (_, method, _, uri, _, mut body) = request.deconstruct();
    match uri {
        AbsolutePath(ref path) => {
            match method {
                Method::Put if path.starts_with("/services/") => {
                    let (_, name) = path.split_at(10);

                    let mut content = String::new();
                    body.read_to_string(&mut content).unwrap();

                    let service = json::decode::<Service>(&content).unwrap();
                    logic.add_service(name, &service.address);

                    *response.status_mut() = StatusCode::Created;
                    response.send(b"").unwrap();
                }
                _ => {
                    *response.status_mut() = StatusCode::NotFound;
                }
            }
        }
        _ => {}
    };

    Ok(())
}
