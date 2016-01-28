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

use std::collections::{HashMap, hash_map};
use std::sync::{RwLock, mpsc};

use super::{Metric, metric};

pub struct Memory {
    entries: RwLock<HashMap<String, Entry>>,
    watches: RwLock<HashMap<String, mpsc::Sender<(String, Entry)>>>,
}

#[derive(Clone, Debug, PartialEq)]
enum Entry {
    Counter(usize),
    Gauge(isize),
}

impl Memory {
    pub fn new() -> Self {
        Memory {
            entries: RwLock::new(HashMap::new()),
            watches: RwLock::new(HashMap::new()),
        }
    }

    pub fn get_counter(&self, key: &str) -> usize {
        let entries = self.entries.read().unwrap();
        entries.get(key)
               .map(|entry| {
                   if let Entry::Counter(value) = *entry {
                       value
                   } else {
                       0
                   }
               })
               .unwrap_or(0)
    }

    pub fn get_gauge(&self, key: &str) -> isize {
        let entries = self.entries.read().unwrap();
        entries.get(key)
               .map(|entry| {
                   if let Entry::Gauge(value) = *entry {
                       value
                   } else {
                       0
                   }
               })
               .unwrap_or(0)
    }

    fn update_counter<F>(&self, key: &str, default: usize, f: F)
        where F: Fn(&mut usize)
    {
        self.update(key, Entry::Counter(default), |entry| {
            if let Entry::Counter(ref mut value) = *entry {
                f(value);
            }
        });
    }

    fn update_gauge<F>(&self, key: &str, default: isize, f: F)
        where F: Fn(&mut isize)
    {
        self.update(key, Entry::Gauge(default), |entry| {
            if let Entry::Gauge(ref mut value) = *entry {
                f(value);
            }
        });
    }

    fn update<F>(&self, key: &str, default: Entry, f: F)
        where F: Fn(&mut Entry)
    {
        let mut entries = self.entries.write().unwrap();
        let entry = match entries.entry(key.to_string()) {
            hash_map::Entry::Vacant(ve) => {
                ve.insert(default.clone());
                default
            }
            hash_map::Entry::Occupied(ref mut oe) => {
                f(oe.get_mut());
                oe.get().clone()
            }
        };

        let watches = self.watches.read().unwrap();
        if let Some(tx) = watches.get(key) {
            tx.send((key.to_string(), entry)).unwrap();
        }
    }

    pub fn watch_counter<F>(&self, pattern: &str, f: F)
        where F: Fn(&str, usize) -> bool
    {
        self.watch(pattern, |key, entry| {
            if let Entry::Counter(value) = *entry {
                return f(key, value);
            }
            false
        });
    }

    pub fn watch_gauge<F>(&self, pattern: &str, f: F)
        where F: Fn(&str, isize) -> bool
    {
        self.watch(pattern, |key, entry| {
            if let Entry::Gauge(value) = *entry {
                return f(key, value);
            }
            false
        });
    }

    fn watch<F>(&self, pattern: &str, f: F)
        where F: Fn(&str, &Entry) -> bool
    {
        let (tx, rx) = mpsc::channel();
        {
            let mut watches = self.watches.write().unwrap();
            if watches.contains_key(pattern) {
                panic!("entry exists");
            }
            watches.insert(pattern.to_string(), tx);
        }

        let (mut key, mut entry) = rx.recv().unwrap();
        while f(&key, &entry) {
            let tuple = rx.recv().unwrap();
            key = tuple.0;
            entry = tuple.1;
        }

        {
            let mut watches = self.watches.write().unwrap();
            watches.remove(pattern);
        }
    }
}

impl Metric for Memory {
    fn increment_counter(&self, key: &str) {
        self.update_counter(key, 1, |value| {
            *value += 1;
        });
    }

    fn change_gauge(&self, key: &str, change: metric::Change) {
        match change {
            metric::Change::Set(new_value) => {
                self.update_gauge(key, new_value, |value| *value = new_value);
            }
            metric::Change::Delta(delta_value) => {
                self.update_gauge(key, delta_value, |value| *value += delta_value);
            }
        }
    }
}

unsafe impl Send for Memory {}

unsafe impl Sync for Memory {}

#[cfg(test)]
mod tests {

    use std::sync::Arc;
    use std::thread;
    use super::super::{Metric, metric};
    use super::Memory;

    #[test]
    fn increment_counter() {
        let metric = Memory::new();
        assert_eq!(0, metric.get_counter("test"));
        metric.increment_counter("test");
        assert_eq!(1, metric.get_counter("test"));
        metric.increment_counter("test");
        assert_eq!(2, metric.get_counter("test"));
    }

    #[test]
    fn increment_counter_concurrently() {
        let metric = Arc::new(Memory::new());

        let metric_clone = metric.clone();
        let jh1 = thread::spawn(move || {
            for _ in 0..10 {
                metric_clone.increment_counter("test");
            }
        });

        let metric_clone = metric.clone();
        let jh2 = thread::spawn(move || {
            for _ in 0..10 {
                metric_clone.increment_counter("test");
            }
        });

        jh1.join().unwrap();
        jh2.join().unwrap();

        assert_eq!(20, metric.get_counter("test"));
    }

    #[test]
    fn watch_counter() {
        let metric = Arc::new(Memory::new());

        let metric_clone = metric.clone();
        thread::spawn(move || {
            for _ in 0..10 {
                metric_clone.increment_counter("test");
            }
        });

        metric.watch_counter("test", |_, value| value < 10);
    }

    #[test]
    fn change_gauge() {
        let metric = Memory::new();
        assert_eq!(0, metric.get_gauge("test"));
        metric.change_gauge("test", metric::Change::Set(10));
        assert_eq!(10, metric.get_gauge("test"));
        metric.change_gauge("test", metric::Change::Delta(-5));
        assert_eq!(5, metric.get_gauge("test"));
        metric.change_gauge("test", metric::Change::Delta(10));
        assert_eq!(15, metric.get_gauge("test"));
    }

    #[test]
    fn change_gauge_concurrently() {
        let metric = Arc::new(Memory::new());

        let metric_clone = metric.clone();
        let jh1 = thread::spawn(move || {
            for _ in 0..10 {
                metric_clone.change_gauge("test", metric::Change::Delta(5));
            }
        });

        let metric_clone = metric.clone();
        let jh2 = thread::spawn(move || {
            for _ in 0..10 {
                metric_clone.change_gauge("test", metric::Change::Delta(-10));
            }
        });

        jh1.join().unwrap();
        jh2.join().unwrap();

        assert_eq!(-50isize, metric.get_gauge("test"));
    }

    #[test]
    fn watch_gauge() {
        let metric = Arc::new(Memory::new());

        let metric_clone = metric.clone();
        thread::spawn(move || {
            for _ in 0..10 {
                metric_clone.change_gauge("test", metric::Change::Delta(-1isize));
            }
        });

        metric.watch_gauge("test", |_, value| value > -10);
    }

}
