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
use std::sync::{Arc, RwLock, atomic};
use super::{Metric, Query, Value, item};

pub struct Memory {
    entries: RwLock<HashMap<String, Arc<Entry>>>,
}

impl Memory {
    pub fn new() -> Self {
        Memory { entries: RwLock::new(HashMap::new()) }
    }

    fn get_or_insert(&self, key: &str, default: Entry) -> Arc<Entry> {
        let mut entries = self.entries.write().unwrap();
        match entries.entry(key.to_string()) {
            hash_map::Entry::Vacant(ve) => {
                let entry = Arc::new(default);
                ve.insert(entry.clone());
                entry
            }
            hash_map::Entry::Occupied(ref mut oe) => oe.get().clone(),
        }
    }
}

impl Metric for Memory {
    fn counter(&self, key: &str) -> item::Counter {
        let entry = self.get_or_insert(key, Entry::Counter(atomic::AtomicUsize::new(0)));
        item::Counter::new(Box::new(move |delta_value| {
            if let Entry::Counter(ref value) = *entry {
                value.fetch_add(delta_value, atomic::Ordering::SeqCst);
            }
        }))
    }

    fn gauge(&self, key: &str) -> item::Gauge {
        let entry = self.get_or_insert(key, Entry::Gauge(atomic::AtomicIsize::new(0)));
        let entry_clone = entry.clone();
        item::Gauge::new(Box::new(move |new_value| {
                             if let Entry::Gauge(ref value) = *entry_clone {
                                 value.store(new_value, atomic::Ordering::SeqCst);
                             }
                         }),
                         Box::new(move |delta_value| {
                             if let Entry::Gauge(ref value) = *entry {
                                 value.fetch_add(delta_value, atomic::Ordering::SeqCst);
                             }
                         }))
    }
}

impl Query for Memory {
    fn get(&self, key: &str) -> Option<Value> {
        let entries = self.entries.read().unwrap();
        entries.get(key).map(|entry| Value::from(&**entry))
    }
}

#[derive(Debug)]
pub enum Entry {
    Counter(atomic::AtomicUsize),
    Gauge(atomic::AtomicIsize),
}

impl<'a> From<&'a Entry> for Value {
    fn from(entry: &Entry) -> Self {
        match *entry {
            Entry::Counter(ref value) => Value::Counter(value.load(atomic::Ordering::SeqCst)),
            Entry::Gauge(ref value) => Value::Gauge(value.load(atomic::Ordering::SeqCst)),
        }
    }
}

#[cfg(test)]
mod tests {

    use std::thread;
    use super::Memory;
    use super::super::{Metric, Query, Value};

    #[test]
    fn counter() {
        let metric = Memory::new();
        let counter = metric.counter("test");
        counter.increment();
        assert_eq!(Some(Value::Counter(1)), metric.get("test"));
    }

    #[test]
    fn concurrent_counter() {
        let metric = Memory::new();

        let counter = metric.counter("test");
        let jh1 = thread::spawn(move || {
            for _ in 0..10 {
                counter.increment();
            }
        });

        let counter = metric.counter("test");
        let jh2 = thread::spawn(move || {
            for _ in 0..10 {
                counter.increment();
            }
        });

        jh1.join().unwrap();
        jh2.join().unwrap();

        assert_eq!(Some(Value::Counter(20)), metric.get("test"));
    }

    #[test]
    fn gauge() {
        let memory = Memory::new();
        let gauge = memory.gauge("test");
        gauge.set(10);
        assert_eq!(Some(Value::Gauge(10)), memory.get("test"));
        gauge.change(-5);
        assert_eq!(Some(Value::Gauge(5)), memory.get("test"));
        gauge.change(10);
        assert_eq!(Some(Value::Gauge(15)), memory.get("test"));
    }

    #[test]
    fn concurrent_gauge() {
        let metric = Memory::new();

        let gauge = metric.gauge("test");
        let jh1 = thread::spawn(move || {
            for _ in 0..10 {
                gauge.change(5);
            }
        });

        let gauge = metric.gauge("test");
        let jh2 = thread::spawn(move || {
            for _ in 0..10 {
                gauge.change(-10);
            }
        });

        jh1.join().unwrap();
        jh2.join().unwrap();

        assert_eq!(Some(Value::Gauge(-50)), metric.get("test"));
    }

}
