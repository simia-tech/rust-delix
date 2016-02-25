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

use std::result;
use std::sync::{Arc, Mutex, atomic, mpsc};
use std::thread;

use time::{self, Duration};

use node::request;
use transport::direct::Link;
use transport::direct::tracker::{Statistic, Store, Subject};

const TIMEOUT_TOLERANCE_MS: i64 = 2;

pub struct Tracker<P, R> {
    store: Arc<Store<(P, Mutex<mpsc::Sender<Result<R>>>)>>,
    statistic: Arc<Statistic>,
    current_id: atomic::AtomicUsize,
    join_handle_and_running_tx: Option<(thread::JoinHandle<()>, Mutex<mpsc::Sender<bool>>)>,
}

pub type Result<T> = result::Result<T, Error>;

#[derive(Debug, PartialEq)]
pub enum Error {
    Timeout,
}

impl<P, R> Tracker<P, R>
    where P: Send + Sync + 'static,
          R: Send + 'static
{
    pub fn new(statistic: Arc<Statistic>, timeout: Option<Duration>) -> Self {
        let store: Arc<Store<(P, Mutex<mpsc::Sender<Result<R>>>)>> = Arc::new(Store::new());
        statistic.assign_query(store.clone());

        let store_clone = store.clone();
        let join_handle_and_running_tx = timeout.map(|timeout| {
            let (running_tx, running_rx) = mpsc::channel();
            (thread::spawn(move || {
                while running_rx.recv().unwrap() {
                    loop {
                        let now = time::now_utc();

                        let (removed, next_at) = store_clone.remove_all_started_before(now -
                                                                                       timeout);
                        for (_, (_, result_tx)) in removed {
                            result_tx.lock().unwrap().send(Err(Error::Timeout)).unwrap();
                        }

                        if next_at.is_none() {
                            break;
                        }
                        let wait_for = next_at.unwrap() - (now - timeout) +
                                       Duration::milliseconds(TIMEOUT_TOLERANCE_MS);
                        thread::sleep(::std::time::Duration::from_millis(wait_for.num_milliseconds() as u64));
                    }
                }
            }),
             Mutex::new(running_tx))
        });

        Tracker {
            store: store,
            statistic: statistic,
            current_id: atomic::AtomicUsize::new(0),
            join_handle_and_running_tx: join_handle_and_running_tx,
        }
    }

    pub fn begin(&self, name: &str, link: &Link, payload: P) -> (u32, mpsc::Receiver<Result<R>>) {
        let (result_tx, result_rx) = mpsc::channel();
        let id = self.current_id.fetch_add(1, atomic::Ordering::SeqCst) as u32;
        let subject = Subject::from_name_and_link(name, link);
        let started_at = time::now_utc();

        if self.store
               .insert(id, subject, started_at, (payload, Mutex::new(result_tx)))
               .unwrap() {
            if let Some((_, ref running_tx)) = self.join_handle_and_running_tx {
                running_tx.lock().unwrap().send(true).unwrap();
            }
        }

        (id, result_rx)
    }

    pub fn end<F>(&self, id: u32, f: F) -> bool
        where F: FnOnce(P) -> R
    {
        let (subject, started_at, (payload, result_tx)) = match self.store.remove(&id) {
            Ok(tuple) => tuple,
            Err(_) => return false,
        };

        // ignore error cause receiver could gone already (request timed out before)
        let _ = result_tx.lock().unwrap().send(Ok(f(payload)));

        self.statistic.push(subject, time::now_utc() - started_at);

        true
    }

    pub fn len(&self) -> usize {
        self.store.len()
    }
}

impl<P, R> Drop for Tracker<P, R> {
    fn drop(&mut self) {
        if let Some((join_handle, running_tx)) = self.join_handle_and_running_tx.take() {
            running_tx.lock().unwrap().send(false).unwrap();
            join_handle.join().unwrap();
        }
    }
}

impl From<Error> for request::Error {
    fn from(error: Error) -> Self {
        match error {
            Error::Timeout => request::Error::Timeout,
        }
    }
}

#[cfg(test)]
mod tests {

    use std::thread;
    use std::sync::Arc;
    use time::Duration;
    use super::{Error, Tracker};
    use super::super::Statistic;
    use super::super::super::Link;

    #[test]
    fn request_tracking() {
        let tracker = Tracker::new(Arc::new(Statistic::new()), None);

        let (id, result_rx) = tracker.begin("test", &Link::Local, "test payload");
        assert!(tracker.end(id, |payload| {
            assert_eq!("test payload", payload);
            "test result"
        }));

        assert_eq!(Ok("test result"), result_rx.recv().unwrap());
        assert_eq!(0, tracker.len());
    }

    #[test]
    fn request_timeout() {
        let tracker = Tracker::new(Arc::new(Statistic::new()), Some(Duration::milliseconds(50)));

        let (_, result_rx) = tracker.begin("test", &Link::Local, "test payload");

        thread::sleep(::std::time::Duration::from_millis(100));

        assert_eq!(Err(Error::Timeout), result_rx.recv().unwrap());
        assert_eq!(0, tracker.len());

        let (id, result_rx) = tracker.begin("test", &Link::Local, "test payload");

        thread::sleep(::std::time::Duration::from_millis(10));

        assert!(tracker.end(id, |payload| {
            assert_eq!("test payload", payload);
            "test result"
        }));

        assert_eq!(Ok("test result"), result_rx.recv().unwrap());
        assert_eq!(0, tracker.len());
    }

    #[test]
    fn request_end_after_timeout() {
        let tracker = Tracker::new(Arc::new(Statistic::new()), Some(Duration::milliseconds(50)));

        let (id, result_rx) = tracker.begin("test", &Link::Local, "test payload");

        thread::sleep(::std::time::Duration::from_millis(100));

        assert_eq!(Err(Error::Timeout), result_rx.recv().unwrap());
        assert_eq!(0, tracker.len());

        assert!(!tracker.end(id, |payload| {
            assert_eq!("test payload", payload);
            "test result"
        }));
    }

    #[test]
    fn concurrent_request_tracking() {
        let tracker = Arc::new(Tracker::new(Arc::new(Statistic::new()), None));

        let mut threads = Vec::new();
        for _ in 0..10 {
            let tracker = tracker.clone();
            threads.push(thread::spawn(move || {
                let (id, result_rx) = tracker.begin("test", &Link::Local, "test payload");
                thread::sleep(::std::time::Duration::from_millis(100));
                assert!(tracker.end(id, |payload| {
                    assert_eq!("test payload", payload);
                    "test result"
                }));
                result_rx.recv().unwrap()
            }));
        }

        for thread in threads {
            assert_eq!(Ok("test result"), thread.join().unwrap());
        }

        assert_eq!(0, tracker.len());
    }

    #[test]
    fn concurrent_request_timeout() {
        let tracker = Arc::new(Tracker::new(Arc::new(Statistic::new()),
                                            Some(Duration::milliseconds(50))));

        let mut threads = Vec::new();
        for _ in 0..10 {
            let tracker = tracker.clone();
            threads.push(thread::spawn(move || {
                let (id, result_rx) = tracker.begin("test", &Link::Local, "test payload");
                thread::sleep(::std::time::Duration::from_millis(100));
                assert!(!tracker.end(id, |_| "test result"));
                result_rx.recv().unwrap()
            }));
        }

        for thread in threads {
            assert_eq!(Err(Error::Timeout), thread.join().unwrap());
        }

        assert_eq!(0, tracker.len());
    }
}
