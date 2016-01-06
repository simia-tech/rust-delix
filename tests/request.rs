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

use delix::node::State;

#[test]
fn single_echo_from_local() {
    helper::set_up();

    let node = helper::build_node("127.0.0.1:3001", &[]);
    node.register("echo", Box::new(|request| Ok(request.to_vec())))
            .unwrap();

    thread::sleep_ms(100);
    helper::assert_node(&node, State::Discovering, 0);

    let response = node.request("echo", b"test message").unwrap();
    assert_eq!(b"test message".to_vec(), response);
}

#[test]
fn single_echo_from_remote() {
    helper::set_up();

    let node_one = helper::build_node("127.0.0.1:3011", &[]);
    node_one.register("echo", Box::new(|request| Ok(request.to_vec())))
            .unwrap();

    let node_two = helper::build_node("127.0.0.1:3012", &["127.0.0.1:3011"]);

    thread::sleep_ms(1000);
    helper::assert_node(&node_one, State::Joined, 1);
    helper::assert_node(&node_two, State::Joined, 1);

    let response = node_two.request("echo", b"test message").unwrap();
    assert_eq!(b"test message".to_vec(), response);
}

#[test]
fn multiple_echos_from_remote() {
    helper::set_up();

    let node_one = helper::build_node("127.0.0.1:3021", &[]);
    node_one.register("echo", Box::new(|request| Ok(request.to_vec())))
            .unwrap();

    let node_two = helper::build_node("127.0.0.1:3022", &["127.0.0.1:3021"]);

    thread::sleep_ms(1000);
    helper::assert_node(&node_one, State::Joined, 1);
    helper::assert_node(&node_two, State::Joined, 1);

    assert_eq!(b"test message one".to_vec(), node_two.request("echo", b"test message one").unwrap());
    assert_eq!(b"test message two".to_vec(), node_two.request("echo", b"test message two").unwrap());
}

#[test]
fn balanced_echos_from_two_remotes() {
    helper::set_up();

    let node_one = helper::build_node("127.0.0.1:3031", &[]);
    node_one.register("echo", Box::new(|_| {
        thread::sleep_ms(10);
        Ok(b"echo one".to_vec())
    })).unwrap();

    let node_two = helper::build_node("127.0.0.1:3032", &["127.0.0.1:3031"]);
    node_two.register("echo", Box::new(|_| {
        thread::sleep_ms(100);
        Ok(b"echo two".to_vec())
    })).unwrap();

    let node_three = helper::build_node("127.0.0.1:3033", &["127.0.0.1:3031"]);

    thread::sleep_ms(1000);
    helper::assert_node(&node_one, State::Joined, 2);
    helper::assert_node(&node_two, State::Joined, 2);
    helper::assert_node(&node_three, State::Joined, 2);

    // in the first round the balancer has no statistic, so every serivice gets a request in order.
    assert_eq!("echo one", String::from_utf8_lossy(&node_three.request("echo", b"").unwrap()));
    assert_eq!("echo two", String::from_utf8_lossy(&node_three.request("echo", b"").unwrap()));

    // in the second round the balancer can access some respond time statistic, so this round
    // contains two requests to node one and one to node two.
    assert_eq!("echo one", String::from_utf8_lossy(&node_three.request("echo", b"").unwrap()));
    assert_eq!("echo one", String::from_utf8_lossy(&node_three.request("echo", b"").unwrap()));
    assert_eq!("echo two", String::from_utf8_lossy(&node_three.request("echo", b"").unwrap()));

    // if a service deregisters in the middle of a round, the changes should be processed
    assert_eq!("echo one", String::from_utf8_lossy(&node_three.request("echo", b"").unwrap()));
    node_two.deregister("echo").unwrap();
    assert_eq!("echo one", String::from_utf8_lossy(&node_three.request("echo", b"").unwrap()));
    assert_eq!("echo one", String::from_utf8_lossy(&node_three.request("echo", b"").unwrap()));
}
