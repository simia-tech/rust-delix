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

use std::net::{AddrParseError, SocketAddr};
use std::io;
use std::result;

use protobuf::error::ProtobufError;

use node::{ID, IDError, ServiceHandler};
use transport::direct::ServiceMapError;

pub trait Transport : Send {
    fn bind(&self, ID) -> Result<()>;
    fn join(&mut self, SocketAddr, ID) -> Result<()>;
    fn connection_count(&self) -> usize;

    fn register(&mut self, &str, Box<ServiceHandler>) -> Result<()>;
    fn deregister(&mut self, &str) -> Result<()>;
    fn service_count(&self) -> usize;

    fn request(&mut self, &str, &[u8]) -> Result<Vec<u8>>;
}

pub type Result<T> = result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    ServiceDoesNotExists,
    IDError(IDError),
    AddrParseError(AddrParseError),
    IOError(io::Error),
    ProtobufError(ProtobufError),
    ServiceMapError(ServiceMapError),
}

impl From<IDError> for Error {
    fn from(error: IDError) -> Self {
        Error::IDError(error)
    }
}

impl From<AddrParseError> for Error {
    fn from(error: AddrParseError) -> Self {
        Error::AddrParseError(error)
    }
}

impl From<io::Error> for Error {
    fn from(error: io::Error) -> Self {
        Error::IOError(error)
    }
}

impl From<ProtobufError> for Error {
    fn from(error: ProtobufError) -> Self {
        Error::ProtobufError(error)
    }
}

impl From<ServiceMapError> for Error {
    fn from(error: ServiceMapError) -> Self {
        Error::ServiceMapError(error)
    }
}
