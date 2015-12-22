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
fn single_echo_cycle() {
    let mut node_one = build_node("127.0.0.1:3001", &[]);
    node_one.register("echo", Box::new(|request| request.to_vec()))
            .unwrap();

    let node_two = build_node("127.0.0.1:3002", &["127.0.0.1:3001"]);

    sleep_ms(1000);
    assert_node(&node_one, State::Joined, 1);
    assert_node(&node_two, State::Joined, 1);

    let response = node_two.request("echo", b"test message").unwrap();
    assert_eq!(b"test message".to_vec(), response);
}

#[test]
fn multiple_echos_cycle() {
    let mut node_one = build_node("127.0.0.1:3011", &[]);
    node_one.register("echo", Box::new(|request| request.to_vec()))
            .unwrap();

    let node_two = build_node("127.0.0.1:3012", &["127.0.0.1:3011"]);

    sleep_ms(1000);
    assert_node(&node_one, State::Joined, 1);
    assert_node(&node_two, State::Joined, 1);

    let response_one = node_two.request("echo", b"test message one").unwrap();
    let response_two = node_two.request("echo", b"test message two").unwrap();
    assert_eq!(b"test message one".to_vec(), response_one);
    assert_eq!(b"test message two".to_vec(), response_two);
}