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

#[test]
fn two_nodes() {
    helper::set_up();

    let node_one = helper::build_node("127.0.0.1:3001", &[], None);
    let node_two = helper::build_node("127.0.0.1:3002", &["127.0.0.1:3001"], None);

    helper::wait_for_joined(&[&node_one, &node_two]);
}

#[test]
fn three_nodes() {
    helper::set_up();

    let node_one = helper::build_node("127.0.0.1:3011", &[], None);
    let node_two = helper::build_node("127.0.0.1:3012", &["127.0.0.1:3011"], None);
    let node_three = helper::build_node("127.0.0.1:3013", &["127.0.0.1:3011"], None);

    helper::wait_for_joined(&[&node_one, &node_two, &node_three]);
}
