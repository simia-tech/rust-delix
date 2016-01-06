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

#[allow(dead_code)] mod helper;

use std::io::{self, Read};
use std::net::SocketAddr;

use hyper::client::Client;
use hyper::server::{self, Server};
use hyper::status::StatusCode;

use helper::{assert_response, build_node, build_http_static_relay};

header! { (XDelixService, "X-Delix-Service") => [String] }

#[test]
fn http_static_with_sized_response() {
    let mut listening = Server::http("127.0.0.1:5000").unwrap().handle(|mut request: server::Request, response: server::Response| {
        let mut body = Vec::new();
        request.read_to_end(&mut body).unwrap();
        response.send(&body).unwrap();
    }).unwrap();

    let node = build_node("127.0.0.1:3001", &[]);
    let relay = build_http_static_relay(&node, Some("127.0.0.1:4000"));
    relay.add_service("echo", "127.0.0.1:5000".parse::<SocketAddr>().unwrap());

    let mut response = Client::new().post("http://127.0.0.1:4000").header(XDelixService("echo".to_owned())).body("test message").send().unwrap();
    assert_response(StatusCode::Ok, b"test message", &mut response);

    listening.close().unwrap();
}

#[test]
fn http_static_with_chunked_response() {
    let mut listening = Server::http("127.0.0.1:5010").unwrap().handle(|mut request: server::Request, response: server::Response| {
        io::copy(&mut request, &mut response.start().unwrap()).unwrap();
    }).unwrap();

    let node = build_node("127.0.0.1:3011", &[]);
    let relay = build_http_static_relay(&node, Some("127.0.0.1:4010"));
    relay.add_service("echo", "127.0.0.1:5010".parse::<SocketAddr>().unwrap());

    let mut response = Client::new().post("http://127.0.0.1:4010").header(XDelixService("echo".to_owned())).body("test message").send().unwrap();
    assert_response(StatusCode::Ok, b"test message", &mut response);

    listening.close().unwrap();
}

#[test]
fn http_static_with_missing_service() {
    let node = build_node("127.0.0.1:3021", &[]);
    let relay = build_http_static_relay(&node, Some("127.0.0.1:4020"));

    let mut response = Client::new().post("http://127.0.0.1:4020").header(XDelixService("echo".to_owned())).body("test message").send().unwrap();
    assert_response(StatusCode::BadGateway, b"service [echo] not found", &mut response);

    drop(relay);
}

#[test]
fn http_static_with_unreachable_service() {
    let node = build_node("127.0.0.1:3031", &[]);
    let relay = build_http_static_relay(&node, Some("127.0.0.1:4030"));
    relay.add_service("echo", "127.0.0.1:5030".parse::<SocketAddr>().unwrap());

    let mut response = Client::new().post("http://127.0.0.1:4030").header(XDelixService("echo".to_owned())).body("test message").send().unwrap();
    assert_response(StatusCode::ServiceUnavailable, b"service [echo] is unavailable", &mut response);
}
