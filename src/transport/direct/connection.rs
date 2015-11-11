
use std::fmt;
use std::net::{SocketAddr, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread::{JoinHandle, spawn};

use protobuf::Message as Message_imported_for_functions;
use message::{Container, Kind, NodeAdd};

pub struct Connection {
    stream: Arc<Mutex<TcpStream>>,
    thread: Option<JoinHandle<()>>,
}

impl Connection {

    pub fn new(s: TcpStream) -> Connection {
        let stream_mutex = Arc::new(Mutex::new(s));

        let stream = stream_mutex.clone();
        let thread = spawn(move || {
            let mut buffer = Vec::new();
            let mut node_add = NodeAdd::new();
            node_add.set_address(vec![0, 1, 2, 3]);
            node_add.write_to_vec(&mut buffer).unwrap();

            let mut container = Container::new();
            container.set_kind(Kind::NodeAddMessage);
            container.set_payload(buffer);

            container.write_to_writer(&mut *stream.lock().unwrap()).unwrap();
        });

        Connection {
            stream: stream_mutex,
            thread: Some(thread),
        }
    }

    pub fn peer_addr(&self) -> SocketAddr {
        (*self.stream.lock().unwrap()).peer_addr().unwrap()
    }

}

impl fmt::Display for Connection {

    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "(-> {})", self.peer_addr())
    }

}

impl Drop for Connection {

    fn drop(&mut self) {
        self.thread.take().unwrap().join().unwrap();
    }

}
