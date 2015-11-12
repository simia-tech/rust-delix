
extern crate delix;

use std::thread::sleep_ms;

use delix::discovery::Constant;
use delix::node::{Node, State};
use delix::transport::Direct;

#[test]
fn discovery() {
    let discovery_one = Box::new(Constant::new(&[] as &[&str]));
    let transport_one = Box::new(Direct::new());
    let node_one = Node::new("127.0.0.1:3001", discovery_one, transport_one).unwrap();

    let discovery_two = Box::new(Constant::new(&["127.0.0.1:3001"]));
    let transport_two = Box::new(Direct::new());
    let node_two = Node::new("127.0.0.1:3002", discovery_two, transport_two).unwrap();

    sleep_ms(100);

    assert_eq!(State::Joined, node_one.state());
    assert_eq!(State::Joined, node_two.state());
}
