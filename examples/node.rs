//! This example shows how to create and run a TCP node using `netzwerk`.
//! You might want to run several instances of such a node in separate
//! terminals and connect those instances by specifying commandline arguments.
//!
//! To set up a node id'ed with "A" listening at `localhost:1337` and with
//! two TCP peers located at localhost 1338, localhost 1339 pass this to `cargo`:
//!
//! ```bash
//! cargo r --example node -- --id A --bind localhost:1337 --peers tcp://localhost:1338 tcp://localhost:1339 --msg hello
//! ```

use netzwerk::{
    Config,
    Connections,
    Command::*,
    Event,
    EventSubscriber as EventSub,
    log::*,
    NetControl,
    Peer, PeerId,
    Protocol,
    tcp::*,
};

use common::*;

use async_std::task::{self, spawn, block_on};
use futures::stream::Stream;
use futures::*;
use futures::prelude::*;
use futures::future::Future;
use structopt::StructOpt;
use stream_cancel::StreamExt;

mod common;

fn main() {
    let args = Args::from_args();
    let config = args.make_config();

    logger::init(log::LevelFilter::Debug);

    let net_control = netzwerk::init(config.clone());

    let mut node = Node::builder()
        .with_config(config)
        .with_net_control(net_control)
        .build();

    //spawn(notification_handler(net_events));

    let msg = Utf8Message::new(&args.msg);

    block_on(node.init());

    //task::spawn(node.spam(msg, 50, 15000));

    block_on(node.shutdown());
}

async fn notification_handler(mut peer_events: EventSub) {
    while let Some(event) = peer_events.next().await {
        logger::info("", &format!("[Node ] Received event {:?}", event));
    }
}

struct Node {
    config: Config,
    net_control: NetControl,
}

impl Node {

    pub async fn init(&mut self) {
        info!("[Node ] Initializing...");

        for peer in self.config.peers().values() {
            self.add_peer(peer.clone()).await;
        }

        info!("[Node ] Initialized");
    }

    pub fn id(&self) -> &String {
        &self.config.id
    }

    pub async fn add_peer(&mut self, peer: Peer) {
        self.net_control.send(AddPeer { peer }).await;
    }

    pub async fn remove_peer(&mut self, peer_id: PeerId) {
        self.net_control.send(RemovePeer { peer_id }).await;
    }

    pub async fn send_msg(&mut self, message: Utf8Message, peer_id: PeerId) {
        self.net_control.send(SendBytes { to: peer_id, bytes: message.as_bytes() }).await;
    }

    pub async fn broadcast_msg(&mut self, message: Utf8Message) {
        self.net_control.send(BroadcastBytes { bytes: message.as_bytes() }).await;
    }

    pub async fn shutdown(mut self) {
        self.block_on_ctrl_c();

        info!("[Node ] Shutting down...");

        self.net_control.send(Shutdown).await;
        self.net_control.finish_tasks().await;
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

    pub async fn spam(&mut self, msg: Utf8Message, num: usize, interval: u64) {
        for _ in 0..num {
            task::sleep(std::time::Duration::from_millis(interval)).await;
            self.broadcast_msg(msg.clone()).await;
        }
    }
}

struct NodeBuilder {
    config: Option<Config>,
    net_control: Option<NetControl>,
}

impl NodeBuilder {
    pub fn new() -> Self {
        Self {
            config: None,
            net_control: None,
        }
    }

    pub fn with_config(mut self, config: Config) -> Self {
        self.config.replace(config);
        self
    }

    pub fn with_net_control(mut self, net_control: NetControl) -> Self {
        self.net_control.replace(net_control);
        self
    }

    pub fn build(self) -> Node {
        Node {
            config: self.config
                .expect("[Node ] No config given to node builder"),
            net_control: self.net_control
                .expect("[Node ] no net-control instance given to node builder"),
        }
    }
}