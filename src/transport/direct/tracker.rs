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

use std::collections::HashMap;
use std::fmt;
use std::result;
use std::sync::{RwLock, mpsc};
use std::sync::atomic;

pub struct Tracker<T> {
    entries: RwLock<HashMap<u32, mpsc::Sender<T>>>,
    current_id: atomic::AtomicUsize,
}

pub type Result<T> = result::Result<T, Error>;

#[derive(Debug, PartialEq)]
pub enum Error {
    InvalidTrackId,
}

impl<T> Tracker<T> {
    pub fn new() -> Tracker<T> {
        Tracker {
            entries: RwLock::new(HashMap::new()),
            current_id: atomic::AtomicUsize::new(0),
        }
    }

    pub fn begin(&self) -> (u32, mpsc::Receiver<T>) {
        let mut entries = self.entries.write().unwrap();
        let (sender, receiver) = mpsc::channel();
        let id = self.current_id.fetch_add(1, atomic::Ordering::SeqCst) as u32;
        entries.insert(id, sender);
        (id, receiver)
    }

    pub fn end(&self, id: u32, result: T) -> Result<()> {
        let mut entries = self.entries.write().unwrap();
        match entries.remove(&id) {
            Some(ref sender) => {
                sender.send(result).unwrap();
                Ok(())
            }
            None => Err(Error::InvalidTrackId),
        }
    }

    pub fn len(&self) -> usize {
        self.entries.read().unwrap().len()
    }
}

impl<T> fmt::Display for Tracker<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "(Direct tracker {} entries)", self.len())
    }
}

unsafe impl<T> Send for Tracker<T> {}

unsafe impl<T> Sync for Tracker<T> {}

#[cfg(test)]
mod tests {

    use std::thread;
    use std::sync::Arc;
    use super::{Result, Tracker};

    #[test]
    fn request_tracking() {
        let tracker: Tracker<Result<&str>> = Tracker::new();

        let (id, result_chan) = tracker.begin();
        tracker.end(id, Ok("test")).unwrap();

        assert_eq!(Ok("test"), result_chan.recv().unwrap());
        assert_eq!(0, tracker.len());
    }

    #[test]
    fn concurrent_request_tracking() {
        let tracker: Arc<Tracker<Result<&str>>> = Arc::new(Tracker::new());

        let mut threads = Vec::new();
        for _ in 0..10 {
            let tracker = tracker.clone();
            threads.push(thread::spawn(move || -> Result<&str> {
                let (id, result_channel) = tracker.begin();
                thread::sleep_ms(100);
                tracker.end(id, Ok("test")).unwrap();
                result_channel.recv().unwrap()
            }));
        }

        for thread in threads {
            assert_eq!(Ok("test"), thread.join().unwrap());
        }

        assert_eq!(0, tracker.len());
    }

}
