
use std::net::ToSocketAddrs;
use std::sync::{Arc, Mutex};
use std::thread::{JoinHandle, spawn, sleep_ms};

use discovery::Discovery;
use transport::Transport;

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum State {
    Started,
    Discovering,
    Joined,
}

pub struct Node {
    thread: Option<JoinHandle<()>>,
}

impl Node {

    pub fn new<A: ToSocketAddrs>(a: A, d: Box<Discovery>, t: Box<Transport>) -> Node {
        let address = a.to_socket_addrs().unwrap().next().expect("no socket addr");
        t.bind(address).unwrap();

        let discovery = Arc::new(Mutex::new(d));
        let transport = Arc::new(Mutex::new(t));

        let discovery_mutex = discovery.clone();
        let transport_mutex = transport.clone();

        let thread = spawn(move || {
            let mut discovery = discovery_mutex.lock().unwrap();
            let mut transport = transport_mutex.lock().unwrap();

            while transport.connection_count() == 0 {
                match discovery.discover() {
                    Some(address) => {
                        transport.join(address).unwrap();
                    },
                    None => {
                        println!("no address discovered - sleep 2s");
                        sleep_ms(2000);
                    }
                }
            }
        });

        Node { thread: Some(thread) }
    }

    pub fn state(&self) -> State {
        State::Joined
    }

}

impl Drop for Node {

    fn drop(&mut self) {
        self.thread.take().unwrap().join().unwrap();
        println!("joined thread");
    }

}
