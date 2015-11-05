
use std::net::{SocketAddr, ToSocketAddrs};

use discovery::Discovery;

pub struct Constant {
    addresses: Vec<SocketAddr>,
    current_index: usize,
}

impl Constant {

    pub fn new<T: ToSocketAddrs>(inputs: &[T]) -> Constant {
        let mut addresses = Vec::new();
        for input in inputs {
            for socket_addr in input.to_socket_addrs().unwrap() {
                addresses.push(socket_addr);
            }
        }
        Constant {
            addresses: addresses,
            current_index: 0,
        }
    }

}

impl Discovery for Constant {

    fn discover(&mut self) -> Option<SocketAddr> {
        let result = self.addresses.get(self.current_index);
        self.current_index += 1;
        if self.current_index >= self.addresses.len() {
            self.current_index = 0;
        }
        match result {
            None => None,
            Some(result) => Some(*result),
        }
    }

}
