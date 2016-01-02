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

use std::sync::{Arc, RwLock};

use transport::direct::Balancer;
use transport::direct::Link;
use transport::direct::tracker::Statistic;

use time::Duration;

pub struct DynamicRoundRobin {
    statistic: RwLock<Option<Arc<Statistic>>>,
}

impl DynamicRoundRobin {
    pub fn new() -> DynamicRoundRobin {
        DynamicRoundRobin { statistic: RwLock::new(None) }
    }
}

impl Balancer for DynamicRoundRobin {
    fn assign_statistic(&self, statistic: Arc<Statistic>) {
        *self.statistic.write().unwrap() = Some(statistic);
    }

    fn build_round(&self, name: &str, links: &[Link]) -> Vec<Link> {
        if links.len() == 0 {
            return links.to_vec();
        }

        let statistic_option = self.statistic.read().unwrap();
        let statistic = statistic_option.as_ref().unwrap();

        let durations = links.iter()
                             .map(|link| statistic.average(name, link))
                             .collect::<Vec<_>>();

        let longest = durations.iter().max().unwrap();
        if longest == &Duration::zero() {
            return links.to_vec();
        }

        let counts = durations.iter()
                              .map(|&duration| {
                                  longest.num_milliseconds() / duration.num_milliseconds()
                              })
                              .collect::<Vec<_>>();

        let mut result = Vec::new();
        for (index, &count) in counts.iter().enumerate() {
            for _ in 0..count {
                result.push(links[index].clone());
            }
        }
        result
    }
}
