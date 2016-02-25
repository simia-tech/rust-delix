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

use super::super::container;

const DEFAULT_BUFFER_SIZE: usize = 64 * 1024;

pub mod request {

    use std::io;
    use super::super::super::container;

    pub fn copy<R: ?Sized, W>(request_id: u32, reader: &mut R, w: W) -> io::Result<usize>
        where R: io::Read,
              W: FnMut(&[u8]) -> io::Result<usize>
    {
        super::copy(container::PacketType::Request, request_id, reader, w)
    }

}

pub mod response {

    use std::io;
    use super::super::super::container;

    pub fn copy<R: ?Sized, W>(request_id: u32, reader: &mut R, w: W) -> io::Result<usize>
        where R: io::Read,
              W: FnMut(&[u8]) -> io::Result<usize>
    {
        super::copy(container::PacketType::Response, request_id, reader, w)
    }

}

fn copy<R: ?Sized, W>(pt: container::PacketType,
                      request_id: u32,
                      reader: &mut R,
                      mut w: W)
                      -> io::Result<usize>
    where R: io::Read,
          W: FnMut(&[u8]) -> io::Result<usize>
{
    let mut buffer = [0; DEFAULT_BUFFER_SIZE];
    let mut total = 0;
    let mut reading = true;
    while reading {
        let result = reader.read(&mut buffer);

        match result {
            Ok(ref size) => {
                if *size > 0 {
                    total += *size;
                } else {
                    reading = false;
                }
            }
            Err(ref error) if error.kind() == io::ErrorKind::Interrupted => continue,
            Err(_) => {
                reading = false;
            }
        }

        let mut bytes = Vec::new();
        try!(container::pack_packet(pt, request_id, result, &buffer).write(&mut bytes));
        try!(w(&bytes));
    }
    Ok(total)
}

#[cfg(test)]
mod tests {

    use std::io::{self, Write};
    use util::reader;
    use super::{request, response};

    #[test]
    fn copy_request_packets_while_reader_has_no_errors() {
        let mut reader = io::Cursor::new(b"test message".to_vec());
        let mut output = Vec::new();
        assert!(request::copy(1, &mut reader, |buffer| output.write(buffer)).is_ok());
        assert_eq!(vec![0, 0, 0, 0, 0, 0, 0, 22, 8, 7, 18, 18, 8, 1, 16, 1, 34, 12, 116, 101,
                        115, 116, 32, 109, 101, 115, 115, 97, 103, 101, 0, 0, 0, 0, 0, 0, 0, 10,
                        8, 7, 18, 6, 8, 1, 16, 1, 34, 0],
                   output);
    }

    #[test]
    fn copy_request_packets_while_reader_has_expecteded_eof() {
        let mut reader = reader::ErrorAfter::new_unexpected_eof(io::Cursor::new(b"test message"
                                                                                    .to_vec()),
                                                                4);
        let mut output = Vec::new();
        assert!(request::copy(1, &mut reader, |buffer| output.write(buffer)).is_ok());
        assert_eq!(vec![0, 0, 0, 0, 0, 0, 0, 18, 8, 7, 18, 14, 8, 1, 16, 1, 34, 8, 116, 101, 115,
                        116, 32, 109, 101, 115, 0, 0, 0, 0, 0, 0, 0, 24, 8, 7, 18, 20, 8, 1, 16,
                        19, 26, 14, 117, 110, 101, 120, 112, 101, 99, 116, 101, 100, 32, 69, 79,
                        70],
                   output);
    }

    #[test]
    fn copy_response_packets_while_reader_has_no_errors() {
        let mut reader = io::Cursor::new(b"test message".to_vec());
        let mut output = Vec::new();
        assert!(response::copy(1, &mut reader, |buffer| output.write(buffer)).is_ok());
        assert_eq!(vec![0, 0, 0, 0, 0, 0, 0, 22, 8, 9, 18, 18, 8, 1, 16, 1, 34, 12, 116, 101,
                        115, 116, 32, 109, 101, 115, 115, 97, 103, 101, 0, 0, 0, 0, 0, 0, 0, 10,
                        8, 9, 18, 6, 8, 1, 16, 1, 34, 0],
                   output);
    }

    #[test]
    fn copy_response_packets_while_reader_has_expecteded_eof() {
        let mut reader = reader::ErrorAfter::new_unexpected_eof(io::Cursor::new(b"test message"
                                                                                    .to_vec()),
                                                                4);
        let mut output = Vec::new();
        assert!(response::copy(1, &mut reader, |buffer| output.write(buffer)).is_ok());
        assert_eq!(vec![0, 0, 0, 0, 0, 0, 0, 18, 8, 9, 18, 14, 8, 1, 16, 1, 34, 8, 116, 101, 115,
                        116, 32, 109, 101, 115, 0, 0, 0, 0, 0, 0, 0, 24, 8, 9, 18, 20, 8, 1, 16,
                        19, 26, 14, 117, 110, 101, 120, 112, 101, 99, 116, 101, 100, 32, 69, 79,
                        70],
                   output);
    }

}
