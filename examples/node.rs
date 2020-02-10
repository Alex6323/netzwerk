// Run this with `cargo r --example node`

use netzwerk::{Address, Peer, Peers, Connections, Message, Protocol, Tcp, Udp, TcpBroker};

use config::NodeConfig;
use utf8msg::Utf8Message;

use async_std::task;
use crossbeam_channel as mpmc;

mod config;
mod utf8msg;

struct TcpOnlyNode {
    config: NodeConfig,
    peers: Peers,
    conns: TcpBroker,
}

impl TcpOnlyNode {
    pub fn from_config(config: NodeConfig) -> Self {
        let peers = config.peers();

        Self {
            config,
            peers,
            conns: TcpBroker::new(),

        }
    }

    pub fn run(&self) {
        /*
        task::spawn(async {
            self.conns.run().await;
        });
        */
    }

    pub fn broadcast_message_to_connected_peers(&mut self, msg: impl Message) {
        for (id, peer) in &*self.peers {
            if peer.is_connected() {
                // send message
            }
        }

    }
}

fn main() {
    //netzwerk::init();

    let config = NodeConfig::builder()
        .with_host_binding("localhost:1337")
        .with_peer("tcp://localhost:1338")
        .with_peer("tcp://localhost:1339")
        .build();

    let mut node = TcpOnlyNode::from_config(config);
    println!("created node");

    node.run();


    let msg = Utf8Message::new("hello netzwerk");

    node.broadcast_message_to_connected_peers(msg);
}