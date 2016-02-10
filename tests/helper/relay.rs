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

use std::net::ToSocketAddrs;
use std::sync::Arc;

use delix::node::Node;
use delix::relay::{self, Relay};

pub fn build_http_static_relay(node: &Arc<Node>, address: Option<&str>) -> Arc<relay::HttpStatic> {
    let relay = relay::HttpStatic::new(node.clone(), "X-Delix-Service");
    if let Some(address) = address {
        relay.bind(address.to_socket_addrs().unwrap().next().unwrap()).unwrap();
    }
    Arc::new(relay)
}
