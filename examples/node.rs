//! This example shows how to create and run a TCP node using `netzwerk`.
//! You might want to run several instances of such a node in separate
//! terminals and connect those instances by specifying commandline arguments.
//!
//! Run it with `cargo r --example node`

use netzwerk::{Address, Peer, Peers, Connections, Message, Protocol, Tcp, Udp, TcpBroker};
use netzwerk::log::*;

use config::NodeConfig;
use utf8msg::Utf8Message;

use async_std::task;
use crossbeam_channel as mpmc;

mod config;
mod logger;
mod screen;
mod utf8msg;

struct TcpOnlyNode {
    config: NodeConfig,
    peers: Peers,
    conns: TcpBroker,
}

impl TcpOnlyNode {
    pub fn from_config(config: NodeConfig) -> Self {
        netzwerk::init();

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
    logger::init(log::LevelFilter::Debug);
    screen::init();

    let config = NodeConfig::builder()
        .with_host_binding("localhost:1337")
        .with_peer("tcp://localhost:1338")
        .with_peer("tcp://localhost:1339")
        .build();

    let mut node = TcpOnlyNode::from_config(config);
    logger::info("created node");

    node.run();

    let msg = Utf8Message::new("hello netzwerk");

    node.broadcast_message_to_connected_peers(msg);

    std::thread::sleep(std::time::Duration::from_millis(3000));
    screen::exit();
}