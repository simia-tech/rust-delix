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

use super::write_size;

pub struct Chunk<T>
    where T: io::Write
{
    parent: T,
    finish_on_drop: bool,
}

impl<T> Chunk<T> where T: io::Write
{
    pub fn new(parent: T, finish_on_drop: bool) -> Self {
        Chunk {
            parent: parent,
            finish_on_drop: finish_on_drop,
        }
    }

    pub fn get_ref(&self) -> &T {
        &self.parent
    }

    pub fn finish(mut self) -> io::Result<()> {
        Ok(try!(write_size(&mut self.parent, 0)))
    }
}

impl<T> io::Write for Chunk<T> where T: io::Write
{
    fn write(&mut self, buffer: &[u8]) -> io::Result<usize> {
        try!(write_size(&mut self.parent, buffer.len()));
        self.parent.write(buffer)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.parent.flush()
    }
}

impl<T> Drop for Chunk<T> where T: io::Write
{
    fn drop(&mut self) {
        if self.finish_on_drop {
            write_size(&mut self.parent, 0).unwrap();
        }
    }
}


#[cfg(test)]
mod tests {

    use std::io::{self, Write};
    use super::Chunk;

    #[test]
    fn write() {
        let mut result = Vec::new();

        {
            let mut writer = Chunk::new(&mut result, false);
            assert!(write!(writer, "test").is_ok());
            assert!(writer.finish().is_ok());
        }

        assert_eq!(vec![0, 0, 0, 0, 0, 0, 0, 4, 116, 101, 115, 116, 0, 0, 0, 0, 0, 0, 0, 0],
                   result);
    }

    #[test]
    fn copy() {
        let mut result = Vec::new();

        {
            let mut writer = Chunk::new(&mut result, true);
            assert_eq!(4,
                       io::copy(&mut io::Cursor::new(b"test".to_vec()), &mut writer).unwrap());
        }

        assert_eq!(vec![0, 0, 0, 0, 0, 0, 0, 4, 116, 101, 115, 116, 0, 0, 0, 0, 0, 0, 0, 0],
                   result);
    }

}
