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
use super::ChunkedBody;

pub struct Http<R>
    where R: Send
{
    reader: Option<R>,
    computed_reader: Option<Box<io::Read + Send>>,
}

impl<R> Http<R> where R: Send
{
    pub fn new(reader: R) -> Http<R> {
        Http {
            reader: Some(reader),
            computed_reader: None,
        }
    }

    pub fn get_ref(&self) -> Option<&R> {
        self.reader.as_ref()
    }
}

impl<R> Http<R> where R: io::Read + Send + 'static
{
    pub fn read_header<F: FnMut(&str, &str)>(&mut self, mut f: F) -> io::Result<usize> {
        let mut buf_reader = io::BufReader::new(self.reader.take().unwrap());
        let mut content_length = Some(0);
        let mut buffer = String::new();
        let mut total = 0;
        loop {
            let mut line = String::new();
            total += try!(buf_reader.read_line(&mut line));
            buffer.push_str(&line);

            if line.trim().len() == 0 {
                break;
            }

            let parts = line.split(':').collect::<Vec<_>>();
            if parts.len() == 2 {
                let key = parts[0].to_lowercase().trim().to_string();
                let value = parts[1].to_string().trim().to_string();

                f(&key, &value);

                match key.as_ref() {
                    "content-length" => {
                        content_length = Some(value.parse::<u64>().unwrap());
                    }
                    "transfer-encoding" if value.to_lowercase() == "chunked" => {
                        content_length = None;
                    }
                    _ => {}
                }
            }
        }

        self.computed_reader = Some(match content_length {
            Some(size) => {
                Box::new(io::Cursor::new(buffer.into_bytes())
                             .chain(buf_reader.take(size))) as Box<io::Read + Send>
            }
            None => {
                Box::new(io::Cursor::new(buffer.into_bytes())
                             .chain(ChunkedBody::new(buf_reader))) as Box<io::Read + Send>
            }
        });

        Ok(total)
    }
}

impl<R> io::Read for Http<R> where R: io::Read + Send + 'static
{
    fn read(&mut self, buffer: &mut [u8]) -> io::Result<usize> {
        if let None = self.computed_reader {
            try!(self.read_header(|_, _| {
            }));
        }

        self.computed_reader.as_mut().unwrap().read(buffer)
    }
}

#[cfg(test)]
mod tests {

    use std::io::{self, Read};
    use super::Http;

    #[test]
    fn read_request_with_sized_body() {
        let stream = b"GET / HTTP/1.1\r\n\
                       Content-Type: text/plain\r\n\
                       Content-Length: 12\r\n\
                       \r\n\
                       test message\
                       xgfdgh";
        let mut http_reader = Http::new(io::Cursor::new(stream.to_vec()));

        let stream = &stream[0..76];
        let mut output = Vec::new();
        assert_eq!(stream.len(), http_reader.read_to_end(&mut output).unwrap());
        assert_eq!(String::from_utf8_lossy(stream),
                   String::from_utf8_lossy(&output));
    }

    #[test]
    fn read_request_with_chunked_body() {
        let stream = b"GET / HTTP/1.1\r\n\
                       Content-Type: text/plain\r\n\
                       Transfer-Encoding: Chunked\r\n\
                       \r\n\
                       c\r\n\
                       test message\r\n\
                       0\r\n\
                       \r\n\
                       xgfdgh";
        let mut http_reader = Http::new(io::Cursor::new(stream.to_vec()));

        let stream = &stream[0..94];
        let mut output = Vec::new();
        assert_eq!(stream.len(), http_reader.read_to_end(&mut output).unwrap());
        assert_eq!(String::from_utf8_lossy(stream),
                   String::from_utf8_lossy(&output));
    }

}
