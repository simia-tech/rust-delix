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

use std::collections::{HashMap, VecDeque};
use std::sync::RwLock;
use time::{self, Duration};

use transport::direct::tracker::{Subject, Store};

const MAXIMAL_SIZE: usize = 20;

pub struct Statistic {
    entries: RwLock<HashMap<Subject, VecDeque<Duration>>>,
}

impl Statistic {
    pub fn new() -> Statistic {
        Statistic { entries: RwLock::new(HashMap::new()) }
    }

    pub fn push(&self, subject: Subject, duration: Duration) {
        let mut entries = self.entries.write().unwrap();

        if !entries.contains_key(&subject) {
            entries.insert(subject.clone(), VecDeque::with_capacity(MAXIMAL_SIZE));
        }
        let durations = entries.get_mut(&subject).unwrap();

        while durations.len() >= durations.capacity() {
            durations.pop_front();
        }
        durations.push_back(duration);
    }

    pub fn average(&self, store: &Store, subject: &Subject) -> Duration {
        let entries = self.entries.read().unwrap();
        let durations = match entries.get(&subject) {
            Some(durations) => durations,
            None => return Duration::zero(),
        };

        let mut sum = durations.iter().fold(Duration::zero(), |sum, &duration| sum + duration);
        let mut count = durations.len() as i32;

        store.started_ats_with_subject(subject, |times| {
            let now = time::now_utc();
            sum = sum +
                  times.iter().fold(Duration::zero(),
                                    |sum, &started_at| sum + (now - *started_at));
            count += times.len() as i32;
        });

        sum / count
    }
}

#[cfg(test)]
mod tests {

    use std::thread;
    use std::sync::mpsc;
    use time::{self, Duration};
    use super::Statistic;
    use super::super::{Subject, Store};

    #[test]
    fn add() {
        let store = Store::new();
        let statistic = Statistic::new();
        let subject = Subject::local("test");
        assert_eq!(Duration::zero(), statistic.average(&store, &subject));

        statistic.push(subject.clone(), Duration::milliseconds(100));

        assert_eq!(Duration::milliseconds(100),
                   statistic.average(&store, &subject));
    }

    #[test]
    fn average() {
        let store = Store::new();
        let statistic = Statistic::new();
        let subject = Subject::local("test");

        statistic.push(subject.clone(), Duration::milliseconds(100));
        statistic.push(subject.clone(), Duration::milliseconds(200));

        assert_eq!(Duration::milliseconds(150),
                   statistic.average(&store, &subject));
    }

    #[test]
    fn average_including_running_requests_in_store() {
        let store = Store::new();
        let statistic = Statistic::new();
        let subject = Subject::local("test");

        statistic.push(subject.clone(), Duration::milliseconds(100));

        let (response_tx, _) = mpsc::channel();
        store.insert(10, response_tx, subject.clone(), time::now_utc()).unwrap();
        thread::sleep_ms(50);

        assert!(statistic.average(&store, &subject) > Duration::milliseconds(50));
        assert!(statistic.average(&store, &subject) < Duration::milliseconds(100));
    }

}
