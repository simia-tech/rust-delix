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
use transport::direct::tracker::{Store, store};

const TIMEOUT_TOLERANCE_MS: i64 = 2;

pub struct Tracker {
    store: Arc<Store>,
    current_id: atomic::AtomicUsize,
    join_handle_and_running_tx: Option<(thread::JoinHandle<()>, mpsc::Sender<bool>)>,
}

pub type Result<T> = result::Result<T, Error>;

#[derive(Debug, PartialEq)]
pub enum Error {
    Store(store::Error),
}

impl Tracker {
    pub fn new(timeout: Option<Duration>) -> Tracker {
        let store = Arc::new(Store::new());
        let store_clone = store.clone();

        let join_handle_and_running_tx = timeout.map(|timeout| {
            let (running_tx, running_rx) = mpsc::channel();
            (thread::spawn(move || {
                while running_rx.recv().unwrap() {
                    loop {
                        let now = time::now_utc();

                        let (removed, next_at) = store_clone.remove_all_started_before(now -
                                                                                       timeout);
                        for (_, response_tx) in removed {
                            response_tx.send(Err(request::Error::Timeout)).unwrap();
                        }

                        if next_at.is_none() {
                            break;
                        }
                        let wait_for = next_at.unwrap() - (now - timeout) +
                                       Duration::milliseconds(TIMEOUT_TOLERANCE_MS);
                        thread::sleep_ms(wait_for.num_milliseconds() as u32);
                    }
                }
            }),
             running_tx)
        });

        Tracker {
            store: store,
            current_id: atomic::AtomicUsize::new(0),
            join_handle_and_running_tx: join_handle_and_running_tx,
        }
    }

    pub fn begin(&self) -> (u32, mpsc::Receiver<request::Response>) {
        let (result_sender, result_receiver) = mpsc::channel();
        let id = self.current_id.fetch_add(1, atomic::Ordering::SeqCst) as u32;

        if self.store.insert(id, result_sender, time::now_utc()).ok().unwrap() {
            if let Some((_, ref running_tx)) = self.join_handle_and_running_tx {
                running_tx.send(true).unwrap();
            }
        }

        (id, result_receiver)
    }

    pub fn end(&self, id: u32, response: request::Response) -> Result<()> {
        let (result_sender, _) = try!(self.store.remove(&id));
        result_sender.send(response).unwrap();
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
        Error::Store(error)
    }
}

#[cfg(test)]
mod tests {

    use std::thread;
    use std::sync::Arc;
    use time::Duration;
    use node::request;
    use super::{Error, Tracker};
    use super::super::store;

    #[test]
    fn request_tracking() {
        let tracker = Tracker::new(None);

        let (id, result_channel) = tracker.begin();
        tracker.end(id, Ok(b"test".to_vec())).unwrap();

        assert_eq!(Ok(b"test".to_vec()), result_channel.recv().unwrap());
        assert_eq!(0, tracker.len());
    }

    #[test]
    fn request_timeout() {
        let tracker = Tracker::new(Some(Duration::milliseconds(50)));

        let (_, result_channel) = tracker.begin();
        assert_eq!(1, tracker.len());
        thread::sleep_ms(100);

        assert_eq!(Err(request::Error::Timeout), result_channel.recv().unwrap());
        assert_eq!(0, tracker.len());

        let (request_id, result_channel) = tracker.begin();
        assert_eq!(1, tracker.len());
        thread::sleep_ms(10);
        tracker.end(request_id, Ok(b"test".to_vec())).unwrap();

        assert_eq!(Ok(b"test".to_vec()), result_channel.recv().unwrap());
        assert_eq!(0, tracker.len());
    }

    #[test]
    fn concurrent_request_tracking() {
        let tracker = Arc::new(Tracker::new(None));

        let mut threads = Vec::new();
        for _ in 0..10 {
            let tracker = tracker.clone();
            threads.push(thread::spawn(move || -> request::Response {
                let (id, result_channel) = tracker.begin();
                thread::sleep_ms(100);
                tracker.end(id, Ok(b"test".to_vec())).unwrap();
                result_channel.recv().unwrap()
            }));
        }

        for thread in threads {
            assert_eq!(Ok(b"test".to_vec()), thread.join().unwrap());
        }

        assert_eq!(0, tracker.len());
    }

    #[test]
    fn concurrent_request_timeout() {
        let tracker = Arc::new(Tracker::new(Some(Duration::milliseconds(50))));

        let mut threads = Vec::new();
        for _ in 0..10 {
            let tracker = tracker.clone();
            threads.push(thread::spawn(move || -> request::Response {
                let (id, result_channel) = tracker.begin();
                thread::sleep_ms(100);
                assert_eq!(Err(Error::Store(store::Error::IdDoesNotExists)),
                           tracker.end(id, Ok(b"test".to_vec())));
                result_channel.recv().unwrap()
            }));
        }

        for thread in threads {
            assert_eq!(Err(request::Error::Timeout), thread.join().unwrap());
        }

        assert_eq!(0, tracker.len());
    }

}
