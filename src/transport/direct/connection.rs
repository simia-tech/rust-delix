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

use std::fmt;
use std::io::{self, Read, Write};
use std::iter;
use std::net::{self, SocketAddr};
use std::result;
use std::sync::{Arc, Mutex, mpsc};
use std::thread;

use protobuf::{self, Message};
use message;
use byteorder::{self, WriteBytesExt, ReadBytesExt};

use node::{self, ID, request};
use transport::cipher::{self, Cipher};
use transport::direct::container;
use util::{reader, writer};

pub struct Connection {
    tx_stream: Arc<Mutex<cipher::Stream<net::TcpStream>>>,
    thread: Option<thread::JoinHandle<()>>,

    node_id: ID,
    peer_node_id: ID,
    peer_public_address: SocketAddr,

    aknowledges_tx: Mutex<mpsc::Sender<mpsc::Sender<()>>>,

    on_add_services: Arc<Mutex<Option<Box<Fn(ID, Vec<String>) + Send>>>>,
    on_remove_services: Arc<Mutex<Option<Box<Fn(ID, Vec<String>) + Send>>>>,
    on_request: Arc<Mutex<Option<Box<Fn(&str, Box<request::Reader>) -> request::Response + Send>>>>,
    on_response: Arc<Mutex<Option<Box<Fn(u32, request::Response) + Send>>>>,
    on_shutdown: Arc<Mutex<Option<Box<Fn(ID) + Send>>>>,
    on_drop: Arc<Mutex<Option<Box<Fn(ID) + Send>>>>,
}

pub type Result<T> = result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    ConnectionLost,
    Id(node::IDError),
    AddrParse(net::AddrParseError),
    Protobuf(protobuf::ProtobufError),
    Io(io::Error),
    Cipher(cipher::Error),
    Container(container::Error),
}

impl Connection {
    pub fn new_inbound(tcp_stream: net::TcpStream,
                       cipher: Arc<Box<Cipher>>,
                       node_id: ID,
                       public_address: SocketAddr,
                       peers: &[(ID, SocketAddr)])
                       -> Result<Connection> {

        let (mut connection, sender) = try!(Self::new(tcp_stream, cipher, node_id, public_address));

        try!(connection.send_peers(peers));
        sender.send(true).unwrap();

        Ok(connection)
    }

    pub fn new_outbound(tcp_stream: net::TcpStream,
                        cipher: Arc<Box<Cipher>>,
                        node_id: ID,
                        public_address: SocketAddr)
                        -> Result<(Connection, Vec<(ID, SocketAddr)>)> {

        let (mut connection, sender) = try!(Self::new(tcp_stream, cipher, node_id, public_address));

        let peers = try!(connection.receive_peers());
        sender.send(true).unwrap();

        Ok((connection, peers))
    }

    fn new(tcp_stream: net::TcpStream,
           cipher: Arc<Box<Cipher>>,
           node_id: ID,
           public_address: SocketAddr)
           -> Result<(Connection, mpsc::Sender<bool>)> {

        let tx_stream = Arc::new(Mutex::new(cipher::Stream::new(tcp_stream.try_clone().unwrap(),
                                                                cipher.box_clone())));
        let tx_stream_clone = tx_stream.clone();
        let mut rx_stream = cipher::Stream::new(tcp_stream, cipher.box_clone());

        let (aknowledges_tx, aknowledges_rx) = mpsc::channel();

        let on_add_services: Arc<Mutex<Option<Box<Fn(ID, Vec<String>) + Send>>>> =
            Arc::new(Mutex::new(None));
        let on_add_services_clone = on_add_services.clone();

        let on_remove_services: Arc<Mutex<Option<Box<Fn(ID, Vec<String>) + Send>>>> =
            Arc::new(Mutex::new(None));
        let on_remove_services_clone = on_remove_services.clone();

        let on_request: Arc<Mutex<Option<Box<Fn(&str, Box<request::Reader>) -> request::Response + Send>>>> =
            Arc::new(Mutex::new(None));
        let on_request_clone = on_request.clone();

        let on_response: Arc<Mutex<Option<Box<Fn(u32, request::Response) + Send>>>> =
            Arc::new(Mutex::new(None));
        let on_response_clone = on_response.clone();

        let on_shutdown: Arc<Mutex<Option<Box<Fn(ID) + Send>>>> = Arc::new(Mutex::new(None));
        let on_shutdown_clone = on_shutdown.clone();

        let (peer_node_id, peer_public_address) = {
            let mut tx_stream = tx_stream.lock().unwrap();
            try!(write_container(&mut *tx_stream,
                                 &container::pack_introduction(node_id, public_address)));
            try!(container::unpack_introduction(try!(read_container(&mut *tx_stream))))
        };

        let (sender, receiver) = mpsc::channel();
        let thread = Some(thread::spawn(move || {
            receiver.recv().unwrap();
            loop {
                let container = match read_container(&mut rx_stream) {
                    Ok(container) => container,
                    Err(Error::ConnectionLost) => {
                        if let Some(ref f) = *on_shutdown_clone.lock().unwrap() {
                            f(peer_node_id);
                        }
                        break;
                    }
                    Err(err) => {
                        error!("{}: error reading connection: {:?}", node_id, err);
                        break;
                    }
                };
                match container.get_kind() {
                    message::Kind::AddServicesMessage => {
                        if let Some(ref f) = *on_add_services_clone.lock().unwrap() {
                            f(peer_node_id,
                              container::unpack_add_services(container).unwrap());
                        }
                        {
                            let mut tx_stream = tx_stream_clone.lock().unwrap();
                            write_container(&mut *tx_stream, &container::pack_aknowledge())
                                .unwrap();
                        }
                    }
                    message::Kind::RemoveServicesMessage => {
                        if let Some(ref f) = *on_remove_services_clone.lock().unwrap() {
                            f(peer_node_id,
                              container::unpack_remove_services(container).unwrap());
                        }
                        {
                            let mut tx_stream = tx_stream_clone.lock().unwrap();
                            write_container(&mut *tx_stream, &container::pack_aknowledge())
                                .unwrap();
                        }
                    }
                    message::Kind::AknowledgeMessage => {
                        container::unpack_aknowledge(container).unwrap();
                        let tx: mpsc::Sender<()> = aknowledges_rx.recv().unwrap();
                        tx.send(()).unwrap();
                    }
                    message::Kind::RequestMessage => {
                        if let Some(ref f) = *on_request_clone.lock().unwrap() {
                            let (request_id, name) = container::unpack_request(container).unwrap();

                            let mut response = f(&name,
                                                 Box::new(reader::Chunk::new(rx_stream.clone())));

                            // some errors indicate that the request body has not been read. in
                            // this case, the receive buffer is emptied here.
                            match response {
                                Err(request::Error::ServiceDoesNotExists) |
                                Err(request::Error::ServiceUnavailable) |
                                Err(request::Error::Timeout) => {
                                    let n = io::copy(&mut reader::Chunk::new(rx_stream.clone()),
                                                     &mut io::sink())
                                                .unwrap();
                                    debug!("{}: request error ({} bytes sinked)", node_id, n);
                                }
                                _ => {}
                            }

                            {
                                let mut tx_stream = tx_stream_clone.lock().unwrap();

                                write_container(&mut *tx_stream,
                                                &container::pack_response(request_id, &response))
                                    .unwrap();
                                if let Ok(ref mut reader) = response {
                                    io::copy(reader, &mut writer::Chunk::new(&mut *tx_stream))
                                        .unwrap();
                                }
                                tx_stream.flush().unwrap();
                            }
                        }
                    }
                    message::Kind::ResponseMessage => {
                        if let Some(ref f) = *on_response_clone.lock().unwrap() {
                            let (request_id, mut response) = container::unpack_response(container)
                                                                 .unwrap();
                            if let Ok(_) = response {
                                response = Ok(Box::new(reader::Chunk::new(rx_stream.clone())));
                            }
                            f(request_id, response);
                        }
                    }
                    _ => {
                        error!("{}: got unexpected container {:?}",
                               node_id,
                               container.get_kind());
                    }
                }
            }
        }));

        Ok((Connection {
            tx_stream: tx_stream,
            thread: thread,
            node_id: node_id,
            peer_node_id: peer_node_id,
            peer_public_address: peer_public_address,
            aknowledges_tx: Mutex::new(aknowledges_tx),
            on_add_services: on_add_services,
            on_remove_services: on_remove_services,
            on_request: on_request,
            on_response: on_response,
            on_shutdown: on_shutdown,
            on_drop: Arc::new(Mutex::new(None)),
        },
            sender))
    }

    pub fn peer_node_id(&self) -> ID {
        self.peer_node_id
    }

    pub fn peer_public_address(&self) -> SocketAddr {
        self.peer_public_address
    }

    pub fn peer_address(&self) -> Option<SocketAddr> {
        self.tx_stream.lock().unwrap().get_ref().peer_addr().ok()
    }

    pub fn local_address(&self) -> Option<SocketAddr> {
        self.tx_stream.lock().unwrap().get_ref().local_addr().ok()
    }

    pub fn set_on_add_services(&mut self, f: Box<Fn(ID, Vec<String>) + Send>) {
        *self.on_add_services.lock().unwrap() = Some(f);
    }

    pub fn set_on_remove_services(&mut self, f: Box<Fn(ID, Vec<String>) + Send>) {
        *self.on_remove_services.lock().unwrap() = Some(f);
    }

    pub fn set_on_request(&mut self,
                          f: Box<Fn(&str, Box<request::Reader>) -> request::Response + Send>) {
        *self.on_request.lock().unwrap() = Some(f);
    }

    pub fn set_on_response(&mut self, f: Box<Fn(u32, request::Response) + Send>) {
        *self.on_response.lock().unwrap() = Some(f);
    }

    pub fn set_on_shutdown(&mut self, f: Box<Fn(ID) + Send>) {
        *self.on_shutdown.lock().unwrap() = Some(f);
    }

    pub fn clear_on_shutdown(&mut self) {
        *self.on_shutdown.lock().unwrap() = None;
    }

    pub fn set_on_drop(&mut self, f: Box<Fn(ID) + Send>) {
        *self.on_drop.lock().unwrap() = Some(f);
    }

    pub fn send_add_services(&mut self, service_names: &[String]) -> Result<()> {
        let (tx, rx) = mpsc::channel();
        self.aknowledges_tx.lock().unwrap().send(tx).unwrap();
        {
            let mut tx_stream = self.tx_stream.lock().unwrap();
            try!(write_container(&mut *tx_stream,
                                 &container::pack_add_services(service_names)));
        }
        rx.recv().unwrap();
        Ok(())
    }

    pub fn send_remove_services(&mut self, service_names: &[String]) -> Result<()> {
        let (tx, rx) = mpsc::channel();
        self.aknowledges_tx.lock().unwrap().send(tx).unwrap();
        {
            let mut tx_stream = self.tx_stream.lock().unwrap();
            try!(write_container(&mut *tx_stream,
                                 &container::pack_remove_services(service_names)));
        }
        rx.recv().unwrap();
        Ok(())
    }

    pub fn send_request(&mut self,
                        id: u32,
                        name: &str,
                        reader: &mut request::Reader)
                        -> Result<()> {
        let mut tx_stream = self.tx_stream.lock().unwrap();
        try!(write_container(&mut *tx_stream, &container::pack_request(id, name)));
        try!(io::copy(reader, &mut writer::Chunk::new(&mut *tx_stream)));
        Ok(())
    }

    fn send_peers(&mut self, peers: &[(ID, SocketAddr)]) -> Result<()> {
        let mut tx_stream = self.tx_stream.lock().unwrap();
        try!(write_container(&mut *tx_stream, &container::pack_peers(peers)));
        Ok(())
    }

    fn receive_peers(&mut self) -> Result<Vec<(ID, SocketAddr)>> {
        let mut tx_stream = self.tx_stream.lock().unwrap();
        Ok(try!(container::unpack_peers(try!(read_container(&mut *tx_stream)))))
    }
}

impl fmt::Display for Connection {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if let (Some(local_address), Some(peer_address)) = (self.local_address(),
                                                            self.peer_address()) {
            write!(f,
                   "(Direct connection {} ({}) -> {} ({}))",
                   self.node_id,
                   local_address,
                   self.peer_node_id,
                   peer_address)
        } else {
            write!(f,
                   "(Direct connection {} (-) -> {} (-))",
                   self.node_id,
                   self.peer_node_id)
        }
    }
}

impl Drop for Connection {
    fn drop(&mut self) {
        self.tx_stream.lock().unwrap().get_ref().shutdown(net::Shutdown::Both).unwrap();
        self.thread.take().unwrap().join().unwrap();

        if let Some(ref f) = *self.on_drop.lock().unwrap() {
            f(self.peer_node_id);
        }
    }
}

impl From<byteorder::Error> for Error {
    fn from(error: byteorder::Error) -> Self {
        match error {
            byteorder::Error::Io(ref err) if err.kind() == io::ErrorKind::Other &&
                                             format!("{}", error) == "unexpected EOF" => {
                Error::ConnectionLost
            }
            byteorder::Error::Io(err) => Error::Io(err),
            byteorder::Error::UnexpectedEOF => Error::ConnectionLost,
        }
    }
}

impl From<node::IDError> for Error {
    fn from(error: node::IDError) -> Self {
        Error::Id(error)
    }
}

impl From<net::AddrParseError> for Error {
    fn from(error: net::AddrParseError) -> Self {
        Error::AddrParse(error)
    }
}

impl From<protobuf::ProtobufError> for Error {
    fn from(error: protobuf::ProtobufError) -> Self {
        Error::Protobuf(error)
    }
}

impl From<io::Error> for Error {
    fn from(error: io::Error) -> Self {
        Error::Io(error)
    }
}

impl From<cipher::Error> for Error {
    fn from(error: cipher::Error) -> Self {
        Error::Cipher(error)
    }
}

impl From<container::Error> for Error {
    fn from(error: container::Error) -> Self {
        Error::Container(error)
    }
}

fn write_container(w: &mut Write, container: &message::Container) -> Result<usize> {
    let bytes = try!(container.write_to_bytes());
    let size = bytes.len() as u64;

    try!(w.write_u64::<byteorder::BigEndian>(size));
    Ok(8 + try!(w.write(&bytes)))
}

fn read_container(stream: &mut io::Read) -> Result<message::Container> {
    let size = try!(stream.read_u64::<byteorder::BigEndian>()) as usize;

    let mut bytes = iter::repeat(0u8).take(size).collect::<Vec<u8>>();
    try!(stream.read_exact(&mut bytes));

    Ok(try!(protobuf::parse_from_bytes::<message::Container>(&bytes)))
}
