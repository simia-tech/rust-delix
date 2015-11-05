
use std::net::SocketAddr;

pub trait Discovery : Send {
    fn discover(&mut self) -> Option<SocketAddr>;
}
