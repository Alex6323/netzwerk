//! This example shows how to create and run a TCP node using `netzwerk`.
//! You might want to run several instances of such a node in separate
//! terminals and connect those instances by specifying commandline arguments.
//!
//! To set up a node id'ed with "A" listening at `localhost:1337` and with
//! two TCP peers located at localhost 1338, localhost 1339 pass this to `cargo`:
//!
//! ```bash
//! cargo r --example node -- --id A --bind localhost:1337 --peers tcp://localhost:1338 tcp://localhost:1339
//! ```

use netzwerk::{
    Config, ConfigBuilder,
    Connections,
    Command, CommandReceiver, CommandSender,
    Event, EventChannel,
    log::*,
    Message,
    Peer, PeerId,
    Protocol,
    tcp::*,
};

use common::*;

use async_std::task;
use structopt::StructOpt;

mod common;

struct Node {
    config: Config,
    event_chan: EventChannel,
}

impl Node {

    pub fn id(&self) -> &String {
        &self.config.id
    }

    pub fn add_peer(&self, peer: Peer) {
        self.event_chan.send(Event::NewPeer { peer }).expect("error sending NewPeer event");
    }

    pub fn drop_peer(&self, peer_id: PeerId) {
        self.event_chan.send(Event::DropPeer { peer_id }).expect("error sending DropPeer event");
    }

    pub fn shutdown(&self) {
        self.event_chan.send(Event::Shutdown).expect("error sending Shutdown event");
    }

    pub fn send_msg(&self, message: impl Message, peer_id: PeerId) {
        self.event_chan.send(Event::SendMessage { bytes: message.into_bytes(), peer_id })
            .expect("error sending SendMessage event");
    }

    pub fn broadcast_msg(&self, message: impl Message) {
        self.event_chan.send(Event::BroadcastMessage { bytes: message.into_bytes() })
            .expect("error sending BroadcastMessage event");
    }

    pub fn builder() -> NodeBuilder {
        NodeBuilder::new()
    }
}

struct NodeBuilder {
    config: Option<Config>,
    event_chan: Option<EventChannel>,
}

impl NodeBuilder {
    pub fn new() -> Self {
        Self {
            config: None,
            event_chan: None,
        }
    }

    pub fn with_config(mut self, config: Config) -> Self {
        self.config.replace(config);
        self
    }

    pub fn with_event_chan(mut self, event_chan: EventChannel) -> Self {
        self.event_chan.replace(event_chan);
        self
    }

    pub fn build(self) -> Node {
        Node {
            config: self.config.expect("NodeBuilder: no config specified"),
            event_chan: self.event_chan.expect("NodeBuilder: no event channel specified"),
        }
    }
}

fn main() {
    let config: Config = args::Args::from_args().into();

    logger::init(log::LevelFilter::Debug);
    screen::init();

    let event_chan = netzwerk::init(config.clone());

    let node = Node::builder()
        .with_config(config)
        .with_event_chan(event_chan)
        .build();

    logger::info(&format!("Created node <<{}>>", node.id()));

    let msg = Utf8Message::new("hello netzwerk");
    node.broadcast_msg(msg);

    std::thread::sleep(std::time::Duration::from_millis(3000));
    screen::exit();
}