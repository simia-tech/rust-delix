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

use delix::node::request;

#[test]
#[allow(unused_variables)]
fn registration_over_incoming_connection() {
    helper::set_up();

    let (node_one, metric_one) = helper::build_node("localhost:3001", &[], None);
    node_one.register("echo", Box::new(|request| Ok(request)))
            .unwrap();

    let (node_two, metric_two) = helper::build_node("localhost:3002", &["localhost:3001"], None);

    helper::wait_for_joined(&[&metric_one, &metric_two]);
    helper::wait_for_services(&[&metric_one, &metric_two], 1);
}

#[test]
#[allow(unused_variables)]
fn registration_over_outgoing_connection() {
    helper::set_up();

    let (node_one, metric_one) = helper::build_node("localhost:3011", &[], None);

    let (node_two, metric_two) = helper::build_node("localhost:3012", &["localhost:3011"], None);
    node_two.register("echo", Box::new(|request| Ok(request)))
            .unwrap();

    helper::wait_for_joined(&[&metric_one, &metric_two]);
    helper::wait_for_services(&[&metric_one, &metric_two], 1);
}

#[test]
#[allow(unused_variables)]
fn registration_in_joined_network() {
    helper::set_up();

    let (node_one, metric_one) = helper::build_node("localhost:3021", &[], None);
    let (node_two, metric_two) = helper::build_node("localhost:3022", &["localhost:3021"], None);

    helper::wait_for_joined(&[&metric_one, &metric_two]);

    helper::wait_for_services(&[&metric_one, &metric_two], 0);
    node_one.register("echo", Box::new(|request| Ok(request)))
            .unwrap();
    helper::wait_for_services(&[&metric_one, &metric_two], 1);
}

#[test]
fn deregistration() {
    helper::set_up();

    let (node, metric) = helper::build_node("localhost:3031", &[], None);
    node.register("echo", Box::new(|request| Ok(request)))
        .unwrap();

    helper::wait_for_services(&[&metric], 1);
    node.deregister("echo").unwrap();
    helper::wait_for_services(&[&metric], 0);
}

#[test]
fn deregistration_in_joined_network() {
    helper::set_up();

    let (node_one, metric_one) = helper::build_node("localhost:3041", &[], None);
    node_one.register("echo", Box::new(|request| Ok(request)))
            .unwrap();

    let (node_two, metric_two) = helper::build_node("localhost:3042", &["localhost:3041"], None);

    helper::wait_for_joined(&[&metric_one, &metric_two]);

    helper::wait_for_services(&[&metric_one, &metric_two], 1);
    node_one.deregister("echo").unwrap();
    helper::wait_for_services(&[&metric_one, &metric_two], 0);

    assert_eq!(Err(request::Error::NoService), node_two.request_bytes("echo", b"test"));
}
