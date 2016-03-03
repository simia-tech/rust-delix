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

use super::{Metric, item};

pub struct Terminal;

impl Terminal {
    pub fn new() -> Self {
        Terminal
    }
}

impl Metric for Terminal {
    fn counter(&self, key: &str) -> item::Counter {
        item::Counter::new(Box::new(move |delta_value| {
        }))
    }

    fn gauge(&self, key: &str) -> item::Gauge {
        item::Gauge::new(Box::new(move |new_value| {
                         }),
                         Box::new(move |delta_value| {
                         }))
    }

    fn display(&self) {}
}
