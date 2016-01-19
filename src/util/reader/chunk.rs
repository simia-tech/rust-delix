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

use byteorder::{self, ReadBytesExt};

pub struct Chunk<T> {
    parent: T,
    buffer: io::Cursor<Vec<u8>>,
}

impl<T> Chunk<T> where T: io::Read
{
    pub fn new(parent: T) -> Self {
        Chunk {
            parent: parent,
            buffer: io::Cursor::new(Vec::new()),
        }
    }

    pub fn get_ref(&self) -> &T {
        &self.parent
    }
}

impl<T> io::Read for Chunk<T> where T: io::Read
{
    fn read(&mut self, buffer: &mut [u8]) -> io::Result<usize> {
        if self.buffer.position() as usize >= self.buffer.get_ref().len() {
            let size = try!(self.parent.read_u64::<byteorder::BigEndian>()) as usize;
            if size == 0 {
                return Ok(0);
            }

            let mut bytes = Vec::with_capacity(size);
            unsafe {
                bytes.set_len(size);
            }
            assert_eq!(size, try!(self.parent.read(&mut bytes)));

            self.buffer = io::Cursor::new(bytes);
        }

        self.buffer.read(buffer)
    }
}

#[cfg(test)]
mod tests {

    use std::io::{self, Read};
    use super::Chunk;

    #[test]
    fn read() {
        let mut reader = Chunk::new(io::Cursor::new(vec![0, 0, 0, 0, 0, 0, 0, 4, 116, 101, 115,
                                                         116, 0, 0, 0, 0, 0, 0, 0, 0]));
        let mut buffer = Vec::new();
        reader.read_to_end(&mut buffer).unwrap();
        assert_eq!("test", String::from_utf8_lossy(&buffer));
    }

}
