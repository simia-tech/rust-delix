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

use std::io::{self, BufRead, Read};
use std::fmt;
use std::error;

pub struct ChunkedBody<R> {
    reader: Option<R>,
    chunk_reader: Option<Box<io::BufRead + Send>>,
    remaining_chunks_size: Option<usize>,
    remaining_chunks: bool,
}

#[derive(Debug, Copy, Clone)]
struct Error;

impl<R> ChunkedBody<R> where R: io::Read + Send + 'static
{
    pub fn new(reader: R) -> ChunkedBody<R> {
        ChunkedBody {
            reader: Some(reader),
            chunk_reader: None,
            remaining_chunks_size: None,
            remaining_chunks: true,
        }
    }

    fn peek_chunk_size(&mut self) -> io::Result<usize> {
        if let None = self.chunk_reader {
            self.chunk_reader = Some(Box::new(io::BufReader::new(self.reader.take().unwrap())));
        }

        let mut line = String::new();
        try!(self.chunk_reader.as_mut().unwrap().read_line(&mut line));

        if !line.ends_with("\r\n") {
            return Err(io::Error::new(io::ErrorKind::InvalidInput, Error));
        }

        let mut chunk_size = {
            let chunk_size = line.split(';').collect::<Vec<&str>>()[0];

            match usize::from_str_radix(chunk_size.trim(), 16) {
                Ok(chunk_size) => chunk_size,
                Err(_) => return Err(io::Error::new(io::ErrorKind::InvalidInput, Error)),
            }
        };
        if chunk_size == 0 {
            self.remaining_chunks = false;
        }

        let line_bytes = line.into_bytes();
        chunk_size += line_bytes.len();

        self.chunk_reader = Some(Box::new(io::BufReader::new(io::Cursor::new(line_bytes)
                                                                 .chain(self.chunk_reader
                                                                            .take()
                                                                            .unwrap()))));

        Ok(chunk_size + 2)
    }
}

impl<R> io::Read for ChunkedBody<R> where R: io::Read + Send + 'static
{
    fn read(&mut self, buffer: &mut [u8]) -> io::Result<usize> {
        let remaining_chunks_size = match self.remaining_chunks_size {
            Some(chunk_size) => chunk_size,
            None => {
                if !self.remaining_chunks {
                    return Ok(0);
                }
                try!(self.peek_chunk_size())
            }
        };

        if buffer.len() < remaining_chunks_size {
            let read = try!(self.chunk_reader.as_mut().unwrap().read(buffer));
            self.remaining_chunks_size = Some(remaining_chunks_size - read);
            return Ok(read);
        }

        assert!(buffer.len() >= remaining_chunks_size);

        let buffer = &mut buffer[..remaining_chunks_size];
        let read = try!(self.chunk_reader.as_mut().unwrap().read(buffer));

        self.remaining_chunks_size = if read == remaining_chunks_size {
            None
        } else {
            Some(remaining_chunks_size - read)
        };

        Ok(read)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(fmt, "Error while decoding chunks")
    }
}

impl error::Error for Error {
    fn description(&self) -> &str {
        "Error while decoding chunks"
    }
}

#[cfg(test)]
mod test {

    use std::io::{self, Read};
    use super::ChunkedBody;

    /// This unit test is taken from from Hyper
    /// https://github.com/hyperium/hyper
    /// Copyright (c) 2014 Sean McArthur
    #[test]
    fn peek_chunk_size() {
        fn peek(s: &'static str, expected: usize) {
            let mut body = ChunkedBody::new(s.as_bytes());
            let actual = body.peek_chunk_size().unwrap();
            assert_eq!(expected, actual);
        }

        fn peek_err(s: &'static str) {
            let mut body = ChunkedBody::new(s.as_bytes());
            let err_kind = body.peek_chunk_size().unwrap_err().kind();
            assert_eq!(err_kind, io::ErrorKind::InvalidInput);
        }

        peek("1\r\n", 6);
        peek("01\r\n", 7);
        peek("0\r\n", 5);
        peek("00\r\n", 6);
        peek("A\r\n", 15);
        peek("a\r\n", 15);
        peek("Ff\r\n", 261);
        peek("Ff   \r\n", 264);
        // Missing LF or CRLF
        peek_err("F\rF");
        peek_err("F");
        // Invalid hex digit
        peek_err("X\r\n");
        peek_err("1X\r\n");
        peek_err("-\r\n");
        peek_err("-1\r\n");
        // Acceptable (if not fully valid) extensions do not influence the size
        peek("1;extension\r\n", 16);
        peek("a;ext name=value\r\n", 30);
        peek("1;extension;extension2\r\n", 27);
        peek("1;;;  ;\r\n", 12);
        peek("2; extension...\r\n", 21);
        peek("3   ; extension=123\r\n", 26);
        peek("3   ;\r\n", 12);
        peek("3   ;   \r\n", 15);
        // Invalid extensions cause an error
        peek_err("1 invalid extension\r\n");
        peek_err("1 A\r\n");
        peek_err("1;no CRLF");
    }

    #[test]
    fn read_valid_chunk() {
        let reader = io::Cursor::new("3\r\nhel\r\nb\r\nlo world!!!\r\n0\r\n\r\nxxx"
                                         .to_string()
                                         .into_bytes());
        let mut body = ChunkedBody::new(reader);

        let mut string = String::new();
        body.read_to_string(&mut string).unwrap();

        assert_eq!("3\r\nhel\r\nb\r\nlo world!!!\r\n0\r\n\r\n", string);
    }

    #[test]
    fn read_zero_length_chunk() {
        let mut decoder = ChunkedBody::new(b"0\r\n\r\n" as &[u8]);

        let mut body = String::new();
        decoder.read_to_string(&mut body).unwrap();

        assert_eq!("0\r\n\r\n", body);
    }

    #[test]
    fn read_invalid_chunk_length() {
        let mut decoder = ChunkedBody::new(b"m\r\n\r\n" as &[u8]);

        let mut body = String::new();
        assert!(decoder.read_to_string(&mut body).is_err());
    }

    #[test]
    fn read_invalid_length_chunk() {
        let reader = io::Cursor::new("2\r\nhel\r\nb\r\nlo world!!!\r\n0\r\n"
                                         .to_string()
                                         .into_bytes());
        let mut body = ChunkedBody::new(reader);

        let mut string = String::new();
        body.read_to_string(&mut string).is_err();
    }

    #[test]
    fn read_invalid_line_break() {
        let reader = io::Cursor::new("3\rhel\r\nb\r\nlo world!!!\r\n0\r\n"
                                         .to_string()
                                         .into_bytes());
        let mut body = ChunkedBody::new(reader);

        let mut string = String::new();
        body.read_to_string(&mut string).is_err();
    }
}
