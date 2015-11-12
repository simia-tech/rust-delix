
use std::net::SocketAddr;
use std::io;
use std::result;

pub trait Transport : Send {
    fn bind(&self, SocketAddr) -> Result<()>;
    fn join(&mut self, SocketAddr) -> Result<()>;
    fn connection_count(&self) -> usize;
}

pub type Result<T> = result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    IO(io::Error)
}

impl From<io::Error> for Error {

    fn from(error: io::Error) -> Self {
        Error::IO(error)
    }

}
