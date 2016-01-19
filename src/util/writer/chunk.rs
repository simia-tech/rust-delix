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

use std::io::{self, Write};

use byteorder::{self, WriteBytesExt};

pub struct Chunk<T>
    where T: io::Write
{
    parent: T,
}

impl<T> Chunk<T> where T: io::Write
{
    pub fn new(parent: T) -> Self {
        Chunk { parent: parent }
    }

    pub fn get_ref(&self) -> &T {
        &self.parent
    }
}

impl<T> io::Write for Chunk<T> where T: io::Write
{
    fn write(&mut self, buffer: &[u8]) -> io::Result<usize> {
        debug!("write chunk {}", buffer.len());
        self.parent.write_u64::<byteorder::BigEndian>(buffer.len() as u64).unwrap();
        Ok(self.parent.write(buffer).unwrap())
    }

    fn flush(&mut self) -> io::Result<()> {
        debug!("flush chunk");
        self.parent.flush()
    }
}

impl<T> Drop for Chunk<T> where T: io::Write
{
    fn drop(&mut self) {
        self.write(&[]).unwrap();
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
            let mut writer = Chunk::new(&mut result);
            write!(writer, "test").unwrap();
        }
        assert_eq!(vec![0, 0, 0, 0, 0, 0, 0, 4, 116, 101, 115, 116, 0, 0, 0, 0, 0, 0, 0, 0],
                   result);
    }

    #[test]
    fn copy() {
        let mut result = Vec::new();
        assert_eq!(4,
                   io::copy(&mut io::Cursor::new(b"test".to_vec()),
                            &mut Chunk::new(&mut result))
                       .unwrap());
        assert_eq!(vec![0, 0, 0, 0, 0, 0, 0, 4, 116, 101, 115, 116, 0, 0, 0, 0, 0, 0, 0, 0],
                   result);
    }

}
