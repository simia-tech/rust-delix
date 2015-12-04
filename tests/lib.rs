/*
Copyright 2015 The Delix Project Authors. See the AUTHORS file at the top level directory.

Licensed under the Apache License, Version 2.0 (the "License");
you may not use this file except in compliance with the License.
You may obtain a copy of the License at

http://www.apache.org/licenses/LICENSE-2.0

Unless required by applicable law or agreed to in writing, software
distributed under the License is distributed on an "AS IS" BASIS,
WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
See the License for the specific language governing permissions and
limitations under the License.
*/

extern crate delix;

use std::thread::sleep_ms;

use delix::discovery::Constant;
use delix::node::{Node, State};
use delix::transport::Direct;

#[test]
fn discovery_with_two_nodes() {
    let discovery_one = Box::new(Constant::new(&[] as &[&str]));
    let transport_one = Box::new(Direct::new());
    let node_one = Node::new("127.0.0.1:3001", discovery_one, transport_one).unwrap();

    let discovery_two = Box::new(Constant::new(&["127.0.0.1:3001"]));
    let transport_two = Box::new(Direct::new());
    let node_two = Node::new("127.0.0.1:3002", discovery_two, transport_two).unwrap();

    sleep_ms(100);

    assert_eq!(State::Joined, node_one.state());
    assert_eq!(1, node_one.connection_count());

    assert_eq!(State::Joined, node_two.state());
    assert_eq!(1, node_two.connection_count());
}

#[test]
fn discovery_with_three_nodes() {
    let discovery_one = Box::new(Constant::new(&[] as &[&str]));
    let transport_one = Box::new(Direct::new());
    let node_one = Node::new("127.0.0.1:3011", discovery_one, transport_one).unwrap();

    let discovery_two = Box::new(Constant::new(&["127.0.0.1:3011"]));
    let transport_two = Box::new(Direct::new());
    let node_two = Node::new("127.0.0.1:3012", discovery_two, transport_two).unwrap();

    let discovery_three = Box::new(Constant::new(&["127.0.0.1:3011"]));
    let transport_three = Box::new(Direct::new());
    let node_three = Node::new("127.0.0.1:3013", discovery_three, transport_three).unwrap();

    sleep_ms(100);

    assert_eq!(State::Joined, node_one.state());
    assert_eq!(2, node_one.connection_count());

    assert_eq!(State::Joined, node_two.state());
    assert_eq!(2, node_two.connection_count());

    assert_eq!(State::Joined, node_three.state());
    assert_eq!(2, node_three.connection_count());
}
