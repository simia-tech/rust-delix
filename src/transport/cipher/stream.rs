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
use std::net;

use transport::cipher::{self, Cipher};
use util::{reader, writer};

pub struct Stream<T> {
    parent: T,
    cipher: Box<Cipher>,
    buffer: io::Cursor<Vec<u8>>,
}

impl<T> Stream<T> {
    pub fn new(parent: T, cipher: Box<Cipher>) -> Stream<T> {
        Stream {
            parent: parent,
            cipher: cipher,
            buffer: io::Cursor::new(Vec::new()),
        }
    }

    pub fn get_ref(&self) -> &T {
        &self.parent
    }
}

impl Stream<net::TcpStream> {
    pub fn try_clone(&self) -> io::Result<Self> {
        Ok(Self::new(try!(self.parent.try_clone()), self.cipher.box_clone()))
    }
}

impl<T> io::Write for Stream<T>
    where T: io::Write
{
    fn write(&mut self, buffer: &[u8]) -> io::Result<usize> {
        let encrypted_bytes = try!(self.cipher.encrypt(buffer));

        try!(writer::write_size(&mut self.parent, encrypted_bytes.len()));
        try!(self.parent.write(&encrypted_bytes));

        Ok(buffer.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        self.parent.flush()
    }
}

impl<T> io::Read for Stream<T>
    where T: io::Read
{
    fn read(&mut self, buffer: &mut [u8]) -> io::Result<usize> {
        if self.buffer.position() as usize >= self.buffer.get_ref().len() {
            let encrypted_size = try!(reader::read_size(&mut self.parent));

            let mut encrypted_bytes = iter::repeat(0u8).take(encrypted_size).collect::<Vec<u8>>();
            try!(self.parent.read_exact(&mut encrypted_bytes));

            let decrypted_bytes = try!(self.cipher.decrypt(&encrypted_bytes));
            self.buffer = io::Cursor::new(decrypted_bytes);
        }

        self.buffer.read(buffer)
    }
}

impl Clone for Stream<net::TcpStream> {
    fn clone(&self) -> Self {
        Self::new(self.parent.try_clone().unwrap(), self.cipher.box_clone())
    }
}

impl From<cipher::Error> for io::Error {
    fn from(error: cipher::Error) -> Self {
        io::Error::new(io::ErrorKind::Other, format!("cipher error: {:?}", error))
    }
}

#[cfg(test)]
mod tests {

    use std::io::{self, Read, Write};
    use rustc_serialize::hex::{FromHex, ToHex};
    use super::Stream;
    use super::super::{Cipher, Symmetric};

    #[test]
    fn write() {
        let mut stream = Stream::new(Vec::new(), build_cipher());
        assert!(stream.write_all(b"test message").is_ok());

        assert_eq!("00000000000000300801120c0000000000000000000000001a0c3db3f427b9f6c3ff90e81d0d2\
                    2102958d0a32be787b9c59da25053419e41",
                   stream.get_ref().to_hex());
    }

    #[test]
    fn read() {
        let mut stream = Stream::new(io::Cursor::new("00000000000000300801120c000000000000000000\
                                                      0000001a0c3db3f427b9f6c3ff90e81d0d22102958\
                                                      d0a32be787b9c59da25053419e41"
                                                         .from_hex()
                                                         .ok()
                                                         .unwrap()
                                                         .to_vec()),
                                     build_cipher());

        let mut buffer = [0u8; 12];
        assert!(stream.read_exact(&mut buffer).is_ok());
        assert_eq!("test message", String::from_utf8_lossy(&buffer));
    }

    fn build_cipher() -> Box<Cipher> {
        Box::new(Symmetric::new(&"000102030405060708090a0b0c0d0e0f"
                                     .from_hex()
                                     .ok()
                                     .unwrap(),
                                Some(&"000000000000000000000000"
                                          .from_hex()
                                          .ok()
                                          .unwrap()))
                     .unwrap())
    }

}
