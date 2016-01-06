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

#[test]
fn loose() {
    helper::set_up();

    let node_one = helper::build_node("127.0.0.1:3001", &[]);
    {
        let node_two = helper::build_node("127.0.0.1:3002", &["127.0.0.1:3001"]);

        sleep_ms(1000);
        helper::assert_node(&node_one, State::Joined, 1);
        helper::assert_node(&node_two, State::Joined, 1);
    }

    sleep_ms(1000);
    helper::assert_node(&node_one, State::Discovering, 0);
}

#[test]
fn loose_and_service_clean_up() {
    helper::set_up();

    let node_one = helper::build_node("127.0.0.1:3011", &[]);
    {
        let node_two = helper::build_node("127.0.0.1:3012", &["127.0.0.1:3011"]);
        node_two.register("echo", Box::new(|request| {
            Ok(request.to_vec())
        })).unwrap();

        sleep_ms(1000);
        helper::assert_node(&node_one, State::Joined, 1);
        helper::assert_node(&node_two, State::Joined, 1);
        assert_eq!(1, node_one.service_count());
        assert_eq!(1, node_two.service_count());
    }

    sleep_ms(1000);
    helper::assert_node(&node_one, State::Discovering, 0);
    assert_eq!(0, node_one.service_count());
}
