
use std::collections::HashMap;
use std::net::{TcpListener, TcpStream, SocketAddr};
use std::io;
use std::sync::{Arc, Mutex};
use std::thread::spawn;

use transport::Transport;
use transport::direct::Connection;

pub struct Direct {
    connections: Arc<Mutex<HashMap<SocketAddr, Connection>>>,
}

impl Direct {

    pub fn new() -> Direct {
        Direct { connections: Arc::new(Mutex::new(HashMap::new())) }
    }

}

impl Transport for Direct {

    fn bind(&self, address: SocketAddr) -> Result<(), io::Error> {
        let tcp_listener = try!(TcpListener::bind(address));
        println!("bound to address {:?}", address);

        let connections = self.connections.clone();
        spawn(move || {
            for stream in tcp_listener.incoming() {
                let stream = stream.unwrap();
                let connection = Connection::new(stream);
                println!("got connection {}", connection);

                connections.lock().unwrap().insert(connection.peer_addr(), connection);
            }
        });

        Ok(())
    }

    fn join(&mut self, address: SocketAddr) -> Result<(), io::Error> {
        println!("join address {:?}", address);
        let stream = TcpStream::connect(address).unwrap();
        let connection = Connection::new(stream);
        self.connections.lock().unwrap().insert(connection.peer_addr(), connection);
        Ok(())
    }

    fn connection_count(&self) -> usize {
        self.connections.lock().unwrap().len()
    }

}
