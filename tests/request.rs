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

use std::io;
use std::iter;
use std::sync::{Arc, RwLock, mpsc};
use std::thread;

use delix::node::request;

#[test]
fn single_echo_from_local_without_timeout() {
    helper::set_up();

    let (node, metric) = helper::build_node("localhost:3001", &[], None);
    node.register("echo", Box::new(|request| Ok(request)))
            .unwrap();

    helper::wait_for_services(&[&metric], 1);

    assert_eq!("test message", String::from_utf8_lossy(&node.request_bytes("echo", b"test message").unwrap()));
}

#[test]
fn single_large_echo_from_local_without_timeout() {
    helper::set_up();

    let (node, metric) = helper::build_node("localhost:3011", &[], None);
    node.register("echo", Box::new(|request| Ok(request)))
            .unwrap();

    helper::wait_for_services(&[&metric], 1);

    let request_bytes = iter::repeat(0u8).take(70000).collect::<Vec<_>>();
    let request = Box::new(io::Cursor::new(request_bytes.clone()));
    node.request("echo", request, Box::new(move |mut reader| {
        assert_eq!(Some(70000), io::copy(&mut reader, &mut io::sink()).ok());
    })).unwrap();
}

#[test]
fn single_echo_from_local_with_timeout() {
    helper::set_up();

    let (node, metric) = helper::build_node("localhost:3021", &[], Some(10));
    node.register("echo", Box::new(|request| {
        thread::sleep(::std::time::Duration::from_millis(20));
        Ok(request)
    })).unwrap();

    helper::wait_for_services(&[&metric], 1);

    assert_eq!(Err(request::Error::Timeout), node.request_bytes("echo", b""));
}

#[test]
fn single_echo_from_remote_without_timeout() {
    helper::set_up();

    let (node_one, metric_one) = helper::build_node("localhost:3031", &[], None);
    node_one.register("echo", Box::new(|request| Ok(request)))
            .unwrap();

    let (node_two, metric_two) = helper::build_node("localhost:3032", &["localhost:3031"], None);

    helper::wait_for_joined(&[&metric_one, &metric_two]);
    helper::wait_for_services(&[&metric_one, &metric_two], 1);

    assert_eq!("test message", String::from_utf8_lossy(&node_two.request_bytes("echo", b"test message").unwrap()));
}

#[test]
fn single_echo_from_remote_with_timeout() {
    helper::set_up();

    let (node_one, metric_one) = helper::build_node("localhost:3041", &[], None);
    node_one.register("echo", Box::new(|request| {
        thread::sleep(::std::time::Duration::from_millis(20));
        Ok(request)
    })).unwrap();

    let (node_two, metric_two) = helper::build_node("localhost:3042", &["localhost:3041"], Some(10));

    helper::wait_for_joined(&[&metric_one, &metric_two]);
    helper::wait_for_services(&[&metric_one, &metric_two], 1);

    assert_eq!(Err(request::Error::Timeout), node_two.request_bytes("echo", b""));
}

#[test]
fn multiple_echos_from_remote() {
    helper::set_up();

    let (node_one, metric_one) = helper::build_node("localhost:3051", &[], None);
    node_one.register("echo", Box::new(|request| Ok(request)))
            .unwrap();

    let (node_two, metric_two) = helper::build_node("localhost:3052", &["localhost:3051"], None);

    helper::wait_for_joined(&[&metric_one, &metric_two]);
    helper::wait_for_services(&[&metric_one, &metric_two], 1);

    assert_eq!(b"test message one".to_vec(), node_two.request_bytes("echo", b"test message one").unwrap());
    assert_eq!(b"test message two".to_vec(), node_two.request_bytes("echo", b"test message two").unwrap());
}

#[test]
fn balanced_echos_from_two_remotes() {
    helper::set_up();

    let (node_one, metric_one) = helper::build_node("localhost:3061", &[], None);

    let (tx, rx) = mpsc::channel();

    let (node_two, metric_two) = helper::build_node("localhost:3062", &["localhost:3061"], None);
    let tx_clone = tx.clone();
    node_two.register("echo", Box::new(move |request| {
        tx_clone.send("two").unwrap();
        Ok(request)
    })).unwrap();

    let (node_three, metric_three) = helper::build_node("localhost:3063", &["localhost:3061"], None);
    let tx_clone = tx.clone();
    node_three.register("echo", Box::new(move |request| {
        tx_clone.send("three").unwrap();
        Ok(request)
    })).unwrap();

    helper::wait_for_joined(&[&metric_one, &metric_two, &metric_three]);
    helper::wait_for_services(&[&metric_one, &metric_two, &metric_three], 1);

    assert_eq!("test", String::from_utf8_lossy(&node_one.request_bytes("echo", b"test").unwrap()));
    assert_eq!("test", String::from_utf8_lossy(&node_one.request_bytes("echo", b"test").unwrap()));

    helper::assert_contains_all(&["two", "three"], &helper::recv_all(&rx));

    node_three.deregister("echo").unwrap();

    assert_eq!("test", String::from_utf8_lossy(&node_one.request_bytes("echo", b"test").unwrap()));
    assert_eq!("test", String::from_utf8_lossy(&node_one.request_bytes("echo", b"test").unwrap()));

    helper::assert_contains_all(&["two", "two"], &helper::recv_all(&rx));
}

#[test]
fn parallel_requests_while_a_node_is_joining() {
    helper::set_up();

    let (node_one, metric_one) = helper::build_node("localhost:3071", &[], None);
    node_one.register("echo", Box::new(move |request| {
        Ok(request)
    })).unwrap();

    let running = Arc::new(RwLock::new(true));

    let node_one_clone = node_one.clone();
    let running_clone = running.clone();
    let jh_one = thread::spawn(move || {
        while *running_clone.read().unwrap() {
            assert_eq!("test", String::from_utf8_lossy(&node_one_clone.request_bytes("echo", b"test").unwrap()));
            thread::sleep(::std::time::Duration::from_millis(10));
        }
    });

    let node_one_clone = node_one.clone();
    let running_clone = running.clone();
    let jh_two = thread::spawn(move || {
        while *running_clone.read().unwrap() {
            assert_eq!("test", String::from_utf8_lossy(&node_one_clone.request_bytes("echo", b"test").unwrap()));
            thread::sleep(::std::time::Duration::from_millis(10));
        }
    });

    thread::sleep(::std::time::Duration::from_millis(50));

    let (node_two, metric_two) = helper::build_node("localhost:3072", &["localhost:3071"], None);
    node_two.register("echo", Box::new(move |request| {
        Ok(request)
    })).unwrap();

    helper::wait_for_joined(&[&metric_one, &metric_two]);
    helper::wait_for_services(&[&metric_one, &metric_two], 1);
    helper::wait_for_endpoints(&[&metric_one, &metric_two], 2);

    thread::sleep(::std::time::Duration::from_millis(50));

    *running.write().unwrap() = false;
    jh_one.join().unwrap();
    jh_two.join().unwrap();
}
