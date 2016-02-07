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
use std::iter;

use super::read_size;

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
            let size = try!(read_size(&mut self.parent));
            if size == 0 {
                return Ok(0);
            }

            let mut bytes = iter::repeat(0u8).take(size).collect::<Vec<u8>>();
            try!(self.parent.read_exact(&mut bytes));

            self.buffer = io::Cursor::new(bytes);
        }

        self.buffer.read(buffer)
    }
}

#[cfg(test)]
mod tests {

    use std::io;
    use super::Chunk;
    use super::super::ErrorAfter;

    #[test]
    fn read() {
        let source = io::Cursor::new(vec![0, 0, 0, 0, 0, 0, 0, 4, 116, 101, 115, 116, 0, 0, 0, 0,
                                          0, 0, 0, 0]);
        let mut reader = Chunk::new(source);
        let mut target = Vec::new();
        assert!(io::copy(&mut reader, &mut target).is_ok());
        assert_eq!("test", String::from_utf8_lossy(&target));
    }

    #[test]
    fn read_unexpected_eof() {
        let source = io::Cursor::new(vec![0, 0, 0, 0, 0, 0, 0, 4, 116, 101, 115, 116, 0, 0, 0, 0,
                                          0, 0, 0, 0]);
        let mut reader = Chunk::new(ErrorAfter::new_unexpected_eof(source, 2));
        let mut target = Vec::new();
        let result = io::copy(&mut reader, &mut target);
        assert!(result.is_err());
        if let Err(error) = result {
            assert_eq!(io::ErrorKind::UnexpectedEof, error.kind());
        }
    }

}
