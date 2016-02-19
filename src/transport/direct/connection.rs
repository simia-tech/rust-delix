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

use openssl::ssl;

use message;
use node::{ID, request, service};
use util::{reader, writer};
use super::packet;
use super::container::{self, Container};

pub struct Connection {
    tx_stream: Arc<Mutex<ssl::SslStream<net::TcpStream>>>,
    thread: Option<thread::JoinHandle<()>>,

    node_id: ID,
    peer_node_id: ID,
    peer_public_address: SocketAddr,

    aknowledges_tx: Mutex<mpsc::Sender<mpsc::Sender<()>>>,

    handler: Arc<Mutex<Handler>>,
}

pub struct Handler {
    pub add_services: Box<Fn(ID, Vec<String>) + Send>,
    pub remove_services: Box<Fn(ID, Vec<String>) + Send>,
    pub request: Box<Fn(&str, Box<request::Reader>) -> service::Result + Send>,
    pub response: Box<Fn(u32, service::Result) -> result::Result<(), io::Error> + Send>,
    pub drop: Box<Fn(ID) + Send>,
    pub error: Option<Box<Fn(ID, &io::Error) + Send>>,
}

impl Connection {
    pub fn new_inbound(ssl_stream: ssl::SslStream<net::TcpStream>,
                       node_id: ID,
                       public_address: SocketAddr,
                       peers: &[(ID, SocketAddr)],
                       handler: Handler)
                       -> io::Result<Connection> {

        let (connection, sender) = try!(Self::new(ssl_stream, node_id, public_address, handler));

        try!(connection.send_peers(peers));
        sender.send(true).unwrap();

        Ok(connection)
    }

    pub fn new_outbound(ssl_stream: ssl::SslStream<net::TcpStream>,
                        node_id: ID,
                        public_address: SocketAddr,
                        handler: Handler)
                        -> io::Result<(Connection, Vec<(ID, SocketAddr)>)> {

        let (connection, sender) = try!(Self::new(ssl_stream, node_id, public_address, handler));

        let peers = try!(connection.receive_peers());
        sender.send(true).unwrap();

        Ok((connection, peers))
    }

    fn new(ssl_stream: ssl::SslStream<net::TcpStream>,
           node_id: ID,
           public_address: SocketAddr,
           handler: Handler)
           -> io::Result<(Connection, mpsc::Sender<bool>)> {

        let tx_stream = Arc::new(Mutex::new(ssl_stream.try_clone().unwrap()));
        let tx_stream_clone = tx_stream.clone();
        let mut rx_stream = ssl_stream;

        let (aknowledges_tx, aknowledges_rx) = mpsc::channel();

        let handler = Arc::new(Mutex::new(handler));
        let handler_clone = handler.clone();

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
                match process_inbound_container(node_id,
                                                peer_node_id,
                                                &mut rx_stream,
                                                &tx_stream_clone,
                                                &aknowledges_rx,
                                                &handler_clone) {
                    Ok(()) => {}
                    Err(ref error) => {
                        if let Some(ref f) = handler_clone.lock().unwrap().error {
                            f(peer_node_id, error);
                        }
                        break;
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
            handler: handler,
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

    pub fn set_on_error(&self, f: Box<Fn(ID, &io::Error) + Send>) {
        self.handler.lock().unwrap().error = Some(f);
    }

    pub fn clear_on_error(&self) {
        self.handler.lock().unwrap().error = None;
    }

    pub fn send_add_services(&self, service_names: &[String]) -> io::Result<()> {
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

    pub fn send_remove_services(&self, service_names: &[String]) -> io::Result<()> {
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

    pub fn send_request(&self,
                        id: u32,
                        name: &str,
                        reader: &mut request::Reader)
                        -> io::Result<()> {
        self.catch_error((), || {
            let mut tx_stream = self.tx_stream.lock().unwrap();

            try!(write_container(&mut *tx_stream, &container::pack_request(id, name)));

            try!(packet::copy(reader, &mut *tx_stream));

            Ok(())
        })
    }

    fn send_peers(&self, peers: &[(ID, SocketAddr)]) -> io::Result<()> {
        let mut tx_stream = self.tx_stream.lock().unwrap();
        try!(write_container(&mut *tx_stream, &container::pack_peers(peers)));
        Ok(())
    }

    fn receive_peers(&self) -> io::Result<Vec<(ID, SocketAddr)>> {
        let mut tx_stream = self.tx_stream.lock().unwrap();
        Ok(try!(container::unpack_peers(try!(read_container(&mut *tx_stream)))))
    }

    fn catch_error<F, T>(&self, default: T, f: F) -> io::Result<T>
        where F: FnOnce() -> io::Result<T>
    {
        match f() {
            Ok(value) => Ok(value),
            Err(ref error) => {
                if let Some(ref f) = self.handler.lock().unwrap().error {
                    f(self.peer_node_id, error);
                }
                Ok(default)
            }
        }
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
        match self.tx_stream.lock().unwrap().get_ref().shutdown(net::Shutdown::Both) {
            Ok(()) => {}
            Err(ref error) if error.kind() == io::ErrorKind::NotConnected => {}
            Err(ref error) => panic!(format!("{:?}", error)),
        }
        self.thread.take().unwrap().join().unwrap();

        (&*self.handler.lock().unwrap().drop)(self.peer_node_id);
    }
}

fn process_inbound_container(node_id: ID,
                             peer_node_id: ID,
                             rx_stream: &mut ssl::SslStream<net::TcpStream>,
                             tx_stream: &Arc<Mutex<ssl::SslStream<net::TcpStream>>>,
                             aknowledges_rx: &mpsc::Receiver<mpsc::Sender<()>>,
                             handler: &Arc<Mutex<Handler>>)
                             -> io::Result<()> {
    let container = try!(case_eof_to_aborted(read_container(rx_stream)));
    match container.get_kind() {
        message::Kind::AddServicesMessage => {
            (&*handler.lock()
                      .unwrap()
                      .add_services)(peer_node_id,
                                     try!(container::unpack_add_services(container)));
            {
                let mut tx_stream = tx_stream.lock().unwrap();
                try!(write_container(&mut *tx_stream, &container::pack_aknowledge()));
            }
        }
        message::Kind::RemoveServicesMessage => {
            (&*handler.lock()
                      .unwrap()
                      .remove_services)(peer_node_id,
                                        try!(container::unpack_remove_services(container)));
            {
                let mut tx_stream = tx_stream.lock().unwrap();
                try!(write_container(&mut *tx_stream, &container::pack_aknowledge()));
            }
        }
        message::Kind::AknowledgeMessage => {
            try!(container::unpack_aknowledge(container));
            let tx: mpsc::Sender<()> = aknowledges_rx.recv().unwrap();
            tx.send(()).unwrap();
        }
        message::Kind::RequestMessage => {
            let (request_id, name) = try!(container::unpack_request(container));

            let handler_clone = handler.clone();
            let reader = packet::Reader::new(rx_stream.try_clone().unwrap(), move |error| {
                if let Some(ref error_handler) = handler_clone.lock().unwrap().error {
                    error_handler(peer_node_id, &error);
                }
            });

            let mut response = (&*handler.lock()
                                         .unwrap()
                                         .request)(&name,
                                                   Box::new(reader::DrainOnDrop::new(reader)));

            {
                let mut tx_stream = tx_stream.lock().unwrap();

                try!(write_container(&mut *tx_stream,
                                     &container::pack_response(request_id, &response)));

                if let Ok(ref mut reader) = response {
                    try!(packet::copy(reader, &mut *tx_stream));
                }
                try!(tx_stream.flush());
            }
        }
        message::Kind::ResponseMessage => {
            let handler_clone = handler.clone();
            let reader = packet::Reader::new(rx_stream.try_clone().unwrap(), move |error| {
                if let Some(ref error_handler) = handler_clone.lock().unwrap().error {
                    error_handler(peer_node_id, &error);
                }
            });

            let (request_id, service_result) =
                try!(container::unpack_response(container,
                                                Box::new(reader::DrainOnDrop::new(reader))));
            try!((&*handler.lock().unwrap().response)(request_id, service_result));
        }
        _ => {
            error!("{}: got unexpected container {:?}",
                   node_id,
                   container.get_kind());
        }
    }
    Ok(())
}

fn write_container(mut w: &mut Write, container: &Container) -> io::Result<usize> {
    let bytes = try!(container.write_to_bytes());
    try!(writer::write_size(&mut w, bytes.len()));
    Ok(8 + try!(w.write(&bytes)))
}

fn read_container(mut stream: &mut io::Read) -> io::Result<Container> {
    let size = try!(reader::read_size(&mut stream));

    let mut bytes = iter::repeat(0u8).take(size).collect::<Vec<u8>>();
    try!(stream.read_exact(&mut bytes));

    Ok(try!(Container::parse_from_bytes(&bytes)))
}

fn case_eof_to_aborted<T>(result: io::Result<T>) -> io::Result<T> {
    match result {
        Ok(value) => Ok(value),
        Err(ref error) if error.kind() == io::ErrorKind::UnexpectedEof => {
            Err(io::Error::new(io::ErrorKind::ConnectionAborted, "connection lost"))
        }
        Err(error) => Err(error),
    }
}
