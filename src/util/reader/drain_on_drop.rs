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

pub struct DrainOnDrop<R>
    where R: io::Read
{
    reader: R,
    drain: bool,
}

impl<R> DrainOnDrop<R> where R: io::Read
{
    pub fn new(reader: R) -> Self {
        DrainOnDrop {
            reader: reader,
            drain: true,
        }
    }
}

impl<R> io::Read for DrainOnDrop<R> where R: io::Read
{
    fn read(&mut self, buffer: &mut [u8]) -> io::Result<usize> {
        match self.reader.read(buffer) {
            Ok(0) => {
                self.drain = false;
                Ok(0)
            }
            Ok(value) => Ok(value),
            Err(error) => {
                self.drain = false;
                Err(error)
            }
        }
    }
}

impl<R> Drop for DrainOnDrop<R> where R: io::Read
{
    fn drop(&mut self) {
        if self.drain {
            let _ = io::copy(&mut self.reader, &mut io::sink());
        }
    }
}

#[cfg(test)]
mod tests {

    use std::io;
    use super::DrainOnDrop;
    use super::super::ErrorAfter;

    #[test]
    fn read() {
        let source = io::Cursor::new(b"test message".to_vec());
        let mut reader = DrainOnDrop::new(source);

        let mut target = Vec::new();
        io::copy(&mut reader, &mut target).unwrap();
        assert_eq!("test message", String::from_utf8_lossy(&target));
    }

    #[test]
    fn copy_leftover_to_sink_on_drop() {
        let mut source = io::Cursor::new(b"test message".to_vec());
        {
            DrainOnDrop::new(&mut source);
        }

        let mut target = Vec::new();
        io::copy(&mut source, &mut target).unwrap();
        assert_eq!("", String::from_utf8_lossy(&target));
    }

    #[test]
    fn read_unexpected_eof() {
        let source = io::Cursor::new(b"test message".to_vec());
        let mut reader = DrainOnDrop::new(ErrorAfter::new_unexpected_eof(source, 4));

        let mut target = Vec::new();
        let result = io::copy(&mut reader, &mut target);
        assert!(result.is_err());
        if let Err(error) = result {
            assert_eq!(io::ErrorKind::UnexpectedEof, error.kind());
        }
    }

}
