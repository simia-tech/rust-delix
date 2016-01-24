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
use std::result;
use std::sync::{Arc, Mutex, RwLock, mpsc};

use node::request;
use transport::direct::tracker::Subject;

use time;

pub struct Store {
    entries: RwLock<HashMap<u32,
                            (Arc<Mutex<request::ResponseWriter>>,
                             mpsc::Sender<request::Response>,
                             Subject,
                             time::Tm)>>,
}

pub type Result<T> = result::Result<T, Error>;

#[derive(Debug, PartialEq)]
pub enum Error {
    IdAlreadyExists,
    IdDoesNotExists,
}

impl Store {
    pub fn new() -> Store {
        Store { entries: RwLock::new(HashMap::new()) }
    }

    pub fn insert(&self,
                  id: u32,
                  response_writer: Arc<Mutex<request::ResponseWriter>>,
                  response_tx: mpsc::Sender<request::Response>,
                  subject: Subject,
                  started_at: time::Tm)
                  -> Result<bool> {

        let mut entries = self.entries.write().unwrap();
        if entries.contains_key(&id) {
            return Err(Error::IdAlreadyExists);
        }
        entries.insert(id, (response_writer, response_tx, subject, started_at));
        Ok(entries.len() == 1)
    }

    pub fn get_response_writer(&self, id: &u32) -> Option<Arc<Mutex<request::ResponseWriter>>> {
        let entries = self.entries.read().unwrap();
        entries.get(id).map(|value| value.0.clone())
    }

    pub fn remove(&self, id: &u32) -> Result<(mpsc::Sender<request::Response>, Subject, time::Tm)> {
        let mut entries = self.entries.write().unwrap();
        if !entries.contains_key(&id) {
            return Err(Error::IdDoesNotExists);
        }
        if let Some(tuple) = entries.remove(id) {
            return Ok((tuple.1, tuple.2, tuple.3));
        }
        Err(Error::IdDoesNotExists)
    }

    pub fn remove_all_started_before(&self,
                                     threshold: time::Tm)
                                     -> (Vec<(u32, mpsc::Sender<request::Response>)>,
                                         Option<time::Tm>) {

        let mut entries = self.entries.write().unwrap();

        let mut to_remove = Vec::new();
        let mut next_at = None;
        for (&id, &(_, _, _, started_at)) in entries.iter() {
            if started_at < threshold {
                to_remove.push(id);
            } else {
                next_at = match next_at {
                    None => Some(started_at),
                    Some(next_at) if started_at < next_at => Some(started_at),
                    Some(next_at) => Some(next_at),
                }
            }
        }

        let mut result = Vec::new();
        for id in to_remove {
            let (_, result_tx, _, _) = entries.remove(&id).unwrap();
            result.push((id, result_tx));
        }

        (result, next_at)
    }

    pub fn started_ats_with_subject<F: FnMut(&[&time::Tm])>(&self, subject: &Subject, mut f: F) {
        let entries = self.entries.read().unwrap();
        f(&entries.iter()
                  .filter_map(|(_, &(_, _, ref entry_subject, ref started_at))| {
                      if entry_subject == subject {
                          Some(started_at)
                      } else {
                          None
                      }
                  })
                  .collect::<Vec<&time::Tm>>());
    }

    pub fn len(&self) -> usize {
        self.entries.read().unwrap().len()
    }
}

unsafe impl Send for Store {}

unsafe impl Sync for Store {}

#[cfg(test)]
mod tests {

    use std::io;
    use std::sync::{Arc, Mutex, mpsc};
    use time;
    use node::request;
    use super::{Error, Store};
    use super::super::Subject;

    #[test]
    fn insert() {
        let store = Store::new();

        let (response_tx, started_at) = build_tx_and_time(100);
        store.insert(0,
                     Arc::new(Mutex::new(io::sink())),
                     response_tx,
                     Subject::local("test"),
                     started_at)
             .unwrap();

        assert_eq!(1, store.len());

        let (response_tx, _) = mpsc::channel();
        assert_eq!(Err(Error::IdAlreadyExists),
                   store.insert(0,
                                Arc::new(Mutex::new(io::sink())),
                                response_tx,
                                Subject::local("test"),
                                time::now_utc()));
    }

    #[test]
    fn remove() {
        let store = Store::new();
        let (response_tx, started_at) = build_tx_and_time(100);
        store.insert(0,
                     Arc::new(Mutex::new(io::sink())),
                     response_tx,
                     Subject::local("test"),
                     started_at)
             .unwrap();

        let (_, _, removed_started_at) = store.remove(&0).unwrap();

        assert_eq!(started_at, removed_started_at);
        assert_eq!(0, store.len());
        assert_eq!(Some(Error::IdDoesNotExists), store.remove(&0).err());
    }

    #[test]
    fn remove_all_started_before() {
        let store = Store::new();
        let (response_tx, started_at) = build_tx_and_time(200);
        store.insert(10,
                     Arc::new(Mutex::new(io::sink())),
                     response_tx,
                     Subject::local("test"),
                     started_at)
             .unwrap();
        let (response_tx, started_at) = build_tx_and_time(100);
        store.insert(20,
                     Arc::new(Mutex::new(io::sink())),
                     response_tx,
                     Subject::local("test"),
                     started_at)
             .unwrap();

        let (removed, next_at) = store.remove_all_started_before(build_time(150));
        assert_eq!(1, removed.len());
        assert_eq!(20, removed[0].0);
        assert_eq!(Some(build_time(200)), next_at);

        let (removed, next_at) = store.remove_all_started_before(build_time(150));
        assert_eq!(0, removed.len());
        assert_eq!(Some(build_time(200)), next_at);

        let (removed, next_at) = store.remove_all_started_before(build_time(250));
        assert_eq!(1, removed.len());
        assert_eq!(10, removed[0].0);
        assert_eq!(None, next_at);

        assert_eq!(0, store.len());
    }

    #[test]
    fn started_ats_with_subject() {
        let store = Store::new();
        let (response_tx, started_at) = build_tx_and_time(200);
        store.insert(10,
                     Arc::new(Mutex::new(io::sink())),
                     response_tx,
                     Subject::local("one"),
                     started_at)
             .unwrap();
        let (response_tx, started_at) = build_tx_and_time(100);
        store.insert(20,
                     Arc::new(Mutex::new(io::sink())),
                     response_tx,
                     Subject::local("two"),
                     started_at)
             .unwrap();

        store.started_ats_with_subject(&Subject::local("two"), |started_ats| {
            assert_eq!(1, started_ats.len());
            assert_eq!(&started_at, started_ats[0]);
        });
    }

    fn build_tx_and_time(seconds: i64) -> (mpsc::Sender<request::Response>, time::Tm) {
        let (response_tx, _) = mpsc::channel();
        (response_tx, build_time(seconds))
    }

    fn build_time(seconds: i64) -> time::Tm {
        time::at(time::Timespec::new(seconds, 0))
    }

}
