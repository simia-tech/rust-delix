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

use std::io;
use std::net::{self, SocketAddr, ToSocketAddrs};

use util::thread;

pub struct TcpServer {
    local_address: SocketAddr,
    thread: thread::Bound,
}

impl TcpServer {
    pub fn bind<T, F>(address: T, mut handler_factory: F) -> io::Result<Self>
        where T: ToSocketAddrs,
              F: FnMut(net::TcpStream) -> Box<FnMut() + Send> + Send + 'static
    {
        let tcp_listener = try!(net::TcpListener::bind(address));
        let local_address = tcp_listener.local_addr().unwrap();

        let thread = thread::Bound::spawn(move |running| {
            let mut threads = Vec::new();
            for stream in tcp_listener.incoming() {
                if !*running.read().unwrap() {
                    break;
                }

                let mut handler = handler_factory(stream.unwrap());
                threads.push(thread::Bound::spawn(move |_| {
                    handler();
                }));
            }
        });

        Ok(TcpServer {
            local_address: local_address,
            thread: thread,
        })
    }

    pub fn local_address(&self) -> SocketAddr {
        self.local_address
    }
}

impl Drop for TcpServer {
    fn drop(&mut self) {
        self.thread.shutdown();
        let _ = net::TcpStream::connect(self.local_address());
    }
}

#[cfg(test)]
mod tests {

    use std::io::{self, BufRead, Read, Write};
    use std::net::{self, SocketAddr};
    use std::thread;
    use std::sync::mpsc;
    use super::TcpServer;

    fn build_echo_server() -> TcpServer {
        TcpServer::bind("localhost:0", |mut stream| {
            Box::new(move || {
                let line = {
                    let mut stream = io::BufReader::new(&stream);
                    let mut line = String::new();
                    stream.read_line(&mut line).unwrap();
                    line
                };
                stream.write_all(line.as_bytes()).unwrap();
            })
        })
            .unwrap()
    }

    fn connect_and_send(address: SocketAddr, text: &str) -> io::Result<String> {
        let mut client = try!(net::TcpStream::connect(address));
        try!(write!(client, "{}", text));

        let mut response = String::new();
        assert_eq!(text.len(), try!(client.read_to_string(&mut response)));

        Ok(response)
    }

    #[test]
    fn simple_echo() {
        let server = build_echo_server();
        let response = connect_and_send(server.local_address(), "hello\n").unwrap();
        assert_eq!("hello\n", response);
    }

    #[test]
    fn multiple_parallel_echos() {
        let server = build_echo_server();

        let address = server.local_address();
        let rx = {
            let (tx, rx) = mpsc::channel();
            for _ in 0..10 {
                let tx = tx.clone();
                thread::spawn(move || {
                    let response = connect_and_send(address, "hello\n").unwrap();
                    assert_eq!("hello\n", response);
                    tx.send(1).unwrap();
                });
            }
            rx
        };

        let mut total = 0;
        for result in rx {
            total += result;
        }
        assert_eq!(10, total);
    }

}
