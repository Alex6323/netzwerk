//! This example shows how to create and run a TCP node using `netzwerk`.
//! You might want to run several instances of such a node in separate
//! terminals and connect those instances by specifying commandline arguments.
//!
//! To set up a node id'ed with "A" listening at `localhost:1337` and with
//! two peers located on localhost 1338, 1339 pass this to `cargo`:
//!
//! ```bash
//! cargo r --example tcpnode -- --id A --bind localhost:1337 --peers tcp://localhost:1338 tcp://localhost:1339
//! ```

use netzwerk::{Address, Peer, Peers, Connections, Message, Protocol, Tcp, Udp, TcpBroker};
use netzwerk::log::*;

use common::*;

use async_std::task;
use crossbeam_channel as mpmc;

mod common;

struct TcpNode {
    config: NodeConfig,
    peers: Peers,
    conns: TcpBroker,
}

impl TcpNode {
    pub fn from_config(config: NodeConfig) -> Self {
        netzwerk::init();

        let peers = config.peers();

        Self {
            config,
            peers,
            conns: TcpBroker::new(),

        }
    }

    pub fn id(&self) -> &String {
        &self.config.id
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
    let config = NodeConfig::from_args();

    logger::init(log::LevelFilter::Debug);
    screen::init();

    let mut node = TcpNode::from_config(config);
    logger::info(&format!("Created node <<{}>>", node.id()));

    node.run();

    let msg = Utf8Message::new("hello netzwerk");

    node.broadcast_message_to_connected_peers(msg);

    std::thread::sleep(std::time::Duration::from_millis(3000));
    screen::exit();
}