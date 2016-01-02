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

/// `path` arguments are always slices comprising the identity of some counter,
/// with segments appearing in order of increasing specificity,
/// e.g. `&["handshakes", "Billy", "Tuesday"]`.
pub trait StatCollector : Send + Sync {
    // Increment the described counter by 1
    fn increment(&self, path: &[&str]);
    // Decrement the described counter by 1
    fn decrement(&self, path: &[&str]);
}

pub struct DebugStatCollector;

impl StatCollector for DebugStatCollector {
    fn increment(&self, path: &[&str]) {
        println!("incrementing stat {}", path.join("."));
    }

    fn decrement(&self, path: &[&str]) {
        println!("decrementing stat {}", path.join("."));
    }
}

pub struct NullStatCollector;

impl StatCollector for NullStatCollector {
    fn increment(&self, _path: &[&str]) {}
    fn decrement(&self, _path: &[&str]) {}
}

pub struct MultiStatCollector {
    collectors: Vec<Box<StatCollector>>,
}

impl MultiStatCollector {
    pub fn new(cs: Vec<Box<StatCollector>>) -> MultiStatCollector {
        return MultiStatCollector { collectors: cs };
    }
}

impl StatCollector for MultiStatCollector {
    fn increment(&self, path: &[&str]) {
        for c in &self.collectors {
            c.increment(path);
        }
    }

    fn decrement(&self, path: &[&str]) {
        for c in &self.collectors {
            c.decrement(path);
        }
    }
}
