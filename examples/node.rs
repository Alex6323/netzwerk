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
    Event, EventProducer,
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
    event_prod: EventProducer,
}

impl Node {

    pub fn id(&self) -> &String {
        &self.config.id
    }

    pub fn add_peer(&self, peer: Peer) {
        self.event_prod.send(Event::AddPeer { peer }).expect("error sending NewPeer event");
    }

    pub fn drop_peer(&self, peer_id: PeerId) {
        self.event_prod.send(Event::DropPeer { peer_id }).expect("error sending DropPeer event");
    }

    pub fn wait(&self, handles: Vec<task::JoinHandle<()>>) {
        debug!("Waiting for async tasks to finish...");
        /*
        task::block_on(async {
            task::sleep(Duration::from_millis(5000)).await;
        });
        */
    }

    pub fn shutdown(self, handles: Vec<task::JoinHandle<()>>) {
        debug!("Shutting down...");

        self.event_prod.send(Event::Shutdown).expect("error sending Shutdown event");

        drop(self.event_prod);

        task::block_on(async {
            for handle in handles {
                handle.await;
            }
        })
    }

    pub fn send_msg(&self, message: impl Message, peer_id: PeerId) {
        self.event_prod.send(Event::SendMessage { bytes: message.into_bytes(), peer_id })
            .expect("error sending SendMessage event");
    }

    pub fn broadcast_msg(&self, message: impl Message) {
        self.event_prod.send(Event::BroadcastMessage { bytes: message.into_bytes() })
            .expect("error sending BroadcastMessage event");
    }

    pub fn builder() -> NodeBuilder {
        NodeBuilder::new()
    }
}

struct NodeBuilder {
    config: Option<Config>,
    event_prod: Option<EventProducer>,
}

impl NodeBuilder {
    pub fn new() -> Self {
        Self {
            config: None,
            event_prod: None,
        }
    }

    pub fn with_config(mut self, config: Config) -> Self {
        self.config.replace(config);
        self
    }

    pub fn with_event_prod(mut self, event_chan: EventProducer) -> Self {
        self.event_prod.replace(event_chan);
        self
    }

    pub fn build(self) -> Node {
        Node {
            config: self.config.expect("NodeBuilder: no config specified"),
            event_prod: self.event_prod.expect("NodeBuilder: no event channel specified"),
        }
    }
}

fn main() {
    let config: Config = args::Args::from_args().into();

    logger::init(log::LevelFilter::Debug);
    //screen::init();

    let (event_prod, handles) = netzwerk::init(config.clone());

    let node = Node::builder()
        .with_config(config)
        .with_event_prod(event_prod)
        .build();

    logger::info(&format!("Created node <<{}>>", node.id()));

    let msg = Utf8Message::new("hello netzwerk");
    node.broadcast_msg(msg);

    std::thread::sleep(std::time::Duration::from_millis(5000));

    node.shutdown(handles);

    netzwerk::exit();

    //screen::exit();
}