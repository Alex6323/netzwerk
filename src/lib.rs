#![recursion_limit="512"]

pub use address::{Address, Protocol, Url};
pub use conns::Connections;
pub use config::{Config, ConfigBuilder};
pub use events::{Event, EventSubscriber};
pub use commands::{Command, CommandDispatcher};
pub use peers::{Peer, PeerId, Peers};
pub use log;

pub mod errors;
pub mod utils;
pub mod tcp;
//pub mod udp;

mod address;
mod config;
mod conns;
mod commands;
mod events;
mod peers;

use async_std::future::Future;
use async_std::task::{self, JoinHandle};
use log::*;

use std::time::Duration;

pub type Result<T> = std::result::Result<T, errors::Error>;

pub struct Handles {
    task_handles: Vec<JoinHandle<()>>,
}

impl Handles {
    pub fn new() -> Self {
        Self {
            task_handles: vec![],
        }
    }

    pub fn task_handles(&mut self) -> &mut Vec<JoinHandle<()>> {
        &mut self.task_handles
    }

    pub fn add_task(&mut self, handle: JoinHandle<()>) {
        self.task_handles.push(handle);
    }

    pub fn num(&self) -> usize {
        self.task_handles.len()
    }
}

/// Initializes the networking layer using a config, and returns a `CommandDispatcher`
/// for the user to interact with the system, an `EventSubscriber` to receive
/// all events, and `Handles` to allow to block on asynchronous tasks.
pub fn init(config: Config) -> (CommandDispatcher, EventSubscriber, Handles) {
    debug!("[Net  ] Initializing network layer");

    let static_peers = config.peers();
    if static_peers.num() == 0 {
        warn!("[Net  ] No static peers from config found.");
    } else {
        info!("[Net  ] Found {} static peer(s) in config.", static_peers.num())
    }

    let mut command_dp = CommandDispatcher::new();
    let mut handles = Handles::new();

    //
    let (conn_tx, conn_rx) = events::channel();
    let (conns_tx, conns_rx) = events::channel();
    let (peers_tx, peers_rx) = events::channel();

    // TODO: ActorLink is Clone & Copy
    // let link = ActorLink::new(command_receiver, event_source, event_sink);

    let binding_addr = if let Address::Ip(binding_addr) = config.binding_addr {
        binding_addr
    } else {
        error!("Other address types than IP addresses are currently not supported.");
        panic!("wrong address type");
    };

    let actor1 = task::spawn(conns::actor::run(command_dp.recv(), conn_rx, conns_tx));
    let actor2 = task::spawn(peers::actor::run(command_dp.recv(), peers_rx, peers_tx));
    wait(500, "[Net  ] Waiting for actors");

    // The TCP actor will listen to incoming TCP streams and send them to the Connections actor.
    let actor3 = spawn_and_log_error(tcp::actor::run(binding_addr, command_dp.recv(), conn_tx.clone()));

    // The UDP actor will listen to incoming UDP packets and send them to the corresponding Peer actor.
    //let actor4 = task::spawn(udp::actor::run(binding_addr, command_dp.recv(), conn_tx.clone()));
    wait(500, "[Net  ] Waiting for actors");

    handles.add_task(actor1);
    handles.add_task(actor2);
    handles.add_task(actor3);

    (command_dp, conns_rx, handles)
}

fn wait(millis: u64, explanation: &str) {
    info!("{}", explanation);
    task::block_on(task::sleep(Duration::from_millis(millis)));
}

fn spawn_and_log_error<F>(fut: F) -> task::JoinHandle<()>
where
    F: Future<Output = ()> + Send + 'static,
{
    task::spawn(async move {
        fut.await
        //if let Err(e) = fut.await {
            //error!("{}", e)
        //}
    })
}
