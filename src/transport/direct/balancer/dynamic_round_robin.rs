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

use std::iter::Iterator;
use std::sync::Arc;

use super::balancer::Balancer;
use super::factory::Factory;
use super::super::Link;
use super::super::tracker::Statistic;

use time::Duration;

pub struct DynamicRoundRobinFactory {
    statistic: Option<Arc<Statistic>>,
}

impl DynamicRoundRobinFactory {
    pub fn new() -> Self {
        DynamicRoundRobinFactory { statistic: None }
    }
}

impl Factory for DynamicRoundRobinFactory {
    fn set_statistic(&mut self, statistic: Arc<Statistic>) {
        self.statistic = Some(statistic);
    }

    fn build(&self, name: &str) -> Box<Balancer<Item = Link>> {
        Box::new(DynamicRoundRobin::new(self.statistic
                                            .as_ref()
                                            .expect("statistic must be set before the factory \
                                                     can build a dynamic round robin balancer")
                                            .clone(),
                                        name))
    }
}

pub struct DynamicRoundRobin {
    statistic: Arc<Statistic>,
    name: String,
    links: Vec<Link>,
    queue: Vec<Link>,
}

impl DynamicRoundRobin {
    pub fn new(statistic: Arc<Statistic>, name: &str) -> Self {
        DynamicRoundRobin {
            statistic: statistic,
            name: name.to_string(),
            links: Vec::new(),
            queue: Vec::new(),
        }
    }

    fn build_round(&mut self) {
        self.queue = Vec::new();
        if self.links.is_empty() {
            return;
        }

        let durations = self.links
                            .iter()
                            .map(|link| self.statistic.average(&self.name, link))
                            .collect::<Vec<_>>();

        let longest = durations.iter().max().unwrap();
        if longest == &Duration::zero() {
            self.queue.append(&mut self.links.clone());
            self.queue.reverse();
            return;
        }

        let counts = durations.iter()
                              .map(|&duration| {
                                  let ms = duration.num_milliseconds();
                                  if ms == 0 {
                                      1
                                  } else {
                                      longest.num_milliseconds() / ms
                                  }
                              })
                              .collect::<Vec<_>>();

        for (index, &count) in counts.iter().enumerate() {
            for _ in 0..count {
                self.queue.push(self.links[index]);
            }
            self.queue.reverse();
        }
    }
}

impl Balancer for DynamicRoundRobin {
    fn set_links(&mut self, links: &[Link]) {
        self.links = links.to_vec();
        self.queue = Vec::new();
    }
}

impl Iterator for DynamicRoundRobin {
    type Item = Link;

    fn next(&mut self) -> Option<Link> {
        if let Some(link) = self.queue.pop() {
            return Some(link);
        }

        self.build_round();

        self.queue.pop()
    }
}

#[cfg(test)]
mod tests {

    use std::sync::Arc;
    use time::Duration;

    use super::DynamicRoundRobinFactory;
    use super::super::Balancer;
    use super::super::Factory;
    use node::ID;
    use transport::direct::Link;
    use transport::direct::tracker::{Statistic, Subject};

    #[test]
    fn round_building_without_statistic() {
        let mut factory = DynamicRoundRobinFactory::new();
        factory.set_statistic(Arc::new(Statistic::new()));
        let mut balancer = factory.build("test");

        let link_one = Link::Local;
        let link_two = Link::Remote(ID::new_random());
        balancer.set_links(&[link_one, link_two]);

        assert_eq!(vec![link_one, link_two],
                   balancer.take(2).collect::<Vec<_>>());
    }

    #[test]
    fn round_building_with_some_statistic() {
        let remote_id = ID::new_random();

        let statistic = Arc::new(Statistic::new());
        statistic.push(Subject::local("test"), Duration::milliseconds(50));
        statistic.push(Subject::remote("test", remote_id),
                       Duration::milliseconds(100));

        let mut factory = DynamicRoundRobinFactory::new();
        factory.set_statistic(statistic);
        let mut balancer = factory.build("test");

        let link_one = Link::Local;
        let link_two = Link::Remote(remote_id);
        balancer.set_links(&[link_one, link_two]);

        assert_eq!(vec![link_one, link_one, link_two],
                   balancer.take(3).collect::<Vec<_>>());
    }
}
