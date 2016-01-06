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

#[allow(dead_code)] mod helper;

use std::thread::sleep_ms;

use delix::node::State;
use delix::node::request;

use helper::{assert_node, build_node};

#[test]
fn distribution_over_incoming_connection() {
    let node_one = build_node("127.0.0.1:3001", &[]);
    node_one.register("echo", Box::new(|request| Ok(request.to_vec())))
            .unwrap();

    let node_two = build_node("127.0.0.1:3002", &["127.0.0.1:3001"]);

    sleep_ms(1000);
    assert_node(&node_one, State::Joined, 1);
    assert_node(&node_two, State::Joined, 1);

    assert_eq!(1, node_one.service_count());
    assert_eq!(1, node_two.service_count());
}

#[test]
fn distribution_over_outgoing_connection() {
    let node_one = build_node("127.0.0.1:3011", &[]);

    let node_two = build_node("127.0.0.1:3012", &["127.0.0.1:3011"]);
    node_two.register("echo", Box::new(|request| Ok(request.to_vec())))
            .unwrap();

    sleep_ms(1000);
    assert_node(&node_one, State::Joined, 1);
    assert_node(&node_two, State::Joined, 1);

    assert_eq!(1, node_one.service_count());
    assert_eq!(1, node_two.service_count());
}

#[test]
fn distribution_in_joined_network() {
    let node_one = build_node("127.0.0.1:3021", &[]);
    let node_two = build_node("127.0.0.1:3022", &["127.0.0.1:3021"]);

    sleep_ms(1000);
    assert_node(&node_one, State::Joined, 1);
    assert_node(&node_two, State::Joined, 1);

    node_one.register("echo", Box::new(|request| Ok(request.to_vec())))
            .unwrap();

    sleep_ms(200);
    assert_eq!(1, node_one.service_count());
    assert_eq!(1, node_two.service_count());
}

#[test]
fn deregistration() {
    let node = build_node("127.0.0.1:3031", &[]);
    node.register("echo", Box::new(|request| Ok(request.to_vec())))
        .unwrap();
    node.deregister("echo").unwrap();

    sleep_ms(100);
    assert_node(&node, State::Discovering, 0);

    assert_eq!(0, node.service_count());
}

#[test]
fn deregistration_in_joined_network() {
    let node_one = build_node("127.0.0.1:3041", &[]);
    node_one.register("echo", Box::new(|request| Ok(request.to_vec())))
            .unwrap();

    let node_two = build_node("127.0.0.1:3042", &["127.0.0.1:3041"]);

    sleep_ms(1000);
    assert_node(&node_one, State::Joined, 1);
    assert_node(&node_two, State::Joined, 1);

    assert_eq!(1, node_one.service_count());
    assert_eq!(1, node_two.service_count());

    node_one.deregister("echo").unwrap();

    assert_eq!(0, node_one.service_count());
    assert_eq!(1, node_two.service_count());

    assert_eq!(Err(request::Error::ServiceDoesNotExists), node_two.request("echo", b"test"));

    assert_eq!(0, node_one.service_count());
    assert_eq!(0, node_two.service_count());
}
