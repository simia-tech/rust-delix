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

extern crate net2;

use std::net::{self, Ipv4Addr, SocketAddr};
use std::io;
use std::thread;
use std::sync::{Mutex, mpsc};

use self::net2::UdpSocketExt;

use super::Discovery;

const PACKET_SIZE: usize = 16;

type Packet = [u8; PACKET_SIZE];

#[derive(Debug)]
enum Kind {
    Ask,
    Tell,
}

pub struct Multicast {
    udp_socket: net::UdpSocket,
    multicast_address: SocketAddr,
    public_address: SocketAddr,
    tx: Mutex<mpsc::Sender<mpsc::Sender<SocketAddr>>>,
}

impl Multicast {
    pub fn new(interface_address: SocketAddr,
               multicast_address: SocketAddr,
               public_address: SocketAddr)
               -> io::Result<Self> {
        let any_ip = Ipv4Addr::new(0, 0, 0, 0);

        let udp_socket = try!(net::UdpSocket::bind(interface_address));

        match multicast_address {
            SocketAddr::V4(ref address) => {
                try!(udp_socket.set_multicast_loop_v4(true));
                try!(udp_socket.join_multicast_v4(address.ip(), &any_ip));
            }
            SocketAddr::V6(_) => panic!("ip v6 is not implemented yet"),
        }

        let udp_socket_clone = udp_socket.try_clone().unwrap();
        let (tx, rx) = mpsc::channel();
        thread::spawn(move || {
            loop {
                match receive_packet(&udp_socket_clone) {
                    Ok(tuple) => {
                        match tuple {
                            (Kind::Ask, address, sender_address) if address != public_address => {
                                send_packet(&udp_socket_clone,
                                            sender_address,
                                            Kind::Tell,
                                            public_address)
                                    .unwrap();
                            }
                            (Kind::Tell, address, _) => {
                                match rx.try_recv() as Result<mpsc::Sender<SocketAddr>,
                                                              mpsc::TryRecvError> {
                                    Ok(tx) => tx.send(address).unwrap(),
                                    Err(mpsc::TryRecvError::Empty) => {}
                                    Err(error) => panic!(error),
                                }
                            }
                            (kind, address, sender_address) => {
                                debug!("{}: got {:?} / {} from {}",
                                       public_address,
                                       kind,
                                       address,
                                       sender_address);
                            }
                        }
                    }
                    Err(error) => {
                        error!("error: {:?}", error);
                    }
                }
            }
        });

        Ok(Multicast {
            udp_socket: udp_socket,
            multicast_address: multicast_address,
            public_address: public_address,
            tx: Mutex::new(tx),
        })
    }
}

impl Discovery for Multicast {
    fn next(&self) -> Option<SocketAddr> {
        let (tx, rx) = mpsc::channel();
        self.tx.lock().unwrap().send(tx).unwrap();

        send_packet(&self.udp_socket,
                    self.multicast_address,
                    Kind::Ask,
                    self.public_address)
            .unwrap();

        Some(rx.recv().unwrap())
    }
}

fn send_packet(udp_socket: &net::UdpSocket,
               destination_address: SocketAddr,
               kind: Kind,
               address: SocketAddr)
               -> io::Result<()> {
    let packet = pack(kind, address);
    try!(udp_socket.send_to(&packet, destination_address));
    Ok(())
}

fn receive_packet(udp_socket: &net::UdpSocket) -> io::Result<(Kind, SocketAddr, SocketAddr)> {
    let mut packet: Packet = [0; PACKET_SIZE];
    let (_, sender_address) = try!(udp_socket.recv_from(&mut packet));
    let (kind, address) = unpack(&packet);
    Ok((kind, address, sender_address))
}

fn pack(kind: Kind, address: SocketAddr) -> Packet {
    let mut p: Packet = [0; PACKET_SIZE];

    p[0] = match kind {
        Kind::Ask => 0,
        Kind::Tell => 1,
    };

    if let SocketAddr::V4(address_v4) = address {
        let ip_bytes = address_v4.ip().octets();
        let port = address_v4.port();
        p[1] = ip_bytes[0];
        p[2] = ip_bytes[1];
        p[3] = ip_bytes[2];
        p[4] = ip_bytes[3];
        p[5] = ((port & 0xff00) >> 8) as u8;
        p[6] = ((port & 0x00ff) >> 0) as u8;
    }

    p
}

fn unpack(p: &Packet) -> (Kind, SocketAddr) {
    let kind = match p[0] {
        0 => Kind::Ask,
        1 => Kind::Tell,
        _ => unreachable!(),
    };
    let ip_address = Ipv4Addr::new(p[1], p[2], p[3], p[4]);
    let port = ((p[5] as u16) << 8) | ((p[6] as u16) << 0);
    (kind,
     SocketAddr::V4(net::SocketAddrV4::new(ip_address, port)))
}

#[cfg(test)]
mod tests {

    use std::net::SocketAddr;
    use super::Multicast;
    use super::super::Discovery;

    #[test]
    fn discovery_with_two_nodes() {
        let address_one = "127.0.0.1:3001".parse::<SocketAddr>().unwrap();
        let discovery_one = Multicast::new("0.0.0.0:4001".parse::<SocketAddr>().unwrap(),
                                           "224.0.0.1:4002".parse::<SocketAddr>().unwrap(),
                                           address_one)
                                .unwrap();

        let address_two = "127.0.0.1:3002".parse::<SocketAddr>().unwrap();
        let discovery_two = Multicast::new("0.0.0.0:4002".parse::<SocketAddr>().unwrap(),
                                           "224.0.0.1:4001".parse::<SocketAddr>().unwrap(),
                                           address_two)
                                .unwrap();

        assert_eq!(Some(address_two), discovery_one.next());
        assert_eq!(Some(address_one), discovery_two.next());
    }

    #[test]
    fn discovery_with_three_nodes() {
        let address_one = "127.0.0.1:3011".parse::<SocketAddr>().unwrap();
        let discovery_one = Multicast::new("0.0.0.0:4011".parse::<SocketAddr>().unwrap(),
                                           "224.0.0.2:4012".parse::<SocketAddr>().unwrap(),
                                           address_one)
                                .unwrap();

        let address_two = "127.0.0.1:3012".parse::<SocketAddr>().unwrap();
        let discovery_two = Multicast::new("0.0.0.0:4012".parse::<SocketAddr>().unwrap(),
                                           "224.0.0.2:4011".parse::<SocketAddr>().unwrap(),
                                           address_two)
                                .unwrap();

        let address_three = "127.0.0.1:3013".parse::<SocketAddr>().unwrap();
        let discovery_three = Multicast::new("0.0.0.0:4013".parse::<SocketAddr>().unwrap(),
                                             "224.0.0.2:4011".parse::<SocketAddr>().unwrap(),
                                             address_three)
                                  .unwrap();

        assert_eq!(Some(address_two), discovery_one.next());
        assert_eq!(Some(address_one), discovery_two.next());
        assert_eq!(Some(address_one), discovery_three.next());
    }

}
