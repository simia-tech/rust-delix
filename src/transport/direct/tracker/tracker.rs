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

use std::fmt;
use std::result;
use std::sync::{Arc, mpsc};
use std::sync::atomic;
use std::thread;

use time::{self, Duration};

use node::request;
use transport::direct::Link;
use transport::direct::tracker::{Statistic, Store, Subject, store};

const TIMEOUT_TOLERANCE_MS: i64 = 2;

pub struct Tracker {
    store: Arc<Store<(Option<Box<request::ResponseWriter>>,
                      mpsc::Sender<request::Response>)>>,
    statistic: Arc<Statistic>,
    current_id: atomic::AtomicUsize,
    join_handle_and_running_tx: Option<(thread::JoinHandle<()>, mpsc::Sender<bool>)>,
}

pub type Result<T> = result::Result<T, Error>;

#[derive(Debug, PartialEq)]
pub enum Error {
    AlreadyEnded,
    Store(store::Error),
}

impl Tracker {
    pub fn new(statistic: Arc<Statistic>, timeout: Option<Duration>) -> Tracker {
        let store = Arc::new(Store::new());
        statistic.assign_store(store.clone());

        let store_clone = store.clone();
        let join_handle_and_running_tx = timeout.map(|timeout| {
            let (running_tx, running_rx) = mpsc::channel();
            (thread::spawn(move || {
                while running_rx.recv().unwrap() {
                    loop {
                        let now = time::now_utc();

                        let (removed, next_at) = store_clone.remove_all_started_before(now -
                                                                                       timeout);
                        for (_, (_, response_tx)) in removed {
                            response_tx.send(Err(request::Error::Timeout)).unwrap();
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
             running_tx)
        });

        Tracker {
            store: store,
            statistic: statistic,
            current_id: atomic::AtomicUsize::new(0),
            join_handle_and_running_tx: join_handle_and_running_tx,
        }
    }

    pub fn begin(&self,
                 name: &str,
                 link: &Link,
                 response_writer: Box<request::ResponseWriter>)
                 -> (u32, mpsc::Receiver<request::Response>) {
        let (response_tx, response_rx) = mpsc::channel();
        let id = self.current_id.fetch_add(1, atomic::Ordering::SeqCst) as u32;
        let subject = Subject::from_name_and_link(name, link);
        let started_at = time::now_utc();

        if self.store
               .insert(id,
                       subject,
                       started_at,
                       (Some(response_writer), response_tx))
               .unwrap() {
            if let Some((_, ref running_tx)) = self.join_handle_and_running_tx {
                running_tx.send(true).unwrap();
            }
        }

        (id, response_rx)
    }

    pub fn take_response_writer(&self, id: u32) -> Option<Box<request::ResponseWriter>> {
        let mut result = None;
        self.store.get_mut(&id, |entry| {
            result = entry.0.take();
        });
        result
    }

    pub fn end(&self, id: u32, response: request::Response) -> Result<()> {
        let (subject, started_at, (_, response_tx)) = try!(self.store.remove(&id));

        response_tx.send(response).unwrap();

        self.statistic.push(subject, time::now_utc() - started_at);

        Ok(())
    }

    pub fn len(&self) -> usize {
        self.store.len()
    }
}

impl fmt::Debug for Tracker {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "tracker({} entries)", self.len())
    }
}

impl Drop for Tracker {
    fn drop(&mut self) {
        if let Some((join_handle, running_tx)) = self.join_handle_and_running_tx.take() {
            running_tx.send(false).unwrap();
            join_handle.join().unwrap();
        }
    }
}

unsafe impl Send for Tracker {}

unsafe impl Sync for Tracker {}

impl From<store::Error> for Error {
    fn from(error: store::Error) -> Self {
        match error {
            store::Error::IdDoesNotExists => Error::AlreadyEnded,
            _ => Error::Store(error),
        }
    }
}

#[cfg(test)]
mod tests {

    use std::io;
    use std::thread;
    use std::sync::Arc;
    use time::Duration;
    use node::request;
    use super::{Error, Tracker};
    use super::super::Statistic;
    use super::super::super::Link;

    #[test]
    fn request_tracking() {
        let tracker = Tracker::new(Arc::new(Statistic::new()), None);

        let (id, response_rx) = tracker.begin("test", &Link::Local, Box::new(io::sink()));
        tracker.end(id, Ok(Box::new(io::Cursor::new(b"test".to_vec())))).unwrap();

        let mut response = response_rx.recv().unwrap().unwrap();
        let mut response_bytes = Vec::new();
        response.read_to_end(&mut response_bytes).unwrap();

        assert_eq!(b"test".to_vec(), response_bytes);
        assert_eq!(0, tracker.len());
    }

    #[test]
    fn request_timeout() {
        let tracker = Tracker::new(Arc::new(Statistic::new()), Some(Duration::milliseconds(50)));

        let (_, response_rx) = tracker.begin("test", &Link::Local, Box::new(io::sink()));
        assert_eq!(1, tracker.len());
        thread::sleep(::std::time::Duration::from_millis(100));

        assert_eq!(Some(request::Error::Timeout),
                   response_rx.recv().unwrap().err());
        assert_eq!(0, tracker.len());

        let (request_id, response_rx) = tracker.begin("test", &Link::Local, Box::new(io::sink()));
        assert_eq!(1, tracker.len());
        thread::sleep(::std::time::Duration::from_millis(10));
        tracker.end(request_id, Ok(Box::new(io::Cursor::new(b"test".to_vec())))).unwrap();

        let mut response = response_rx.recv().unwrap().unwrap();
        let mut response_bytes = Vec::new();
        response.read_to_end(&mut response_bytes).unwrap();

        assert_eq!(b"test".to_vec(), response_bytes);
        assert_eq!(0, tracker.len());
    }

    #[test]
    fn request_end_after_timeout() {
        let tracker = Tracker::new(Arc::new(Statistic::new()), Some(Duration::milliseconds(50)));

        let (id, response_rx) = tracker.begin("test", &Link::Local, Box::new(io::sink()));
        assert_eq!(1, tracker.len());
        thread::sleep(::std::time::Duration::from_millis(100));

        assert_eq!(Some(request::Error::Timeout),
                   response_rx.recv().unwrap().err());
        assert_eq!(0, tracker.len());

        assert_eq!(Err(Error::AlreadyEnded),
                   tracker.end(id, Ok(Box::new(io::Cursor::new(b"test".to_vec())))));
    }

    #[test]
    fn concurrent_request_tracking() {
        let tracker = Arc::new(Tracker::new(Arc::new(Statistic::new()), None));

        let mut threads = Vec::new();
        for _ in 0..10 {
            let tracker = tracker.clone();
            threads.push(thread::spawn(move || -> request::Response {
                let (id, response_rx) = tracker.begin("test", &Link::Local, Box::new(io::sink()));
                thread::sleep(::std::time::Duration::from_millis(100));
                tracker.end(id, Ok(Box::new(io::Cursor::new(b"test".to_vec())))).unwrap();
                response_rx.recv().unwrap()
            }));
        }

        for thread in threads {
            let mut response = thread.join().unwrap().unwrap();
            let mut response_bytes = Vec::new();
            response.read_to_end(&mut response_bytes).unwrap();
            assert_eq!(b"test".to_vec(), response_bytes);
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
            threads.push(thread::spawn(move || -> request::Response {
                let (id, response_rx) = tracker.begin("test", &Link::Local, Box::new(io::sink()));
                thread::sleep(::std::time::Duration::from_millis(100));
                assert_eq!(Err(Error::AlreadyEnded),
                           tracker.end(id, Ok(Box::new(io::Cursor::new(b"test".to_vec())))));
                response_rx.recv().unwrap()
            }));
        }

        for thread in threads {
            assert_eq!(Some(request::Error::Timeout), thread.join().unwrap().err());
        }

        assert_eq!(0, tracker.len());
    }

}
