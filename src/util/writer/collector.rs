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

use std::io;
use std::result;
use std::sync::{Arc, RwLock};

pub struct Collector {
    buffer: Arc<RwLock<Vec<u8>>>,
}

pub type Result<T> = result::Result<T, Error>;

#[derive(Clone, Debug, PartialEq)]
pub enum Error {
    TooManyReferences,
}

impl Collector {
    pub fn new() -> Collector {
        Collector { buffer: Arc::new(RwLock::new(Vec::new())) }
    }

    pub fn vec(self) -> Result<Vec<u8>> {
        let buffer_mutex = match Arc::try_unwrap(self.buffer) {
            Ok(bm) => bm,
            Err(_) => return Err(Error::TooManyReferences),
        };
        Ok(buffer_mutex.into_inner().unwrap())
    }
}

impl io::Write for Collector {
    fn write(&mut self, buffer: &[u8]) -> io::Result<usize> {
        Ok(try!(self.buffer.write().unwrap().write(buffer)))
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl Clone for Collector {
    fn clone(&self) -> Self {
        Collector { buffer: self.buffer.clone() }
    }
}

#[cfg(test)]
mod tests {

    use std::io::Write;
    use std::thread;
    use super::Collector;

    #[test]
    fn write_from_no_instance() {
        let collector = Collector::new();
        assert_eq!(0, collector.vec().unwrap().len());
    }

    #[test]
    fn write_from_single_instance() {
        let collector = Collector::new();

        {
            let mut collector_clone = collector.clone();
            thread::spawn(move || {
                write!(collector_clone, "test").unwrap();
            })
                .join()
                .unwrap();
        }

        assert_eq!("test", String::from_utf8_lossy(&collector.vec().unwrap()));
    }

    #[test]
    fn write_from_multiple_instances() {
        let collector = Collector::new();

        {
            let mut collector_clone = collector.clone();
            thread::spawn(move || {
                write!(collector_clone, "test").unwrap();
            })
                .join()
                .unwrap();
        }

        {
            let mut collector_clone = collector.clone();
            thread::spawn(move || {
                write!(collector_clone, "test").unwrap();
            })
                .join()
                .unwrap();
        }

        assert_eq!("testtest",
                   String::from_utf8_lossy(&collector.vec().unwrap()));
    }

}
