//! This example shows how two TCP nodes follow a predefined policy to drop a duplicate
//! connection, and end up agreeing on one single TCP connection between them.
//!
//! You might want to run several instances of such a node in separate
//! terminals and connect those instances by specifying commandline arguments.
//!
//! ```bash
//! cargo r --example duplicate -- --bind localhost:1337 --peers tcp://localhost:1338 --msg ping
//! cargo r --example duplicate -- --bind localhost:1338 --peers tcp://localhost:1337 --msg pong
//! ```

use netzwerk::{
    Config,
    Command::*,
    Event,
    EventSubscriber as Events,
    Network,
    Peer, PeerId,
    Shutdown,
};

use common::*;

use async_std::task::{self, block_on, spawn};
use async_std::prelude::*;
use futures::channel::oneshot;
use futures::{select, FutureExt};
use futures::sink::SinkExt;
use log::*;
use serde::{Serialize, Deserialize};
use structopt::StructOpt;

mod common;

fn main() {
    let args = Args::from_args();
    let config = args.make_config();

    logger::init(log::LevelFilter::Info);

    let (network, shutdown, events) = netzwerk::init(config.clone());

    let mut node = Node::builder()
        .with_config(config)
        .with_network(network.clone())
        .with_shutdown(shutdown)
        .with_events(events)
        .build();


    block_on(node.init());
    block_on(node.run());
}

#[derive(Serialize, Deserialize)]
struct Handshake {
    server_port: u16,
}

impl Handshake {
    pub fn serialize(&self) -> Vec<u8> {
        bincode::serialize(self).unwrap()
    }
}

enum NodeState {
    Initializing,
    Running,
    Stopping,
}

struct Node {
    config: Config,
    network: Network,
    shutdown: Shutdown,
    events: Events,
    state: NodeState,
}

impl Node {

    pub async fn init(&mut self) {

        info!("[Node ] Initializing...");
        self.state = NodeState::Initializing;

        for peer in self.config.peers().values() {
            self.add_peer(peer.clone()).await;
        }

        info!("[Node ] Initialized");
    }

    pub async fn run(mut self) {
        self.state = NodeState::Running;

        info!("[Node ] Running...");
        block_on(self.event_loop());
    }

    async fn event_loop(mut self) {
        let shutdown_rx = &mut self.shutdown_rx();

        loop {
            select! {
                event = self.events.next().fuse() => {
                    let event = event.unwrap();

                    match event {
                        Event::PeerConnected { peer_id, num_conns, timestamp } => {
                            error!("Stay calm! I just needed the red color: Event::PeerConnected");
                        }
                        Event::BytesReceived { from_peer, num_bytes, buffer, .. } => {
                            info!("[Node ] Received: '{}' from peer {}",
                                Utf8Message::from_bytes(&buffer[0..num_bytes]),
                                from_peer);
                        }
                        _ => (),
                    }
                },
                shutdown = shutdown_rx.fuse() => {
                    break;
                }
            }
        }

        info!("[Node ] Shutting down...");
        self.state = NodeState::Stopping;

        self.network.send(Shutdown).await;
        self.shutdown.finish_tasks().await;

        info!("[Node ] Complete. See you soon!");
    }

    pub async fn add_peer(&mut self, peer: Peer) {
        self.network.send(AddPeer { peer, connect_attempts: Some(5) }).await;
    }

    pub async fn remove_peer(&mut self, peer_id: PeerId) {
        self.network.send(RemovePeer { peer_id }).await;
    }

    pub async fn connect(&mut self, peer_id: PeerId) {
        self.network.send(Connect { peer_id,  num_retries: 5 }).await;
    }

    pub async fn send_handshake(&mut self, handshake: Handshake, peer_id: PeerId) {
        self.network.send(SendBytes { to_peer: peer_id, bytes: handshake.serialize() }).await;
    }

    fn shutdown_rx(&self) -> oneshot::Receiver<()> {
        let (sender, receiver) = oneshot::channel();

        spawn(async move {
            let mut rt = tokio::runtime::Runtime::new()
                .expect("[Node ] Error creating Tokio runtime");

            rt.block_on(tokio::signal::ctrl_c())
                .expect("[Node ] Error blocking on CTRL-C");

            sender.send(());
        });

        receiver
    }


    pub fn builder() -> NodeBuilder {
        NodeBuilder::new()
    }

}

struct NodeBuilder {
    config: Option<Config>,
    network: Option<Network>,
    shutdown: Option<Shutdown>,
    events: Option<Events>,
}

impl NodeBuilder {
    pub fn new() -> Self {
        Self {
            config: None,
            network: None,
            shutdown: None,
            events: None,
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

    pub fn with_events(mut self, events: Events) -> Self {
        self.events.replace(events);
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

            events: self.events
                .expect("[Node ] No events instance provided"),

            state: NodeState::Initializing,
        }
    }
}