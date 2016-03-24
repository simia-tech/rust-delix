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
use std::net::{self, SocketAddr};
use std::result;
use std::sync::{Arc, Mutex, mpsc};
use std::thread;

use message;
use node::{ID, request, service};
use super::packet;
use super::dispatcher::Dispatcher;
use super::container::{self, Container};
use super::super::cipher::{self, Cipher};

pub struct Connection {
    tx_stream: Arc<Mutex<cipher::Stream<net::TcpStream>>>,
    thread: Option<thread::JoinHandle<()>>,

    node_id: ID,
    peer_node_id: ID,
    peer_public_address: SocketAddr,

    aknowledges_tx: Mutex<mpsc::Sender<mpsc::Sender<()>>>,

    error_handler: Arc<Mutex<Option<Box<Fn(ID, &io::Error) + Send>>>>,
    drop_handler: Box<Fn(ID) + Send + Sync>,
}

pub struct Handlers {
    pub add_services: Box<Fn(ID, Vec<String>) + Send>,
    pub remove_services: Box<Fn(ID, Vec<String>) + Send>,
    pub request: Box<Fn(ID, u32, &str, Box<request::Reader>) + Send + 'static>,
    pub response: Box<Fn(u32, service::Result) -> result::Result<(), io::Error> + Send>,
    pub drop: Box<Fn(ID) + Send + Sync>,
}

impl Connection {
    pub fn new_inbound(stream: cipher::Stream<net::TcpStream>,
                       node_id: ID,
                       public_address: SocketAddr,
                       peers: &[(ID, SocketAddr)],
                       handlers: Handlers)
                       -> io::Result<Connection> {

        let (connection, sender) = try!(Self::new(stream, node_id, public_address, handlers));

        try!(connection.send_peers(peers));
        sender.send(true).unwrap();

        Ok(connection)
    }

    pub fn new_outbound(stream: cipher::Stream<net::TcpStream>,
                        node_id: ID,
                        public_address: SocketAddr,
                        handlers: Handlers)
                        -> io::Result<(Connection, Vec<(ID, SocketAddr)>)> {

        let (connection, sender) = try!(Self::new(stream, node_id, public_address, handlers));

        let peers = try!(connection.receive_peers());
        sender.send(true).unwrap();

        Ok((connection, peers))
    }

    fn new(stream: cipher::Stream<net::TcpStream>,
           node_id: ID,
           public_address: SocketAddr,
           handlers: Handlers)
           -> io::Result<(Connection, mpsc::Sender<bool>)> {

        let tx_stream = Arc::new(Mutex::new(stream.try_clone().unwrap()));
        let tx_stream_clone = tx_stream.clone();
        let mut rx_stream = stream;

        let (aknowledges_tx, aknowledges_rx) = mpsc::channel();

        let Handlers{ add_services: add_services_handler,
                      remove_services: remove_services_handler,
                      request: request_handler,
                      response: response_handler,
                      drop: drop_handler } = handlers;
        let error_handler: Arc<Mutex<Option<Box<Fn(ID, &io::Error) + Send>>>> =
            Arc::new(Mutex::new(None));
        let error_handler_clone = error_handler.clone();

        let (peer_node_id, peer_public_address) = {
            let mut tx_stream = tx_stream.lock().unwrap();
            try!(container::pack_introduction(node_id, public_address).write(&mut *tx_stream));
            try!(container::unpack_introduction(try!(Container::read(&mut *tx_stream))))
        };

        let (sender, receiver) = mpsc::channel();
        let thread = Some(thread::spawn(move || {
            receiver.recv().unwrap();
            let request_dispatcher = Dispatcher::new();
            let response_dispatcher = Dispatcher::new();
            loop {
                match process_inbound_container(node_id,
                                                peer_node_id,
                                                &mut rx_stream,
                                                &tx_stream_clone,
                                                &aknowledges_rx,
                                                &request_dispatcher,
                                                &response_dispatcher,
                                                &add_services_handler,
                                                &remove_services_handler,
                                                &request_handler,
                                                &response_handler) {
                    Ok(()) => {}
                    Err(ref error) => {
                        if let Some(error_handler) = error_handler_clone.lock().unwrap().take() {
                            error_handler(peer_node_id, error);
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
            error_handler: error_handler,
            drop_handler: drop_handler,
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

    pub fn set_error_handler(&self, f: Box<Fn(ID, &io::Error) + Send>) {
        *self.error_handler.lock().unwrap() = Some(f);
    }

    pub fn clear_error_handler(&self) {
        *self.error_handler.lock().unwrap() = None;
    }

    pub fn send_add_services(&self, service_names: &[String]) -> io::Result<()> {
        let (tx, rx) = mpsc::channel();
        self.aknowledges_tx.lock().unwrap().send(tx).unwrap();
        {
            let mut tx_stream = self.tx_stream.lock().unwrap();
            try!(container::pack_add_services(service_names).write(&mut *tx_stream));
        }
        rx.recv().unwrap();
        Ok(())
    }

    pub fn send_remove_services(&self, service_names: &[String]) -> io::Result<()> {
        let (tx, rx) = mpsc::channel();
        self.aknowledges_tx.lock().unwrap().send(tx).unwrap();
        {
            let mut tx_stream = self.tx_stream.lock().unwrap();
            try!(container::pack_remove_services(service_names).write(&mut *tx_stream));
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
            {
                let mut tx_stream = self.tx_stream.lock().unwrap();
                try!(container::pack_request(id, name).write(&mut *tx_stream));
            }

            try!(packet::request::copy(id, reader, |buffer| {
                let mut tx_stream = self.tx_stream.lock().unwrap();
                try!(tx_stream.write(buffer));
                Ok(buffer.len())
            }));

            Ok(())
        })
    }

    pub fn send_response(&self,
                         request_id: u32,
                         mut service_result: service::Result)
                         -> io::Result<()> {
        self.catch_error((), || {
            {
                let mut tx_stream = self.tx_stream.lock().unwrap();
                try!(container::pack_response(request_id, &service_result).write(&mut *tx_stream));
            }

            if let Ok(ref mut reader) = service_result {
                try!(packet::response::copy(request_id, reader, |buffer| {
                    let mut tx_stream = self.tx_stream.lock().unwrap();
                    try!(tx_stream.write_all(buffer));
                    Ok(buffer.len())
                }));
            }

            Ok(())
        })
    }

    pub fn shutdown(&self) {
        match self.tx_stream.lock().unwrap().get_ref().shutdown(net::Shutdown::Both) {
            Ok(()) => {}
            Err(ref error) if error.kind() == io::ErrorKind::NotConnected => {}
            Err(ref error) => panic!(format!("{:?}", error)),
        }
    }

    fn send_peers(&self, peers: &[(ID, SocketAddr)]) -> io::Result<()> {
        let mut tx_stream = self.tx_stream.lock().unwrap();
        try!(container::pack_peers(peers).write(&mut *tx_stream));
        Ok(())
    }

    fn receive_peers(&self) -> io::Result<Vec<(ID, SocketAddr)>> {
        let mut tx_stream = self.tx_stream.lock().unwrap();
        Ok(try!(container::unpack_peers(try!(Container::read(&mut *tx_stream)))))
    }

    fn catch_error<F, T>(&self, default: T, f: F) -> io::Result<T>
        where F: FnOnce() -> io::Result<T>
    {
        match f() {
            Ok(value) => Ok(value),
            Err(ref error) => {
                if let Some(error_handler) = self.error_handler.lock().unwrap().take() {
                    error_handler(self.peer_node_id, error);
                } else {
                    error!("got error but no handler: {:?}", error);
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
        if let Some(join_handle) = self.thread.take() {
            self.shutdown();
            join_handle.join().unwrap();
            (self.drop_handler)(self.peer_node_id);
        }
    }
}

fn process_inbound_container(node_id: ID,
                             peer_node_id: ID,
                             rx_stream: &mut cipher::Stream<net::TcpStream>,
                             tx_stream: &Arc<Mutex<cipher::Stream<net::TcpStream>>>,
                             aknowledges_rx: &mpsc::Receiver<mpsc::Sender<()>>,
                             request_dispatcher: &Dispatcher,
                             response_dispatcher: &Dispatcher,
                             add_services_handler: &Box<Fn(ID, Vec<String>) + Send>,
                             remove_services_handler: &Box<Fn(ID, Vec<String>) + Send>,
                             request_handler: &Box<Fn(ID, u32, &str, Box<request::Reader>) + Send + 'static>,
                             response_handler: &Box<Fn(u32, service::Result) -> result::Result<(), io::Error> + Send>)
                             -> io::Result<()> {
    let container = try!(cast_eof_to_aborted(Container::read(rx_stream)));
    match container.get_kind() {
        message::Kind::AddServicesMessage => {
            add_services_handler(peer_node_id,
                                 try!(container::unpack_add_services(container)));
            {
                let mut tx_stream = tx_stream.lock().unwrap();
                try!(container::pack_aknowledge().write(&mut *tx_stream));
            }
        }
        message::Kind::RemoveServicesMessage => {
            remove_services_handler(peer_node_id,
                                    try!(container::unpack_remove_services(container)));
            {
                let mut tx_stream = tx_stream.lock().unwrap();
                try!(container::pack_aknowledge().write(&mut *tx_stream));
            }
        }
        message::Kind::AknowledgeMessage => {
            try!(container::unpack_aknowledge(container));
            let tx: mpsc::Sender<()> = aknowledges_rx.recv().unwrap();
            tx.send(()).unwrap();
        }
        message::Kind::RequestMessage => {
            let (request_id, name) = try!(container::unpack_request(container));

            let reader = request_dispatcher.begin(request_id);

            request_handler(peer_node_id, request_id, &name, reader);
        }
        message::Kind::RequestPacketMessage => {
            let (request_id, result) = try!(container::unpack_packet(container));

            request_dispatcher.dispatch(request_id, result).unwrap();
        }
        message::Kind::ResponseMessage => {
            let (request_id, service_result) =
                try!(container::unpack_response(container, Box::new(io::Cursor::new(Vec::new()))));

            let reader = response_dispatcher.begin(request_id);

            let service_result = match service_result {
                Ok(_) => Ok(reader),
                Err(error) => Err(error),
            };

            try!(response_handler(request_id, service_result));
        }
        message::Kind::ResponsePacketMessage => {
            let (request_id, result) = try!(container::unpack_packet(container));

            response_dispatcher.dispatch(request_id, result).unwrap();
        }
        _ => {
            error!("{}: got unexpected container {:?}",
                   node_id,
                   container.get_kind());
        }
    }
    Ok(())
}

fn cast_eof_to_aborted<T>(result: io::Result<T>) -> io::Result<T> {
    match result {
        Ok(value) => Ok(value),
        Err(ref error) if error.kind() == io::ErrorKind::UnexpectedEof => {
            Err(io::Error::new(io::ErrorKind::ConnectionAborted, "connection aborted"))
        }
        Err(error) => Err(error),
    }
}
