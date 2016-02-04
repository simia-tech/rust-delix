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
use std::sync::{Arc, Condvar, RwLock, Mutex, atomic};
use super::{Metric, Query, Value, item};

pub struct Memory {
    entries: RwLock<HashMap<String, Arc<Entry>>>,
    watches: Arc<RwLock<HashMap<u16,
                                (String,
                                 Box<Fn(&str, &Value) -> bool + Send + Sync>,
                                 Arc<(Mutex<bool>, Condvar)>)>>>,
    next_watch_id: RwLock<u16>,
}

impl Memory {
    pub fn new() -> Self {
        Memory {
            entries: RwLock::new(HashMap::new()),
            watches: Arc::new(RwLock::new(HashMap::new())),
            next_watch_id: RwLock::new(0u16),
        }
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
        let key = key.to_string();
        let watches = self.watches.clone();
        item::Counter::new(Box::new(move |delta_value| {
            if let Entry::Counter(ref value) = *entry {
                value.fetch_add(delta_value, atomic::Ordering::SeqCst);
                trigger_watches(&watches, &key, Value::from(&*entry));
            }
        }))
    }

    fn gauge(&self, key: &str) -> item::Gauge {
        let entry = self.get_or_insert(key, Entry::Gauge(atomic::AtomicIsize::new(0)));
        let entry_clone = entry.clone();
        let key = key.to_string();
        let key_clone = key.to_string();
        let watches = self.watches.clone();
        let watches_clone = self.watches.clone();
        item::Gauge::new(Box::new(move |new_value| {
                             if let Entry::Gauge(ref value) = *entry_clone {
                                 value.store(new_value, atomic::Ordering::SeqCst);
                                 trigger_watches(&watches_clone,
                                                 &key_clone,
                                                 Value::from(&*entry_clone));
                             }
                         }),
                         Box::new(move |delta_value| {
                             if let Entry::Gauge(ref value) = *entry {
                                 value.fetch_add(delta_value, atomic::Ordering::SeqCst);
                                 trigger_watches(&watches, &key, Value::from(&*entry));
                             }
                         }))
    }
}

impl Query for Memory {
    fn get(&self, key: &str) -> Option<Value> {
        let entries = self.entries.read().unwrap();
        entries.get(key).map(|entry| Value::from(&**entry))
    }

    fn watch<P>(&self, prefix: &str, predicate: P)
        where P: Fn(&str, &Value) -> bool + Send + Sync + 'static
    {
        let id = {
            let mut next_watch_id = self.next_watch_id.write().unwrap();
            let id = *next_watch_id;
            *next_watch_id += 1;
            id
        };

        let tuple = Arc::new((Mutex::new(false), Condvar::new()));
        {
            let mut watches = self.watches.write().unwrap();

            let entries = self.entries.read().unwrap();
            for (key, entry) in entries.iter() {
                if key.starts_with(prefix) && !predicate(key, &Value::from(&**entry)) {
                    return;
                }
            }

            watches.insert(id, (prefix.to_string(), Box::new(predicate), tuple.clone()));
        }

        let &(ref mutex, ref condvar) = &*tuple;
        let mut matched = mutex.lock().unwrap();
        while !*matched {
            matched = condvar.wait(matched).unwrap();
        }

        {
            let mut watches = self.watches.write().unwrap();
            watches.remove(&id);
        }
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

fn trigger_watches(watches: &Arc<RwLock<HashMap<u16,
                                                (String,
                                                 Box<Fn(&str, &Value) -> bool + Send + Sync>,
                                                 Arc<(Mutex<bool>, Condvar)>)>>>,
                   key: &str,
                   value: Value) {
    let watches = watches.read().unwrap();
    for (_, &(ref prefix, ref predicate, ref tuple)) in watches.iter() {
        if key.starts_with(prefix) && !predicate(&key, &value) {
            let &(ref mutex, ref condvar) = &**tuple;
            let mut matched = mutex.lock().unwrap();
            *matched = true;
            condvar.notify_all();
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
    fn watch_counter() {
        let metric = Memory::new();

        let counter = metric.counter("test");
        thread::spawn(move || {
            for _ in 0..10 {
                counter.increment();
            }
        });

        metric.watch("test", |_, value| *value < Value::Counter(10));
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

    #[test]
    fn watch_gauge() {
        let metric = Memory::new();

        let gauge = metric.gauge("test");
        thread::spawn(move || {
            for _ in 0..10 {
                gauge.change(-1);
            }
        });

        metric.watch("test", |_, value| *value > Value::Gauge(-10));
    }

    #[test]
    fn watch_gauges_using_prefix() {
        let metric = Memory::new();

        let gauge_one = metric.gauge("test_one");
        let gauge_two = metric.gauge("test_two");

        metric.watch("test", |_, value| *value != Value::Gauge(0));

        thread::spawn(move || {
            for _ in 0..10 {
                gauge_one.change(1);
                gauge_two.change(2);
            }
        });

        metric.watch("test", |_, value| *value < Value::Gauge(20));
    }
}
