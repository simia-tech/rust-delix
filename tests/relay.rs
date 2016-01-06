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
#[macro_use] extern crate hyper;

mod helper;

use std::io::{self, Read};
use std::net::SocketAddr;
use std::thread;

use hyper::client::Client;
use hyper::server::{self, Server};

use delix::node::State;
use delix::relay::Relay;

use helper::{assert_node, build_node, build_http_static_relay};

header! { (XDelixService, "X-Delix-Service") => [String] }

#[test]
fn http_static_with_sized_response() {
    let mut listening = Server::http("127.0.0.1:5000").unwrap().handle(|mut request: server::Request, response: server::Response| {
        let mut body = Vec::new();
        request.read_to_end(&mut body).unwrap();
        response.send(&body).unwrap();
    }).unwrap();

    let node_one = build_node("127.0.0.1:3001", &[]);
    let relay_one = build_http_static_relay(&node_one, None);
    relay_one.add_service("echo", "127.0.0.1:5000".parse::<SocketAddr>().unwrap());

    let node_two = build_node("127.0.0.1:3002", &["127.0.0.1:3001"]);
    let relay_two = build_http_static_relay(&node_two, Some("127.0.0.1:4000"));

    thread::sleep_ms(1000);
    assert_node(&node_one, State::Joined, 1);
    assert_node(&node_two, State::Joined, 1);

    let mut response = Client::new().post("http://127.0.0.1:4000").header(XDelixService("echo".to_owned())).body("test message").send().unwrap();
    assert_eq!(hyper::Ok, response.status);
    let mut response_body = String::new();
    response.read_to_string(&mut response_body).unwrap();
    assert_eq!("test message", &response_body);

    relay_two.unbind().unwrap();
    listening.close().unwrap();
}

#[test]
fn http_static_with_chunked_response() {
    let mut listening = Server::http("127.0.0.1:5010").unwrap().handle(|mut request: server::Request, response: server::Response| {
        io::copy(&mut request, &mut response.start().unwrap()).unwrap();
    }).unwrap();

    let node_one = build_node("127.0.0.1:3011", &[]);
    let relay_one = build_http_static_relay(&node_one, None);
    relay_one.add_service("echo", "127.0.0.1:5010".parse::<SocketAddr>().unwrap());

    let node_two = build_node("127.0.0.1:3012", &["127.0.0.1:3011"]);
    let relay_two = build_http_static_relay(&node_two, Some("127.0.0.1:4010"));

    thread::sleep_ms(1000);
    assert_node(&node_one, State::Joined, 1);
    assert_node(&node_two, State::Joined, 1);

    let mut response = Client::new().post("http://127.0.0.1:4010").header(XDelixService("echo".to_owned())).body("test message").send().unwrap();
    assert_eq!(hyper::Ok, response.status);
    let mut response_body = String::new();
    response.read_to_string(&mut response_body).unwrap();
    assert_eq!("test message", &response_body);

    relay_two.unbind().unwrap();
    listening.close().unwrap();
}
