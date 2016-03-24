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

use std::io;
use std::net::{SocketAddr, ToSocketAddrs};

pub fn socket_address(address: &str) -> io::Result<SocketAddr> {
    Ok(try!(try!(address.to_socket_addrs())
                .next()
                .ok_or(io::Error::new(io::ErrorKind::Other,
                                      format!("could not resolve address [{}]", address)))))
}

pub fn socket_addresses(addresses: &[String]) -> io::Result<Vec<SocketAddr>> {
    let mut result = Vec::new();
    for address in addresses {
        result.append(&mut try!(address.to_socket_addrs()).collect::<Vec<SocketAddr>>());
    }
    Ok(result)
}
