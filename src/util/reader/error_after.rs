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

pub struct ErrorAfter<R> {
    reader: R,
    limit: usize,
    error: Option<io::Error>,
    bytes_read: usize,
}


impl<R> ErrorAfter<R> {
    pub fn new(reader: R, limit: usize, error: io::Error) -> Self {
        ErrorAfter {
            reader: reader,
            limit: limit,
            error: Some(error),
            bytes_read: 0,
        }
    }

    pub fn new_unexpected_eof(reader: R, limit: usize) -> Self {
        Self::new(reader,
                  limit,
                  io::Error::new(io::ErrorKind::UnexpectedEof, "unexpected EOF"))
    }
}

impl<R> io::Read for ErrorAfter<R> where R: io::Read
{
    fn read(&mut self, buffer: &mut [u8]) -> io::Result<usize> {
        if self.bytes_read >= self.limit {
            if let Some(error) = self.error.take() {
                return Err(error);
            }
        }

        let result = self.reader.read(buffer);

        if let Ok(count) = result {
            self.bytes_read += count;
            if self.bytes_read >= self.limit {
                return Ok(self.bytes_read - self.limit);
            }
        }

        result
    }
}

#[cfg(test)]
mod tests {

    use std::error::Error;
    use std::io;
    use super::ErrorAfter;

    #[test]
    fn read() {
        let parent = io::Cursor::new(b"test message".to_vec());
        let mut reader = ErrorAfter::new(parent,
                                         30,
                                         io::Error::new(io::ErrorKind::Other, "test error"));

        let mut output = Vec::new();
        assert!(io::copy(&mut reader, &mut output).is_ok());
        assert_eq!("test message", String::from_utf8_lossy(&output));
    }

    #[test]
    fn read_and_error_after() {
        let parent = io::Cursor::new(b"test message".to_vec());
        let error = io::Error::new(io::ErrorKind::Other, "test error");
        let mut reader = ErrorAfter::new(parent, 4, error);

        let error = io::copy(&mut reader, &mut Vec::new()).unwrap_err();
        assert_eq!(io::ErrorKind::Other, error.kind());
        assert_eq!("test error", error.description());
    }

}
