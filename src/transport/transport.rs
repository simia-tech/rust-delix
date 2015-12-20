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

use protobuf::error::ProtobufError;

use node::{ID, ServiceHandler};
use transport::direct::ServiceMapError;

pub trait Transport : Send {
    fn bind(&self, ID) -> Result<()>;
    fn join(&mut self, SocketAddr, ID) -> Result<()>;
    fn connection_count(&self) -> usize;
    fn register_service(&mut self, &str, Box<ServiceHandler>) -> Result<()>;
    fn service_count(&self) -> usize;
}

pub type Result<T> = result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    IO(io::Error),
    ProtobufError(ProtobufError),
    ServiceMapError(ServiceMapError),
}

impl From<io::Error> for Error {
    fn from(error: io::Error) -> Self {
        Error::IO(error)
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
