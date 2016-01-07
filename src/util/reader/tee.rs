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

pub struct Tee<R> {
    reader: R,
    buffer: Option<Vec<u8>>,
}

impl<R: io::Read> Tee<R> {
    pub fn new(reader: R) -> Tee<R> {
        Tee {
            reader: reader,
            buffer: Some(Vec::new()),
        }
    }

    pub fn take_buffer(&mut self) -> Vec<u8> {
        let buffer = self.buffer.take().unwrap();
        self.buffer = Some(Vec::new());
        buffer
    }
}

impl<R: io::Read> io::Read for Tee<R> {
    fn read(&mut self, b: &mut [u8]) -> io::Result<usize> {
        let result = self.reader.read(b);
        if let Ok(count) = result {
            if let Some(ref mut buffer) = self.buffer {
                buffer.append(&mut b[0..count].to_vec());
            }
        }
        result
    }
}
