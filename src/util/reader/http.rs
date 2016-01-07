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

use chunked_transfer;

use util::reader;

pub struct Http<R, H> {
    reader: io::BufReader<reader::Tee<R>>,
    handler: Option<H>,
    buffer: Option<io::Cursor<Vec<u8>>>,
}

impl<R: io::Read, H: FnMut(&str, &str)> Http<R, H> {
    pub fn new(reader: R, handler: H) -> Http<R, H> {
        Http {
            reader: io::BufReader::new(reader::Tee::new(reader)),
            handler: Some(handler),
            buffer: None,
        }
    }

    fn read_all(&mut self) -> io::Result<usize> {
        let mut total = 0;
        let mut content_length = 0;
        let mut chunked_transfer_encoding = false;
        total += try!(self.read_header(|name, value| {
            match name {
                "content-length" => {
                    content_length = value.parse::<usize>().unwrap();
                }
                "transfer-encoding" if value == "chunked" => {
                    chunked_transfer_encoding = true;
                }
                _ => {}
            }
        }));

        if content_length > 0 {
            total += try!(self.read_sized_body(content_length))
        } else if chunked_transfer_encoding {
            total += try!(self.read_chunked_body())
        }

        self.buffer = Some(io::Cursor::new(self.reader.get_mut().take_buffer()));

        Ok(total)
    }

    fn read_header<F: FnMut(&str, &str)>(&mut self, mut f: F) -> io::Result<usize> {
        let mut total = 0;
        loop {
            let mut line = String::new();
            total += try!(self.reader.read_line(&mut line));

            if line.trim().len() == 0 {
                break;
            }

            let parts = line.split(':').collect::<Vec<_>>();
            if parts.len() == 2 {
                let key = parts[0].to_lowercase().trim().to_string();
                let value = parts[1].to_string().trim().to_string();
                f(&key, &value);
                if let Some(ref mut handler) = self.handler {
                    handler(&key, &value);
                }
            }
        }
        Ok(total)
    }

    fn read_sized_body(&mut self, content_length: usize) -> io::Result<usize> {
        let mut body = Vec::with_capacity(content_length);
        unsafe {
            body.set_len(content_length);
        }
        Ok(try!(self.reader.read(&mut body)))
    }

    fn read_chunked_body(&mut self) -> io::Result<usize> {
        let mut decoder = chunked_transfer::Decoder::new(&mut self.reader);
        Ok(try!(decoder.read_to_end(&mut Vec::new())))
    }
}

impl<R: io::Read, H: FnMut(&str, &str)> io::Read for Http<R, H> {
    fn read(&mut self, buffer: &mut [u8]) -> io::Result<usize> {
        if self.buffer.is_none() {
            try!(self.read_all());
        }
        if let Some(ref mut cursor) = self.buffer {
            return Ok(try!(cursor.read(buffer)));
        }
        Ok(0)
    }
}
