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

extern crate delix;
extern crate hyper;

mod helper;

use std::io::{self, Read};
use std::thread;

use hyper::client::Client;
use hyper::server::{self, Server};

use delix::node::State;

use helper::{assert_node, build_node, build_http_static_relay};

#[test]
fn static_http_with_nodes() {
    let mut listening = Server::http("127.0.0.1:5000").unwrap().handle(|mut request: server::Request, response: server::Response| {
        io::copy(&mut request, &mut response.start().unwrap()).unwrap();
    }).unwrap();

    let node_one = build_node("127.0.0.1:3001", &[]);
    let relay_one = build_http_static_relay(&node_one, "127.0.0.1:4001");
    relay_one.add_service("echo", "127.0.0.1:5000");

    let node_two = build_node("127.0.0.1:3002", &["127.0.0.1:3001"]);
    build_http_static_relay(&node_two, "127.0.0.1:4002");

    thread::sleep_ms(1000);
    assert_node(&node_one, State::Joined, 1);
    assert_node(&node_two, State::Joined, 1);

    let mut response = Client::new().post("http://127.0.0.1:4002").body("test message").send().unwrap();
    assert_eq!(hyper::Ok, response.status);
    let mut response_body = String::new();
    response.read_to_string(&mut response_body).unwrap();
    assert_eq!("test message", &response_body);

    listening.close().unwrap();
}
