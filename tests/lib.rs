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

mod helper;

use std::thread::sleep_ms;

use delix::node::State;

use helper::{assert_node, build_node};

#[test]
fn discovery_with_two_nodes() {
    let node_one = build_node("127.0.0.1:3001", &[]);
    let node_two = build_node("127.0.0.1:3002", &["127.0.0.1:3001"]);

    sleep_ms(1000);

    assert_node(&node_one, State::Joined, 1);
    assert_node(&node_two, State::Joined, 1);
}

#[test]
fn discovery_with_three_nodes() {
    let node_one = build_node("127.0.0.1:3011", &[]);
    let node_two = build_node("127.0.0.1:3012", &["127.0.0.1:3011"]);
    let node_three = build_node("127.0.0.1:3013", &["127.0.0.1:3011"]);

    sleep_ms(1000);

    assert_node(&node_one, State::Joined, 2);
    assert_node(&node_two, State::Joined, 2);
    assert_node(&node_three, State::Joined, 2);
}

#[test]
fn services_distribution_over_incoming_connection() {
    let mut node_one = build_node("127.0.0.1:3021", &[]);
    node_one.register_service("echo", Box::new(|request| {
        request
    })).unwrap();

    let node_two = build_node("127.0.0.1:3022", &["127.0.0.1:3021"]);

    sleep_ms(1000);
    assert_node(&node_one, State::Joined, 1);
    assert_node(&node_two, State::Joined, 1);

    assert_eq!(1, node_one.service_count());
    assert_eq!(1, node_two.service_count());
}

#[test]
fn services_distribution_over_outgoing_connection() {
    let node_one = build_node("127.0.0.1:3031", &[]);

    let mut node_two = build_node("127.0.0.1:3032", &["127.0.0.1:3031"]);
    node_two.register_service("echo", Box::new(|request| {
        request
    })).unwrap();

    sleep_ms(1000);
    assert_node(&node_one, State::Joined, 1);
    assert_node(&node_two, State::Joined, 1);

    assert_eq!(1, node_one.service_count());
    assert_eq!(1, node_two.service_count());
}

#[test]
fn services_distribution_in_joined_network() {
    let mut node_one = build_node("127.0.0.1:3041", &[]);
    let node_two = build_node("127.0.0.1:3042", &["127.0.0.1:3041"]);

    sleep_ms(1000);
    assert_node(&node_one, State::Joined, 1);
    assert_node(&node_two, State::Joined, 1);

    node_one.register_service("echo", Box::new(|request| {
        request
    })).unwrap();

    sleep_ms(200);
    assert_eq!(1, node_one.service_count());
    assert_eq!(1, node_two.service_count());
}

#[test]
fn services_deregistration() {
    let mut node = build_node("127.0.0.1:3051", &[]);
    node.register_service("echo", Box::new(|request| {
        request
    })).unwrap();
    node.deregister_service("echo").unwrap();

    sleep_ms(100);
    assert_node(&node, State::Discovering, 0);

    assert_eq!(0, node.service_count());
}
