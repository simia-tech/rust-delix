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
use std::sync::mpsc;

pub struct Tracker<T> {
    entries: HashMap<u32, mpsc::Sender<T>>,
    current_id: u32,
}

pub type Result<T> = result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    InvalidTrackId,
}

impl<T> Tracker<T> {
    pub fn new() -> Tracker<T> {
        Tracker {
            entries: HashMap::new(),
            current_id: 0,
        }
    }

    pub fn begin(&mut self) -> (u32, mpsc::Receiver<T>) {
        let (sender, receiver) = mpsc::channel();
        let id = self.current_id;
        self.current_id.wrapping_add(1);
        self.entries.insert(id, sender);
        (id, receiver)
    }

    pub fn end(&mut self, id: u32, result: T) -> Result<()> {
        match self.entries.remove(&id) {
            Some(ref sender) => {
                sender.send(result).unwrap();
                Ok(())
            }
            None => Err(Error::InvalidTrackId),
        }
    }

    pub fn len(&self) -> usize {
        self.entries.len()
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

    use super::Tracker;

    #[test]
    fn request_tracking() {
        let mut tracker: Tracker<Result<&str, &str>> = Tracker::new();

        let (id, result_chan) = tracker.begin();
        tracker.end(id, Ok("test")).unwrap();

        assert_eq!(Ok("test"), result_chan.recv().unwrap());
        assert_eq!(0, tracker.len());
    }

}
