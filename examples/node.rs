//! This example shows how to create and run a TCP node using `netzwerk`.
//! You might want to run several instances of such a node in separate
//! terminals and connect those instances by specifying commandline arguments.
//!
//! ```bash
//! cargo r --example node -- --bind localhost:1337 --peers tcp://localhost:1338 --msg ping
//! cargo r --example node -- --bind localhost:1338 --peers tcp://localhost:1337 --msg pong
//! ```

use netzwerk::{
    Config,
    Command::*,
    Event,
    EventSubscriber as EventSub,
    Network,
    Peer, PeerId,
    Shutdown,
};

use common::*;

use async_std::task::{self, block_on};
use futures::prelude::*;
use log::*;
use structopt::StructOpt;

mod common;

fn main() {
    let args = Args::from_args();
    let config = args.make_config();

    logger::init(log::LevelFilter::Info);

    let (network, shutdown, receiver) = netzwerk::init(config.clone());

    let mut node = Node::builder()
        .with_config(config)
        .with_network(network.clone())
        .with_shutdown(shutdown)
        .build();

    task::spawn(notification_handler(receiver));

    let msg = Utf8Message::new(&args.msg);

    block_on(node.init());

    std::thread::spawn(|| spam(network, msg, 50, 1000));

    block_on(node.shutdown());
}

async fn notification_handler(mut bytes_received_events: EventSub) {
    while let Some(bytes_received_event) = bytes_received_events.next().await {
        logger::info("", &format!("[Node ] Received event {:?}", bytes_received_event));
        match bytes_received_event {
            Event::BytesReceived { num_bytes, from, bytes } => {
                info!("[Node ] Received {} from {}.", num_bytes, from);
            }
            _ => (),
        }
    }
}

struct Node {
    config: Config,
    network: Network,
    shutdown: Shutdown,
}

impl Node {

    pub async fn init(&mut self) {
        info!("[Node ] Initializing...");

        for peer in self.config.peers().values() {
            self.add_peer(peer.clone()).await;
        }

        info!("[Node ] Initialized");
    }

    pub async fn add_peer(&mut self, peer: Peer) {
        self.network.send(AddPeer { peer }).await;
    }

    pub async fn remove_peer(&mut self, peer_id: PeerId) {
        self.network.send(RemovePeer { peer_id }).await;
    }

    pub async fn send_msg(&mut self, message: Utf8Message, peer_id: PeerId) {
        self.network.send(SendBytes { to: peer_id, bytes: message.as_bytes() }).await;
    }

    pub async fn broadcast_msg(&mut self, message: Utf8Message) {
        self.network.send(BroadcastBytes { bytes: message.as_bytes() }).await;
    }

    pub async fn shutdown(mut self) {
        self.block_on_ctrl_c();

        info!("[Node ] Shutting down...");

        self.network.send(Shutdown).await;
        self.shutdown.finish_tasks().await;

        info!("[Node ] Complete. See you soon!");
    }

    fn block_on_ctrl_c(&self) {
        let mut rt = tokio::runtime::Runtime::new()
            .expect("[Node ] Error creating Tokio runtime");

        rt.block_on(tokio::signal::ctrl_c())
            .expect("[Node ] Error blocking on CTRL-C");
    }


    pub fn builder() -> NodeBuilder {
        NodeBuilder::new()
    }

}

fn spam(mut network: Network, msg: Utf8Message, num: usize, interval: u64) {
    info!("[Node ] Starting spammer: {:?} messages", num);
    task::block_on(async move {
        for _ in 0..num {
            task::sleep(std::time::Duration::from_millis(interval)).await;
            network.send(BroadcastBytes { bytes: msg.as_bytes() }).await;
        }
    });
    info!("[Node ] Stopping spammer");
}

struct NodeBuilder {
    config: Option<Config>,
    network: Option<Network>,
    shutdown: Option<Shutdown>,
}

impl NodeBuilder {
    pub fn new() -> Self {
        Self {
            config: None,
            network: None,
            shutdown: None,
        }
    }

    pub fn with_config(mut self, config: Config) -> Self {
        self.config.replace(config);
        self
    }

    pub fn with_network(mut self, network: Network) -> Self {
        self.network.replace(network);
        self
    }

    pub fn with_shutdown(mut self, shutdown: Shutdown) -> Self {
        self.shutdown.replace(shutdown);
        self
    }

    pub fn build(self) -> Node {
        Node {
            config: self.config
                .expect("[Node ] No config provided"),

            network: self.network
                .expect("[Node ] No network instance provided"),

            shutdown: self.shutdown
                .expect("[Node ] No shutdown instance provided"),
        }
    }
}