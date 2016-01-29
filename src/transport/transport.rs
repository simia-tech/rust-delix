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

use node::{ID, request};
use transport::direct;

pub trait Transport : Send + Sync {
    fn bind(&self, ID) -> Result<()>;
    fn join(&self, SocketAddr, ID) -> Result<()>;

    fn register(&self, &str, Box<request::Handler>) -> Result<()>;
    fn deregister(&self, &str) -> Result<()>;
    fn service_count(&self) -> usize;

    fn request(&self,
               &str,
               Box<request::Reader>,
               Box<request::ResponseWriter>)
               -> request::Response;
}

pub type Result<T> = result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    ServiceDoesNotExists,
    Io(io::Error),
    Connection(direct::ConnectionError),
    ConnectionMap(direct::ConnectionMapError),
    ServiceMap(direct::ServiceMapError),
}

impl From<io::Error> for Error {
    fn from(error: io::Error) -> Self {
        Error::Io(error)
    }
}

impl From<direct::ConnectionError> for Error {
    fn from(error: direct::ConnectionError) -> Self {
        Error::Connection(error)
    }
}

impl From<direct::ConnectionMapError> for Error {
    fn from(error: direct::ConnectionMapError) -> Self {
        Error::ConnectionMap(error)
    }
}

impl From<direct::ServiceMapError> for Error {
    fn from(error: direct::ServiceMapError) -> Self {
        Error::ServiceMap(error)
    }
}
