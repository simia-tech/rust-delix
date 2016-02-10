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

#[test]
#[allow(unused_variables)]
fn two_nodes() {
    helper::set_up();

    let (node_one, metric_one) = helper::build_node("localhost:3001", &[], None);
    let (node_two, metric_two) = helper::build_node("localhost:3002", &["localhost:3001"], None);

    helper::wait_for_joined(&[&metric_one, &metric_two]);
}

#[test]
#[allow(unused_variables)]
fn three_nodes() {
    helper::set_up();

    let (node_one, metric_one) = helper::build_node("localhost:3011", &[], None);
    let (node_two, metric_two) = helper::build_node("localhost:3012", &["localhost:3011"], None);
    let (node_three, metric_three) = helper::build_node("localhost:3013", &["localhost:3011"], None);

    helper::wait_for_joined(&[&metric_one, &metric_two, &metric_three]);
}
