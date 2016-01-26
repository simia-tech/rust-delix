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
use std::sync::RwLock;

use transport::direct::tracker::Subject;

use time;

pub struct Store<T> {
    entries: RwLock<HashMap<u32, (Subject, time::Tm, T)>>,
}

pub type Result<T> = result::Result<T, Error>;

#[derive(Debug, PartialEq)]
pub enum Error {
    IdAlreadyExists,
    IdDoesNotExists,
}

impl<T> Store<T> {
    pub fn new() -> Store<T> {
        Store { entries: RwLock::new(HashMap::new()) }
    }

    pub fn insert(&self,
                  id: u32,
                  subject: Subject,
                  started_at: time::Tm,
                  entry: T)
                  -> Result<bool> {

        let mut entries = self.entries.write().unwrap();
        if entries.contains_key(&id) {
            return Err(Error::IdAlreadyExists);
        }
        entries.insert(id, (subject, started_at, entry));
        Ok(entries.len() == 1)
    }

    pub fn get_mut<F: FnMut(&mut T)>(&self, id: &u32, mut f: F) {
        let mut entries = self.entries.write().unwrap();
        if let Some(ref mut entry) = entries.get_mut(id).map(|value| &mut value.2) {
            f(entry);
        }
    }

    pub fn remove(&self, id: &u32) -> Result<(Subject, time::Tm, T)> {
        let mut entries = self.entries.write().unwrap();
        if !entries.contains_key(&id) {
            return Err(Error::IdDoesNotExists);
        }
        if let Some(tuple) = entries.remove(id) {
            return Ok(tuple);
        }
        Err(Error::IdDoesNotExists)
    }

    pub fn remove_all_started_before(&self,
                                     threshold: time::Tm)
                                     -> (Vec<(u32, T)>, Option<time::Tm>) {

        let mut entries = self.entries.write().unwrap();

        let mut to_remove = Vec::new();
        let mut next_at = None;
        for (&id, &(_, started_at, _)) in entries.iter() {
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
            let (_, _, entry) = entries.remove(&id).unwrap();
            result.push((id, entry));
        }

        (result, next_at)
    }

    pub fn started_ats_with_subject<F: FnMut(&[&time::Tm])>(&self, subject: &Subject, mut f: F) {
        let entries = self.entries.read().unwrap();
        f(&entries.iter()
                  .filter_map(|(_, &(ref entry_subject, ref started_at, _))| {
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

unsafe impl<T> Send for Store<T> {}

unsafe impl<T> Sync for Store<T> {}

#[cfg(test)]
mod tests {

    use time;
    use super::{Error, Store};
    use super::super::Subject;

    #[test]
    fn insert() {
        let store = Store::new();

        let started_at = build_time(100);
        store.insert(0, Subject::local("test"), started_at, "test entry")
             .unwrap();

        assert_eq!(1, store.len());

        assert_eq!(Err(Error::IdAlreadyExists),
                   store.insert(0, Subject::local("test"), time::now_utc(), "test entry"));
    }

    #[test]
    fn remove() {
        let store = Store::new();
        store.insert(0, Subject::local("test"), build_time(100), "test entry")
             .unwrap();

        let (removed_subject, removed_started_at, removed_entry) = store.remove(&0).unwrap();

        assert_eq!(Subject::local("test"), removed_subject);
        assert_eq!(build_time(100), removed_started_at);
        assert_eq!("test entry", removed_entry);
        assert_eq!(0, store.len());
        assert_eq!(Some(Error::IdDoesNotExists), store.remove(&0).err());
    }

    #[test]
    fn remove_all_started_before() {
        let store = Store::new();
        store.insert(10, Subject::local("test"), build_time(200), "test entry")
             .unwrap();
        store.insert(20, Subject::local("test"), build_time(100), "test entry")
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
        store.insert(10, Subject::local("one"), build_time(200), "test entry")
             .unwrap();
        store.insert(20, Subject::local("two"), build_time(100), "test entry")
             .unwrap();

        store.started_ats_with_subject(&Subject::local("two"), |started_ats| {
            assert_eq!(1, started_ats.len());
            assert_eq!(&build_time(100), started_ats[0]);
        });
    }

    fn build_time(seconds: i64) -> time::Tm {
        time::at(time::Timespec::new(seconds, 0))
    }

}
