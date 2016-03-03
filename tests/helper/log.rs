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

extern crate log;

use std::sync::{self, Arc};

use delix::logger;
use delix::metric;

static START: sync::Once = sync::ONCE_INIT;

pub fn set_up() {
    START.call_once(|| {
        let metric: Arc<metric::Metric> = Arc::new(metric::Memory::new());
        logger::Console::init(log::LogLevelFilter::Trace, "delix", &metric).unwrap();
    });
}
