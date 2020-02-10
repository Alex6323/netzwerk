// Run this with `cargo r --example node`

use netzwerk::api;
use netzwerk::{Peers, Connections, Tcp};

struct NetzwerkNode {
    peers: Peers,
    tcp_conns: Connections<Tcp>,
    udp_conns: Connections<Udp>,
}

impl NetzwerkNode {
    pub fn new() -> Self {
        Self {
            peers: Peers::new(),
            tcp_conns: Connections::new(),
            udp_conns: Connections::new(),
        }
    }
}

fn main() {
    api::init();

    let mut node = NetzwerkNode::new();

    println!("hello");
}