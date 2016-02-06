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
use delix::util::reader;

#[test]
#[allow(unused_variables)]
fn loose() {
    helper::set_up();

    let (node_one, metric_one) = helper::build_node("127.0.0.1:3001", &[], None);
    {
        let (node_two, metric_two) = helper::build_node("127.0.0.1:3002", &["127.0.0.1:3001"], None);
        helper::wait_for_joined(&[&metric_one, &metric_two]);
    }

    helper::wait_for_discovering(&metric_one);
}

#[test]
#[allow(unused_variables)]
fn loose_and_service_clean_up() {
    helper::set_up();

    let (node_one, metric_one) = helper::build_node("127.0.0.1:3011", &[], None);
    {
        let (node_two, metric_two) = helper::build_node("127.0.0.1:3012", &["127.0.0.1:3011"], None);
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

    let (node_one, metric_one) = helper::build_node("127.0.0.1:3021", &[], None);
    let (node_two, metric_two) = helper::build_node("127.0.0.1:3022", &["127.0.0.1:3021"], None);
    node_two.register("echo", Box::new(|request| {
        Ok(request)
    })).unwrap();

    helper::wait_for_joined(&[&metric_one, &metric_two]);
    helper::wait_for_services(&[&metric_one, &metric_two], 1);

    let request = Box::new(reader::ErrorAfter::new_connection_lost(io::Cursor::new(b"test message".to_vec()), 4));
    assert!(node_one.request("echo", request, Box::new(Vec::new())).is_err());
}
