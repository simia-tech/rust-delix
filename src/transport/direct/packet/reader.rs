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
use std::sync::mpsc;

pub struct Reader {
    rx: mpsc::Receiver<io::Result<Vec<u8>>>,
    buffer: Box<io::Read + Send + 'static>,
}

impl Reader {
    pub fn new() -> (mpsc::Sender<io::Result<Vec<u8>>>, Self) {
        let (tx, rx) = mpsc::channel();
        (tx,
         Reader {
            rx: rx,
            buffer: Box::new(io::Cursor::new(Vec::new())),
        })
    }
}

impl io::Read for Reader {
    fn read(&mut self, buffer: &mut [u8]) -> io::Result<usize> {
        let mut result = self.buffer.read(buffer);
        if let Ok(0) = result {
            let received = match self.rx.recv() {
                Ok(result) => result,
                Err(mpsc::RecvError) => {
                    return Err(io::Error::new(io::ErrorKind::ConnectionAborted,
                                              "connection aborted"))
                }
            };
            result = match received {
                Ok(payload) => {
                    if payload.len() > 0 {
                        self.buffer = Box::new(io::Cursor::new(payload));
                        self.buffer.read(buffer)
                    } else {
                        Ok(0)
                    }
                }
                Err(error) => Err(error),
            };
        }
        result
    }
}

#[cfg(test)]
mod tests {

    use std::error::Error;
    use std::io;
    use std::thread;
    use std::sync::mpsc;
    use super::Reader;

    #[test]
    fn read_from_while_source_has_no_errors() {
        let (tx, mut reader) = Reader::new();
        thread::spawn(move || {
            send_bytes(&tx, b"test message");
            send_bytes(&tx, b"");
        });

        let mut output = Vec::new();
        assert_eq!(Some(12), io::copy(&mut reader, &mut output).ok());
        assert_eq!("test message", String::from_utf8_lossy(&output));
    }

    #[test]
    fn read_from_while_source_is_ongoing() {
        let (tx, mut reader) = Reader::new();
        thread::spawn(move || {
            send_bytes(&tx, b"test message");
            send_bytes(&tx, b"");
            send_bytes(&tx, b"ongoing");
        });

        let mut output = Vec::new();
        assert_eq!(Some(12), io::copy(&mut reader, &mut output).ok());
        assert_eq!("test message", String::from_utf8_lossy(&output));
    }

    #[test]
    fn read_from_while_source_has_unexpected_eof() {
        let (tx, mut reader) = Reader::new();
        thread::spawn(move || {
            send_unexpected_eof(&tx);
        });

        let mut output = Vec::new();
        let error = io::copy(&mut reader, &mut output).unwrap_err();
        assert_eq!(io::ErrorKind::UnexpectedEof, error.kind());
        assert_eq!("unexpected EOF", error.description());
    }

    fn send_bytes(tx: &mpsc::Sender<io::Result<Vec<u8>>>, message: &[u8]) {
        let _ = tx.send(Ok(message.to_vec()));
    }

    fn send_unexpected_eof(tx: &mpsc::Sender<io::Result<Vec<u8>>>) {
        assert!(tx.send(Err(io::Error::new(io::ErrorKind::UnexpectedEof, "unexpected EOF")))
                  .is_ok());
    }

}
