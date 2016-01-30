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

#[allow(dead_code)] mod assert;
#[allow(dead_code)] mod log;
#[allow(dead_code)] mod node;
#[allow(dead_code)] mod relay;

pub use helper::assert::*;
pub use helper::log::*;
pub use helper::node::*;
pub use helper::relay::*;

use std::sync::mpsc;

#[allow(dead_code)] pub fn recv_all<T>(rx: &mpsc::Receiver<T>) -> Vec<T> {
    let mut result = Vec::new();
    loop {
        result.push(match rx.try_recv() {
            Ok(value) => value,
            Err(mpsc::TryRecvError::Empty) => break,
            Err(error) => panic!(error),
        });
    }
    result
}
