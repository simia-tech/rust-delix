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

use std::thread;
use std::sync::{Arc, RwLock};

pub struct Bound {
    running: Arc<RwLock<bool>>,
    join_handle: Option<thread::JoinHandle<()>>,
}

impl Bound {
    pub fn spawn<F>(f: F) -> Self
        where F: FnOnce(Arc<RwLock<bool>>) + Send + 'static
    {
        let running = Arc::new(RwLock::new(true));
        let running_clone = running.clone();

        let join_handle = thread::spawn(move || {
            f(running_clone);
        });

        Bound {
            running: running,
            join_handle: Some(join_handle),
        }
    }

    pub fn shutdown(&self) {
        *self.running.write().unwrap() = false;
    }
}

impl Drop for Bound {
    fn drop(&mut self) {
        self.shutdown();
        if let Some(join_handle) = self.join_handle.take() {
            join_handle.join().unwrap();
        }
    }
}

#[cfg(test)]
mod tests {

    use std::sync::mpsc;
    use std::time::Duration;
    use std::thread;
    use super::Bound;

    #[test]
    fn drop() {
        let (tx, rx) = mpsc::channel();
        {
            Bound::spawn(move |running| {
                while *running.read().unwrap() {
                    thread::sleep(Duration::from_millis(50));
                }
                tx.send(true).unwrap();
            });
        }
        assert!(rx.recv().unwrap());
    }

}
