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
use std::io;
use std::result;
use std::sync::{Mutex, RwLock, mpsc};

use super::packet;

pub struct Dispatcher {
    entries: RwLock<HashMap<u32, Mutex<mpsc::Sender<io::Result<Vec<u8>>>>>>,
}

pub type Result<T> = result::Result<T, Error>;

#[derive(Debug)]
pub enum Error { }

impl Dispatcher {
    pub fn new() -> Self {
        Dispatcher { entries: RwLock::new(HashMap::new()) }
    }

    pub fn begin(&self, id: u32) -> Box<io::Read + Send> {
        let mut entries = self.entries.write().unwrap();

        let (tx, reader) = packet::Reader::new();

        entries.insert(id, Mutex::new(tx));

        Box::new(reader)
    }

    pub fn dispatch(&self, id: u32, result: io::Result<Vec<u8>>) -> Result<()> {
        let mut entries = self.entries.write().unwrap();

        let mut remove = match result {
            Ok(ref payload) if payload.len() == 0 => true,
            Err(_) => true,
            _ => false,
        };

        if let Some(ref entry) = entries.get(&id) {
            if let Err(_) = entry.lock().unwrap().send(result) {
                remove = true;
            }
        }

        if remove {
            entries.remove(&id);
        }

        Ok(())
    }

    #[cfg(test)]
    pub fn len(&self) -> usize {
        self.entries.read().unwrap().len()
    }
}

#[cfg(test)]
mod tests {

    use std::error::Error;
    use std::io;
    use std::sync::Arc;
    use std::thread;
    use super::Dispatcher;

    #[test]
    fn dispatch_of_a_message() {
        let dispatcher = Arc::new(Dispatcher::new());
        let dispatcher_clone = dispatcher.clone();

        let mut reader = dispatcher.begin(1);
        assert_eq!(1, dispatcher.len());

        thread::spawn(move || {
            assert!(dispatcher_clone.dispatch(1, Ok(b"test message".to_vec())).is_ok());
            assert!(dispatcher_clone.dispatch(1, Ok(b"".to_vec())).is_ok());
        });

        let mut output = Vec::new();
        assert_eq!(Some(12), io::copy(&mut reader, &mut output).ok());
        assert_eq!("test message", String::from_utf8_lossy(&output));

        assert_eq!(0, dispatcher.len());
    }

    #[test]
    fn dispatch_of_an_error() {
        let dispatcher = Arc::new(Dispatcher::new());
        let dispatcher_clone = dispatcher.clone();

        let mut reader = dispatcher.begin(1);
        assert_eq!(1, dispatcher.len());

        thread::spawn(move || {
            assert!(dispatcher_clone.dispatch(1,
                                              Err(io::Error::new(io::ErrorKind::UnexpectedEof,
                                                                 "unexpected EOF")))
                                    .is_ok());
        });

        let result = io::copy(&mut reader, &mut io::sink()).unwrap_err();
        assert_eq!(io::ErrorKind::UnexpectedEof, result.kind());
        assert_eq!("unexpected EOF", result.description());

        assert_eq!(0, dispatcher.len());
    }

}
