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
    Command,
    CommandDispatcher as CommandDp,
    Event,
    EventSubscriber as EventSub,
    Handles,
    log::*,
    Peer, PeerId,
    Protocol,
    tcp::*,
};

use common::*;

use async_std::task;
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

    let (command_dp, event_rx, handles) = netzwerk::init(config.clone());

    task::spawn(notification_handler(event_rx));

    let mut node = Node::builder()
        .with_config(config)
        .with_command_dispatcher(command_dp)
        .with_handles(handles)
        .build();

    task::block_on(node.init());

    logger::info("", &format!("Created node [{}]", node.id()));

    node.wait_for_kill_signal();

    /*
    // TEMP: broadcast a message each second
    task::block_on(async {
        for _ in 0..5 {
            task::sleep(std::time::Duration::from_millis(1000)).await;
            node.broadcast_msg(Utf8Message::new(&args.msg)).await;
        }
    });
    */

    task::block_on(node.shutdown());
}

async fn notification_handler(mut event_sub: EventSub) {
    while let Some(event) = event_sub.next().await {
        logger::info("", &format!("Received event {:?}", event));
    }
}

struct Node {
    config: Config,
    command_dp: CommandDp,
    handles: Handles,
}

impl Node {

    pub async fn init(&mut self) {
        for peer in self.config.peers().values() {
            self.add_peer(peer.clone()).await;
        }
    }

    // TODO: How can we not use Tokio, and just async_std?
    pub fn wait_for_kill_signal(&self) {
        let mut rt = tokio::runtime::Runtime::new().expect("error");

        rt.block_on(tokio::signal::ctrl_c()).expect("error");
    }

    pub fn id(&self) -> &String {
        &self.config.id
    }

    pub async fn add_peer(&mut self, peer: Peer) {
        self.command_dp.dispatch(Command::AddPeer { peer }).await;
    }

    pub async fn remove_peer(&mut self, peer_id: PeerId) {
        self.command_dp.dispatch(Command::RemovePeer { peer_id }).await;
    }

    pub fn wait(&self, handles: Vec<task::JoinHandle<()>>) {
        debug!("Waiting for async tasks to finish...");
        /*
        task::block_on(async {
            task::sleep(Duration::from_millis(5000)).await;
        });
        */
    }

    pub async fn shutdown(mut self) {
        info!("Shutting down...");

        self.command_dp.dispatch(Command::Shutdown).await;

        for handle in self.handles.task_handles() {
            handle.await;
        }
    }

    pub async fn send_msg(&mut self, message: Utf8Message, peer_id: PeerId) {
        self.command_dp.dispatch(Command::SendBytes { receiver: peer_id, bytes: message.as_bytes() }).await;
    }

    pub async fn broadcast_msg(&mut self, message: Utf8Message) {
        self.command_dp.dispatch(Command::BroadcastBytes { bytes: message.as_bytes() }).await;
    }

    pub fn builder() -> NodeBuilder {
        NodeBuilder::new()
    }
}

struct NodeBuilder {
    config: Option<Config>,
    command_dp: Option<CommandDp>,
    handles: Option<Handles>,
}

impl NodeBuilder {
    pub fn new() -> Self {
        Self {
            config: None,
            command_dp: None,
            handles: None,
        }
    }

    pub fn with_config(mut self, config: Config) -> Self {
        self.config.replace(config);
        self
    }

    pub fn with_command_dispatcher(mut self, command_dp: CommandDp) -> Self {
        self.command_dp.replace(command_dp);
        self
    }

    pub fn with_handles(mut self, handles: Handles) -> Self {
        self.handles.replace(handles);
        self
    }

    pub fn build(self) -> Node {
        Node {
            config: self.config.expect("NodeBuilder: no config specified"),
            command_dp: self.command_dp.expect("NodeBuilder: no command dispatcher specified"),
            handles: self.handles.expect("NodeBuilder: no handles specified"),
        }
    }
}