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

use std::thread;

use delix::node::request;

#[test]
fn distribution_over_incoming_connection() {
    helper::set_up();

    let node_one = helper::build_node("127.0.0.1:3001", &[], None);
    node_one.register("echo", Box::new(|request| Ok(request)))
            .unwrap();

    let node_two = helper::build_node("127.0.0.1:3002", &["127.0.0.1:3001"], None);

    helper::wait_for_joined(&[&node_one, &node_two]);

    assert_eq!(1, node_one.service_count());
    assert_eq!(1, node_two.service_count());
}

#[test]
fn distribution_over_outgoing_connection() {
    helper::set_up();

    let node_one = helper::build_node("127.0.0.1:3011", &[], None);

    let node_two = helper::build_node("127.0.0.1:3012", &["127.0.0.1:3011"], None);
    node_two.register("echo", Box::new(|request| Ok(request)))
            .unwrap();

    helper::wait_for_joined(&[&node_one, &node_two]);

    assert_eq!(1, node_one.service_count());
    assert_eq!(1, node_two.service_count());
}

#[test]
fn distribution_in_joined_network() {
    helper::set_up();

    let node_one = helper::build_node("127.0.0.1:3021", &[], None);
    let node_two = helper::build_node("127.0.0.1:3022", &["127.0.0.1:3021"], None);

    helper::wait_for_joined(&[&node_one, &node_two]);

    node_one.register("echo", Box::new(|request| Ok(request)))
            .unwrap();

            thread::sleep(::std::time::Duration::from_millis(200));
    assert_eq!(1, node_one.service_count());
    assert_eq!(1, node_two.service_count());
}

#[test]
fn deregistration() {
    helper::set_up();

    let node = helper::build_node("127.0.0.1:3031", &[], None);
    node.register("echo", Box::new(|request| Ok(request)))
        .unwrap();
    node.deregister("echo").unwrap();

    thread::sleep(::std::time::Duration::from_millis(100));

    assert_eq!(0, node.service_count());
}

#[test]
fn deregistration_in_joined_network() {
    helper::set_up();

    let node_one = helper::build_node("127.0.0.1:3041", &[], None);
    node_one.register("echo", Box::new(|request| Ok(request)))
            .unwrap();

    let node_two = helper::build_node("127.0.0.1:3042", &["127.0.0.1:3041"], None);

    helper::wait_for_joined(&[&node_one, &node_two]);

    assert_eq!(1, node_one.service_count());
    assert_eq!(1, node_two.service_count());

    node_one.deregister("echo").unwrap();
    thread::sleep(::std::time::Duration::from_millis(100));

    assert_eq!(0, node_one.service_count());
    assert_eq!(0, node_two.service_count());

    assert_eq!(Err(request::Error::ServiceDoesNotExists), node_two.request_bytes("echo", b"test"));
}
