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

use std::error::Error;
use std::io;
use std::iter;

use protobuf::{self, Message};

use message;
use util::{reader, writer};

const DEFAULT_BUFFER_SIZE: usize = 64 * 1024;

pub struct Reader<R, F>
    where R: io::Read,
          F: FnMut(io::Error)
{
    reader: R,
    buffer: Box<io::Read + Send + 'static>,
    error_handler: F,
}

impl<R, F> Reader<R, F>
    where R: io::Read,
          F: FnMut(io::Error)
{
    pub fn new(reader: R, error_handler: F) -> Self {
        Reader {
            reader: reader,
            buffer: Box::new(io::Cursor::new(Vec::new())),
            error_handler: error_handler,
        }
    }

    fn read_packet(&mut self) -> io::Result<message::Packet> {
        let size = try!(reader::read_size(&mut self.reader));
        let mut bytes = iter::repeat(0u8).take(size).collect::<Vec<u8>>();
        try!(self.reader.read_exact(&mut bytes));
        Ok(protobuf::parse_from_bytes::<message::Packet>(&bytes).unwrap())
    }
}

impl<R, F> io::Read for Reader<R, F>
    where R: io::Read,
          F: FnMut(io::Error)
{
    fn read(&mut self, buffer: &mut [u8]) -> io::Result<usize> {
        let mut result = self.buffer.read(buffer);
        if let Ok(0) = result {
            let mut packet = match self.read_packet() {
                Ok(packet) => packet,
                Err(error) => {
                    (self.error_handler)(error);
                    return Err(io::Error::new(io::ErrorKind::Other, "connection error"));
                }
            };

            result = match unpack(&mut packet) {
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

pub fn copy<R: ?Sized, W: ?Sized>(reader: &mut R, writer: &mut W) -> io::Result<usize>
    where R: io::Read,
          W: io::Write
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

        let packet = pack(result, &buffer);

        let bytes = packet.write_to_bytes().unwrap();
        try!(writer::write_size(writer, bytes.len()));
        try!(writer.write_all(&bytes));
    }
    Ok(total)
}

fn pack(result: io::Result<usize>, buffer: &[u8]) -> message::Packet {
    match result {
        Ok(size) => {
            let mut packet = message::Packet::new();
            packet.set_result(message::Packet_Result::Ok);
            packet.set_payload(buffer[..size].to_vec());
            packet
        }
        Err(error) => {
            let mut packet = message::Packet::new();
            packet.set_result(match error.kind() {
                io::ErrorKind::NotFound => message::Packet_Result::NotFound,
                io::ErrorKind::PermissionDenied => message::Packet_Result::PermissionDenied,
                io::ErrorKind::ConnectionRefused => message::Packet_Result::ConnectionRefused,
                io::ErrorKind::ConnectionReset => message::Packet_Result::ConnectionReset,
                io::ErrorKind::ConnectionAborted => message::Packet_Result::ConnectionAborted,
                io::ErrorKind::NotConnected => message::Packet_Result::NotConnected,
                io::ErrorKind::AddrInUse => message::Packet_Result::AddrInUse,
                io::ErrorKind::AddrNotAvailable => message::Packet_Result::AddrNotAvailable,
                io::ErrorKind::BrokenPipe => message::Packet_Result::BrokenPipe,
                io::ErrorKind::AlreadyExists => message::Packet_Result::AlreadyExists,
                io::ErrorKind::WouldBlock => message::Packet_Result::WouldBlock,
                io::ErrorKind::InvalidInput => message::Packet_Result::InvalidInput,
                io::ErrorKind::InvalidData => message::Packet_Result::InvalidData,
                io::ErrorKind::TimedOut => message::Packet_Result::TimedOut,
                io::ErrorKind::WriteZero => message::Packet_Result::WriteZero,
                io::ErrorKind::Interrupted => message::Packet_Result::Interrupted,
                io::ErrorKind::Other => message::Packet_Result::Other,
                io::ErrorKind::UnexpectedEof => message::Packet_Result::UnexpectedEof,
                _ => unreachable!(),
            });
            packet.set_message(error.description().to_string());
            packet.set_payload(Vec::new());
            packet
        }
    }
}

fn unpack(packet: &mut message::Packet) -> io::Result<Vec<u8>> {
    match packet.get_result() {
        message::Packet_Result::Ok => Ok(packet.take_payload()),
        _ => {
            let message = packet.take_message();
            let kind = match packet.get_result() {
                message::Packet_Result::NotFound => io::ErrorKind::NotFound,
                message::Packet_Result::PermissionDenied => io::ErrorKind::PermissionDenied,
                message::Packet_Result::ConnectionRefused => io::ErrorKind::ConnectionRefused,
                message::Packet_Result::ConnectionReset => io::ErrorKind::ConnectionReset,
                message::Packet_Result::ConnectionAborted => io::ErrorKind::ConnectionAborted,
                message::Packet_Result::NotConnected => io::ErrorKind::NotConnected,
                message::Packet_Result::AddrInUse => io::ErrorKind::AddrInUse,
                message::Packet_Result::AddrNotAvailable => io::ErrorKind::AddrNotAvailable,
                message::Packet_Result::BrokenPipe => io::ErrorKind::BrokenPipe,
                message::Packet_Result::AlreadyExists => io::ErrorKind::AlreadyExists,
                message::Packet_Result::WouldBlock => io::ErrorKind::WouldBlock,
                message::Packet_Result::InvalidInput => io::ErrorKind::InvalidInput,
                message::Packet_Result::InvalidData => io::ErrorKind::InvalidData,
                message::Packet_Result::TimedOut => io::ErrorKind::TimedOut,
                message::Packet_Result::WriteZero => io::ErrorKind::WriteZero,
                message::Packet_Result::Interrupted => io::ErrorKind::Interrupted,
                message::Packet_Result::Other => io::ErrorKind::Other,
                message::Packet_Result::UnexpectedEof => io::ErrorKind::UnexpectedEof,
                _ => unreachable!(),
            };
            Err(io::Error::new(kind, message))
        }
    }
}

#[cfg(test)]
mod tests {

    use std::error::Error;
    use std::io::{self, Read};
    use util::reader;
    use super::{Reader, copy};

    #[test]
    fn copy_while_reader_has_no_errors() {
        let mut reader = io::Cursor::new(b"test message".to_vec());
        let mut writer = Vec::new();
        assert_eq!(Some(12), copy(&mut reader, &mut writer).ok());
        assert_eq!(36, writer.len());
    }

    #[test]
    fn copy_while_reader_has_expecteded_eof() {
        let mut reader = reader::ErrorAfter::new_unexpected_eof(io::Cursor::new(b"test message"
                                                                                    .to_vec()),
                                                                4);
        let mut writer = Vec::new();
        assert_eq!(Some(8), copy(&mut reader, &mut writer).ok());
        assert_eq!(48, writer.len());
    }

    #[test]
    fn read_from_while_source_has_no_errors() {
        let mut reader = io::Cursor::new(b"test message".to_vec());

        let mut buffer = Vec::new();
        assert_eq!(Some(12), copy(&mut reader, &mut buffer).ok());
        assert_eq!(36, buffer.len());

        let mut reader = Reader::new(io::Cursor::new(buffer), |_| {});
        let mut writer = Vec::new();
        assert_eq!(Some(12), io::copy(&mut reader, &mut writer).ok());

        assert_eq!("test message", String::from_utf8_lossy(&writer));
    }

    #[test]
    fn read_from_while_source_is_ongoing() {
        let mut reader = io::Cursor::new(b"test message".to_vec());

        let mut buffer = Vec::new();
        assert_eq!(Some(12), copy(&mut reader, &mut buffer).ok());
        assert_eq!(36, buffer.len());

        let mut reader = Reader::new(io::Cursor::new(buffer)
                                         .chain(io::Cursor::new(b"ongoing".to_vec())),
                                     |_| {
                                     });
        let mut writer = Vec::new();
        assert_eq!(Some(12), io::copy(&mut reader, &mut writer).ok());

        assert_eq!("test message", String::from_utf8_lossy(&writer));
    }

    #[test]
    fn read_from_while_source_has_unexpected_eof() {
        let mut reader = reader::ErrorAfter::new_unexpected_eof(io::Cursor::new(b"test message"
                                                                                    .to_vec()),
                                                                4);

        let mut buffer = Vec::new();
        assert_eq!(Some(8), copy(&mut reader, &mut buffer).ok());
        assert_eq!(48, buffer.len());

        let mut reader = Reader::new(io::Cursor::new(buffer), |_| {});
        let mut writer = Vec::new();
        let error = io::copy(&mut reader, &mut writer).unwrap_err();

        assert_eq!(io::ErrorKind::UnexpectedEof, error.kind());
        assert_eq!("unexpected EOF", error.description());
    }

    #[test]
    fn read_from_while_buffer_has_unexpected_eof() {
        let mut reader = io::Cursor::new(b"test message".to_vec());

        let mut buffer = Vec::new();
        assert_eq!(Some(12), copy(&mut reader, &mut buffer).ok());
        assert_eq!(36, buffer.len());

        let reader = reader::ErrorAfter::new_unexpected_eof(io::Cursor::new(buffer), 4);
        let mut reader_error = None;
        {
            let mut reader = Reader::new(reader, |error| {
                reader_error = Some(error);
            });

            let mut writer = Vec::new();
            let error = io::copy(&mut reader, &mut writer).unwrap_err();
            assert_eq!(io::ErrorKind::Other, error.kind());
            assert_eq!("connection error", error.description());
        }

        assert_eq!(io::ErrorKind::UnexpectedEof,
                   reader_error.as_ref().unwrap().kind());
        assert_eq!("unexpected EOF",
                   reader_error.as_ref().unwrap().description());
    }

}
