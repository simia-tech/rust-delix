
use std::net::SocketAddr;
use std::io;

pub trait Transport : Send {
    fn bind(&self, SocketAddr) -> Result<(), io::Error>;
    fn join(&mut self, SocketAddr) -> Result<(), io::Error>;
    fn connection_count(&self) -> usize;
}

/*
type Result<T> = Result<T, Error>;

pub enum Error {
    IOError(io::Error)
}

impl From<io::Error> for Error {

}
*/
