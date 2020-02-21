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
    Config, ConfigBuilder,
    Connections,
    Command, Controller,
    Event, EventSink,
    log::*,
    Message,
    Peer, PeerId,
    Protocol,
    tcp::*,
};

use common::*;

use async_std::task;
use futures::stream::Stream;
use futures::*;
use futures::future::Future;
use structopt::StructOpt;
use stream_cancel::StreamExt;

mod common;

fn main() {
    let args = Args::from_args();
    let config = args.make_config();

    logger::init(log::LevelFilter::Debug);

    let (controller, notifier) = netzwerk::init(config.clone());

    task::spawn(notification_handler(notifier));

    let node = Node::builder()
        .with_config(config)
        .with_controller(controller)
        .build();

    node.init();

    logger::info("example", &format!("Created node [{}]", node.id()));

    node.wait_for_kill_signal();

    /*
    // TEMP: broadcast a message each second
    for _ in 0..5 {
        std::thread::sleep(std::time::Duration::from_millis(1000));
        node.broadcast_msg(Utf8Message::new(&args.msg));
    }

    */
    //node.shutdown();
}

async fn notification_handler(event_sink: EventSink) {
    while let Ok(event) = event_sink.recv() {
        logger::info("example", &format!("Received event {:?}", event));
    }
}

struct Node {
    config: Config,
    controller: Controller,
}

impl Node {

    pub fn init(&self) {
        for peer in self.config.peers().values() {
            self.add_peer(peer.clone());
        }
    }

    pub fn wait_for_kill_signal(&self) {
        let mut rt = tokio::runtime::Runtime::new().expect("error");

        rt.block_on(tokio::signal::ctrl_c()).expect("error");
    }

    pub fn id(&self) -> &String {
        &self.config.id
    }

    pub fn add_peer(&self, peer: Peer) {
        self.controller.send(Command::AddPeer { peer });
    }

    pub fn remove_peer(&self, peer_id: PeerId) {
        self.controller.send(Command::RemovePeer { peer_id });
    }

    pub fn wait(&self, handles: Vec<task::JoinHandle<()>>) {
        debug!("Waiting for async tasks to finish...");
        /*
        task::block_on(async {
            task::sleep(Duration::from_millis(5000)).await;
        });
        */
    }

    pub fn shutdown(mut self) {
        info!("Shutting down...");

        self.controller.send(Command::Shutdown);

        task::block_on(async {
            for handle in self.controller.task_handles() {
                handle.await;
            }
        })
    }

    pub fn send_msg(&self, message: impl Message, peer_id: PeerId) {
        self.controller.send(Command::SendBytes { receiver: peer_id, bytes: message.as_bytes() });
    }

    pub fn broadcast_msg(&self, message: impl Message) {
        self.controller.send(Command::BroadcastBytes { bytes: message.as_bytes() });
    }

    pub fn builder() -> NodeBuilder {
        NodeBuilder::new()
    }
}

struct NodeBuilder {
    config: Option<Config>,
    controller: Option<Controller>,
}

impl NodeBuilder {
    pub fn new() -> Self {
        Self {
            config: None,
            controller: None,
        }
    }

    pub fn with_config(mut self, config: Config) -> Self {
        self.config.replace(config);
        self
    }

    pub fn with_controller(mut self, controller: Controller) -> Self {
        self.controller.replace(controller);
        self
    }

    pub fn build(self) -> Node {
        Node {
            config: self.config.expect("NodeBuilder: no config specified"),
            controller: self.controller.expect("NodeBuilder: no controller specified"),
        }
    }
}