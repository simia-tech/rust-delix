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
use std::net;

use byteorder::{self, WriteBytesExt, ReadBytesExt};

use transport::cipher::{self, Cipher};

pub struct Stream {
    tcp_stream: net::TcpStream,
    cipher: Box<Cipher>,
    buffer: io::Cursor<Vec<u8>>,
}

impl Stream {
    pub fn new(tcp_stream: net::TcpStream, cipher: Box<Cipher>) -> Stream {
        Stream {
            tcp_stream: tcp_stream,
            cipher: cipher,
            buffer: io::Cursor::new(Vec::new()),
        }
    }

    pub fn try_clone(&self) -> io::Result<Self> {
        Ok(Self::new(try!(self.tcp_stream.try_clone()), self.cipher.box_clone()))
    }

    pub fn get_ref(&self) -> &net::TcpStream {
        &self.tcp_stream
    }
}

impl io::Write for Stream {
    fn write(&mut self, buffer: &[u8]) -> io::Result<usize> {
        let encrypted_bytes = try!(self.cipher.encrypt(buffer));
        let encrypted_size = encrypted_bytes.len() as u64;

        try!(self.tcp_stream.write_u64::<byteorder::BigEndian>(encrypted_size));
        try!(self.tcp_stream.write(&encrypted_bytes));

        Ok(buffer.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        self.tcp_stream.flush()
    }
}

impl io::Read for Stream {
    fn read(&mut self, buffer: &mut [u8]) -> io::Result<usize> {
        if self.buffer.position() as usize >= self.buffer.get_ref().len() {
            let encrypted_size = try!(self.tcp_stream.read_u64::<byteorder::BigEndian>()) as usize;

            let mut encrypted_bytes = Vec::with_capacity(encrypted_size);
            unsafe {
                encrypted_bytes.set_len(encrypted_size);
            }
            try!(self.tcp_stream.read(&mut encrypted_bytes));

            let decrypted_bytes = try!(self.cipher.decrypt(&encrypted_bytes));
            self.buffer = io::Cursor::new(decrypted_bytes);
        }

        self.buffer.read(buffer)
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
    use std::net;
    use std::sync::mpsc;
    use std::thread;
    use rustc_serialize::hex::{FromHex, ToHex};
    use super::Stream;
    use super::super::Symmetric;

    #[test]
    fn write() {
        let rx = run_receiver("127.0.0.1:3001");

        {
            let mut stream = build_stream("127.0.0.1:3001");
            assert!(stream.write_all(b"test message").is_ok());
        }


        assert_eq!("00000000000000300801120c0000000000000000000000001a0c3db3f427b9f6c3ff90e81d0d2\
                    2102958d0a32be787b9c59da25053419e41",
                   rx.recv().unwrap().to_hex());
    }

    #[test]
    fn read() {
        let tx = run_sender("127.0.0.1:3011");

        let mut stream = build_stream("127.0.0.1:3011");
        tx.send("00000000000000300801120c0000000000000000000000001a0c3db3f427b9f6c3ff90e81d0d2210\
                 2958d0a32be787b9c59da25053419e41"
                    .from_hex()
                    .ok()
                    .unwrap()
                    .to_vec())
          .unwrap();

        let mut buffer = Vec::new();
        match stream.read_to_end(&mut buffer) {
            Err(ref error) if error.kind() == io::ErrorKind::Other &&
                              format!("{}", error) == "unexpected EOF" => {}
            Err(error) => panic!(error),
            Ok(_) => {}
        }
        assert_eq!("test message", String::from_utf8_lossy(&buffer));
    }

    fn run_receiver(address: &str) -> mpsc::Receiver<Vec<u8>> {
        let address = address.parse::<net::SocketAddr>().unwrap();
        let tcp_listener = net::TcpListener::bind(address).unwrap();
        let (tx, rx) = mpsc::channel();
        thread::spawn(move || {
            let (mut tcp_stream, _) = tcp_listener.accept().unwrap();
            let mut buffer = Vec::new();
            tcp_stream.read_to_end(&mut buffer).unwrap();
            tx.send(buffer).unwrap();
        });
        rx
    }

    fn run_sender(address: &str) -> mpsc::Sender<Vec<u8>> {
        let address = address.parse::<net::SocketAddr>().unwrap();
        let tcp_listener = net::TcpListener::bind(address).unwrap();
        let (tx, rx) = mpsc::channel::<Vec<u8>>();
        thread::spawn(move || {
            let (mut tcp_stream, _) = tcp_listener.accept().unwrap();
            let buffer = rx.recv().unwrap();
            tcp_stream.write_all(&buffer).unwrap();
        });
        tx
    }

    fn build_stream(address: &str) -> Stream {
        let cipher = Box::new(Symmetric::new(&"000102030405060708090a0b0c0d0e0f"
                                                  .from_hex()
                                                  .ok()
                                                  .unwrap(),
                                             Some(&"000000000000000000000000"
                                                       .from_hex()
                                                       .ok()
                                                       .unwrap()))
                                  .unwrap());
        let tcp_stream = net::TcpStream::connect(address.parse::<net::SocketAddr>()
                                                        .unwrap())
                             .unwrap();
        Stream::new(tcp_stream, cipher)
    }

}
