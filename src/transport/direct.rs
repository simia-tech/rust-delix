
use std::net::{TcpListener, TcpStream, SocketAddr};
use std::io;
use std::sync::{Arc, Mutex};
use std::thread::spawn;

use transport::Transport;

pub struct Direct {
    connections: Arc<Mutex<usize>>,
}

impl Direct {

    pub fn new() -> Direct {
        Direct { connections: Arc::new(Mutex::new(0)) }
    }

}

impl Transport for Direct {

    fn bind(&self, address: SocketAddr) -> Result<(), io::Error> {
        let tcp_listener = try!(TcpListener::bind(address));
        println!("bound to address {:?}", address);

        let connections = self.connections.clone();
        spawn(move || {
            for connection in tcp_listener.incoming() {
                let connection = connection.unwrap();
                println!("got connection {}", connection.peer_addr().unwrap());
                *connections.lock().unwrap() += 1;
            }
        });

        Ok(())
    }

    fn join(&mut self, address: SocketAddr) -> Result<(), io::Error> {
        println!("join address {:?}", address);
        let stream = TcpStream::connect(address).unwrap();
        *self.connections.lock().unwrap() += 1;
        Ok(())
    }

    fn connection_count(&self) -> usize {
        *self.connections.lock().unwrap()
    }

}
