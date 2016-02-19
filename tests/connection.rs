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

use std::error::Error;
use std::io;
use delix::util::{reader, writer};

#[test]
#[allow(unused_variables)]
fn loose() {
    helper::set_up();

    let (node_one, metric_one) = helper::build_node("localhost:3001", &[], None);
    {
        let (node_two, metric_two) = helper::build_node("localhost:3002", &["localhost:3001"], None);
        helper::wait_for_joined(&[&metric_one, &metric_two]);
    }

    helper::wait_for_discovering(&metric_one);
}

#[test]
#[allow(unused_variables)]
fn loose_and_service_clean_up() {
    helper::set_up();

    let (node_one, metric_one) = helper::build_node("localhost:3011", &[], None);
    {
        let (node_two, metric_two) = helper::build_node("localhost:3012", &["localhost:3011"], None);
        node_two.register("echo", Box::new(|request| {
            Ok(request)
        })).unwrap();

        helper::wait_for_joined(&[&metric_one, &metric_two]);
        helper::wait_for_services(&[&metric_one, &metric_two], 1);
    }

    helper::wait_for_discovering(&metric_one);
    helper::wait_for_services(&[&metric_one], 0);
}

#[test]
fn loose_while_transmitting_request() {
    helper::set_up();

    let (node_one, metric_one) = helper::build_node("localhost:3021", &[], None);
    let (node_two, metric_two) = helper::build_node("localhost:3022", &["localhost:3021"], None);
    node_two.register("echo", Box::new(|mut request| {
        let result = io::copy(&mut request, &mut io::sink()).unwrap_err();
        assert_eq!(io::ErrorKind::UnexpectedEof, result.kind());
        assert_eq!("unexpected EOF", result.description());
        Ok(Box::new(io::Cursor::new(b"test message".to_vec())))
    })).unwrap();

    helper::wait_for_joined(&[&metric_one, &metric_two]);
    helper::wait_for_services(&[&metric_one, &metric_two], 1);

    let request = Box::new(reader::ErrorAfter::new_unexpected_eof(io::Cursor::new(b"test message".to_vec()), 4));
    let response = Box::new(writer::Collector::new());
    assert!(node_one.request("echo", request, response.clone()).is_ok());
    assert_eq!("test message", String::from_utf8_lossy(&response.vec().unwrap()));
}

#[test]
fn loose_while_transmitting_response() {
    helper::set_up();

    let (node_one, metric_one) = helper::build_node("localhost:3031", &[], None);
    let (node_two, metric_two) = helper::build_node("localhost:3032", &["localhost:3031"], None);
    node_two.register("echo", Box::new(|request| {
        Ok(Box::new(reader::ErrorAfter::new_unexpected_eof(request, 4)))
    })).unwrap();

    helper::wait_for_joined(&[&metric_one, &metric_two]);
    helper::wait_for_services(&[&metric_one, &metric_two], 1);

    let request = Box::new(io::Cursor::new(b"test message".to_vec()));
    let response = Box::new(writer::Collector::new());
    assert!(node_one.request("echo", request, response.clone()).is_ok());
    assert!(String::from_utf8_lossy(&response.vec().unwrap()).starts_with("test"));
}
