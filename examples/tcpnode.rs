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

use netzwerk::{
    Address,
    Config,
    Connections,
    Command, CommandReceiver, CommandSender,
    Event, EventTx,
    log::*,
    Message,
    Peer, PeerId, Peers,
    Protocol,
    tcp::*,
};

use common::*;

use async_std::task;
use crossbeam_channel as mpmc;
use futures::channel as mpsc;

mod common;

struct TcpNode {
    config: NodeConfig,
    peers: Peers,
    event_tx: Option<EventTx>,
}

impl TcpNode {
    pub fn from_config(config: NodeConfig) -> Self {
        let peers = config.peers();

        Self {
            config,
            peers,
            event_tx: None,
        }
    }

    pub fn id(&self) -> &String {
        &self.config.id
    }

    pub fn init(&mut self) {
        let event_tx = netzwerk::init(Config {});
    }

    pub fn add_peer(&self, peer: Peer) {
        if let Some(event_tx) = &self.event_tx {
            event_tx.send(Event::NewPeer { peer });
        }
    }

    pub fn drop_peer(&self, peer_id: PeerId) {
        if let Some(event_tx) = &self.event_tx {
            event_tx.send(Event::DropPeer { peer_id });
        }
    }

    pub fn shutdown(&self) {
        if let Some(event_tx) = &self.event_tx {
            event_tx.send(Event::Shutdown);
        }
    }

    pub fn send_msg(&self, message: impl Message, peer_id: PeerId) {
        if let Some(event_tx) = &self.event_tx {
            event_tx.send(Event::SendMessage { bytes: message.into_bytes(), peer_id });
        }
    }

    pub fn broadcast_msg(&self, message: impl Message) {
        if let Some(event_tx) = &self.event_tx {
            event_tx.send(Event::BroadcastMessage { bytes: message.into_bytes() });
        }
    }

    pub fn run(&self) {
        // TODO
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

    node.init();

    let msg = Utf8Message::new("hello netzwerk");

    node.broadcast_message_to_connected_peers(msg);

    std::thread::sleep(std::time::Duration::from_millis(3000));
    screen::exit();
}