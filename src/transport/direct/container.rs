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

use std::error::Error as StdError;
use std::io::{self, Read};
use std::iter;
use std::net::{self, SocketAddr};
use std::result;

use protobuf::{self, Message};

use message;
use node::{ID, id, response, service};
use util::{reader, writer};

pub struct Container {
    message: message::Container,
}

pub type Result<T> = result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    Id(id::Error),
    Protobuf(protobuf::ProtobufError),
    AddrParse(net::AddrParseError),
}

#[derive(Clone, Copy, Debug)]
pub enum PacketType {
    Request,
    Response,
}

impl Container {
    pub fn read<R>(reader: &mut R) -> io::Result<Self>
        where R: io::Read
    {
        let size = try!(reader::read_size(reader));

        let mut bytes = iter::repeat(0u8).take(size).collect::<Vec<u8>>();
        try!(reader.read_exact(&mut bytes));

        let message = match protobuf::parse_from_bytes::<message::Container>(&bytes) {
            Ok(message) => message,
            Err(error) => {
                return Err(io::Error::new(io::ErrorKind::InvalidData, format!("{:?}", error)))
            }
        };

        Ok(Container { message: message })
    }

    pub fn write<W>(&self, writer: &mut W) -> io::Result<usize>
        where W: io::Write
    {
        let bytes = match self.message.write_to_bytes() {
            Ok(bytes) => bytes,
            Err(error) => {
                return Err(io::Error::new(io::ErrorKind::InvalidData, format!("{:?}", error)))
            }
        };
        let mut total = try!(writer::write_size(writer, bytes.len()));
        try!(writer.write_all(&bytes));
        total += bytes.len();
        Ok(total)
    }

    pub fn get_kind(&self) -> message::Kind {
        self.message.get_kind()
    }
}

pub fn pack_introduction(node_id: ID, public_address: SocketAddr) -> Container {
    let mut introduction = message::Introduction::new();
    introduction.set_id(node_id.to_vec());
    introduction.set_public_address(format!("{}", public_address));
    pack(message::Kind::IntroductionMessage, introduction)
}

pub fn unpack_introduction(container: Container) -> Result<(ID, SocketAddr)> {
    let introduction_packet = try!(unpack::<message::Introduction>(&container));
    Ok((try!(ID::from_vec(introduction_packet.get_id().to_vec())),
        try!(introduction_packet.get_public_address()
                                .parse::<SocketAddr>())))
}

pub fn pack_peers(peers: &[(ID, SocketAddr)]) -> Container {
    let mut peers_packet = message::Peers::new();
    for peer in peers {
        let (peer_node_id, peer_public_address) = *peer;
        let mut peer_packet = message::Peer::new();
        peer_packet.set_id(peer_node_id.to_vec());
        peer_packet.set_public_address(format!("{}", peer_public_address));
        peers_packet.mut_peers().push(peer_packet);
    }
    pack(message::Kind::PeersMessage, peers_packet)
}

pub fn unpack_peers(container: Container) -> Result<Vec<(ID, SocketAddr)>> {
    Ok(try!(unpack::<message::Peers>(&container))
           .get_peers()
           .iter()
           .map(|peer_packet| {
               (ID::from_vec(peer_packet.get_id().to_vec()).unwrap(),
                peer_packet.get_public_address()
                           .parse::<SocketAddr>()
                           .unwrap())
           })
           .collect())
}

pub fn pack_add_services(service_names: &[String]) -> Container {
    let mut services_packet = message::AddServices::new();
    for service_name in service_names {
        let mut service_packet = message::Service::new();
        service_packet.set_name((*service_name).to_string());
        services_packet.mut_services().push(service_packet);
    }
    pack(message::Kind::AddServicesMessage, services_packet)
}

pub fn unpack_add_services(container: Container) -> Result<Vec<String>> {
    Ok(try!(unpack::<message::AddServices>(&container))
           .get_services()
           .to_vec()
           .iter()
           .map(|service_packet| service_packet.get_name().to_string())
           .collect())
}

pub fn pack_remove_services(service_names: &[String]) -> Container {
    let mut services_packet = message::RemoveServices::new();
    for service_name in service_names {
        let mut service_packet = message::Service::new();
        service_packet.set_name((*service_name).to_string());
        services_packet.mut_services().push(service_packet);
    }
    pack(message::Kind::RemoveServicesMessage, services_packet)
}

pub fn unpack_remove_services(container: Container) -> Result<Vec<String>> {
    Ok(try!(unpack::<message::RemoveServices>(&container))
           .get_services()
           .to_vec()
           .iter()
           .map(|service_packet| service_packet.get_name().to_string())
           .collect())
}

pub fn pack_aknowledge() -> Container {
    pack(message::Kind::AknowledgeMessage, message::Aknowledge::new())
}

pub fn unpack_aknowledge(container: Container) -> Result<()> {
    try!(unpack::<message::Aknowledge>(&container));
    Ok(())
}

pub fn pack_request(id: u32, name: &str) -> Container {
    let mut request_packet = message::Request::new();
    request_packet.set_id(id);
    request_packet.set_name(name.to_string());
    pack(message::Kind::RequestMessage, request_packet)
}

pub fn unpack_request(container: Container) -> Result<(u32, String)> {
    let request_packet = try!(unpack::<message::Request>(&container));
    Ok((request_packet.get_id(),
        request_packet.get_name().to_string()))
}

pub fn pack_response(request_id: u32, response: &service::Result) -> Container {
    let mut response_packet = message::Response::new();
    response_packet.set_request_id(request_id);
    match *response {
        Ok(_) => {
            response_packet.set_kind(message::Response_Kind::OK);
        }
        Err(service::Error::Unavailable) => {
            response_packet.set_kind(message::Response_Kind::Unavailable);
        }
        Err(service::Error::Timeout) => {
            response_packet.set_kind(message::Response_Kind::Timeout);
        }
        Err(service::Error::Internal(ref message)) => {
            response_packet.set_kind(message::Response_Kind::Internal);
            response_packet.set_message(message.to_string());
        }
    }
    pack(message::Kind::ResponseMessage, response_packet)
}

pub fn unpack_response(container: Container,
                       response_reader: Box<response::Reader>)
                       -> Result<(u32, service::Result)> {
    let response_packet = try!(unpack::<message::Response>(&container));
    let result = match response_packet.get_kind() {
        message::Response_Kind::OK => Ok(response_reader),
        message::Response_Kind::Unavailable => Err(service::Error::Unavailable),
        message::Response_Kind::Timeout => Err(service::Error::Timeout),
        message::Response_Kind::Internal => {
            Err(service::Error::Internal(response_packet.get_message().to_string()))
        }
    };
    Ok((response_packet.get_request_id(), result))
}

pub fn pack_packet(pt: PacketType,
                   request_id: u32,
                   result: io::Result<usize>,
                   buffer: &[u8])
                   -> Container {
    let mut packet = message::Packet::new();
    packet.set_request_id(request_id);
    match result {
        Ok(size) => {
            packet.set_result(message::Packet_Result::Ok);
            packet.set_payload(buffer[..size].to_vec());
        }
        Err(error) => {
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
        }
    }
    pack(message::Kind::from(pt), packet)
}

pub fn unpack_packet(container: Container) -> Result<(u32, io::Result<Vec<u8>>)> {
    let mut packet = try!(unpack::<message::Packet>(&container));
    match packet.get_result() {
        message::Packet_Result::Ok => Ok((packet.get_request_id(), Ok(packet.take_payload()))),
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
            Ok((packet.get_request_id(), Err(io::Error::new(kind, message))))
        }
    }
}

fn pack<T>(kind: message::Kind, message: T) -> Container
    where T: protobuf::Message + protobuf::MessageStatic
{
    let mut payload = Vec::new();
    message.write_to_vec(&mut payload).unwrap();

    let mut container_message = message::Container::new();
    container_message.set_kind(kind);
    container_message.set_payload(payload);
    Container { message: container_message }
}

fn unpack<T>(container: &Container) -> Result<T>
    where T: protobuf::Message + protobuf::MessageStatic
{
    Ok(try!(protobuf::parse_from_bytes::<T>(container.message.get_payload())))
}

impl From<PacketType> for message::Kind {
    fn from(pt: PacketType) -> Self {
        match pt {
            PacketType::Request => message::Kind::RequestPacketMessage,
            PacketType::Response => message::Kind::ResponsePacketMessage,
        }
    }
}

impl From<id::Error> for Error {
    fn from(error: id::Error) -> Self {
        Error::Id(error)
    }
}

impl From<protobuf::ProtobufError> for Error {
    fn from(error: protobuf::ProtobufError) -> Self {
        Error::Protobuf(error)
    }
}

impl From<net::AddrParseError> for Error {
    fn from(error: net::AddrParseError) -> Self {
        Error::AddrParse(error)
    }
}

impl From<Error> for io::Error {
    fn from(error: Error) -> Self {
        io::Error::new(io::ErrorKind::InvalidData, format!("{:?}", error))
    }
}
