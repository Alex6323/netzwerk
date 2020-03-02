#![recursion_limit="2048"]

pub use address::{Address, Protocol, Url};
pub use conns::Connections;
pub use config::{Config, ConfigBuilder};
pub use events::{Event, EventSubscriber};
pub use commands::Command;
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

use async_std::task::{self, spawn, JoinHandle};
use futures::prelude::*;
use log::*;

use std::time::Duration;

use crate::commands::{CommandDispatcher, CommandSender};

pub type Result<T> = std::result::Result<T, errors::Error>;

#[derive(Clone)]
pub struct Network(CommandSender);

impl Network {
    pub(crate) fn new(command_tx: CommandSender) -> Self {
        Self(command_tx)
    }

    pub async fn send(&mut self, command: Command) {
        self.0.send(command).await
            .expect("network: error sending command");
    }
}

pub struct Shutdown {
    tasks: Vec<JoinHandle<()>>,
}

impl Shutdown {
    pub(crate) fn new() -> Self {
        Self {
            tasks: vec![],
        }
    }

    pub(crate) fn add_task(&mut self, task: JoinHandle<()>) {
        self.tasks.push(task);
    }

    pub fn num_tasks(&self) -> usize {
        self.tasks.len()
    }

    pub async fn finish_tasks(&mut self) {
        for task in &mut self.tasks {
            task.await;
        }
    }
}

/// Initializes the networking layer.
pub fn init(config: Config) -> (Network, Shutdown, EventSubscriber) {
    info!("[Net  ] Initializing...");

    let static_peers = config.peers();
    if static_peers.num() == 0 {
        warn!("[Net  ] No static peers from config found.");
    } else {
        info!("[Net  ] Found {} static peer/s in config.", static_peers.num())
    }

    let binding_addr = if let Address::Ip(binding_addr) = config.binding_addr {
        binding_addr
    } else {
        error!("Other address types than IP addresses are currently not supported.");
        panic!("wrong address type");
    };

    let mut command_dp = CommandDispatcher::new();

    // Register Peers actor (pa = peer actor)
    let command_pa_recv = command_dp.register(crate::commands::PEERS);
    let (event_pa_send, event_pa_recv) = events::channel();

    // Register TCP actor (ta = tcp actor)
    let command_ta_recv = command_dp.register(crate::commands::TCP);
    //let (event_ta_send, event_ta_recv) = events::channel();

    // Register UDP actor (ta = tcp actor)
    //let command_ua_recv = command_dp.register(crate::udp::UDP);
    //let (event_ua_send, event_ua_recv) = events::channel();

    // Make a channel for the "user" to issue commands
    let (command_net_send, command_net_recv) = commands::channel();

    // Make a channel for the "user" to receive events
    let (message_net_send, message_net_recv) = events::channel();

    // This actor dispatches/multiplexes incoming commands from the user.
    // It knows which other actor needs to listen to which command, and
    // therefore helps reducing redundant message passing.
    let ca = spawn(commands::actor(command_dp, command_net_recv));

    // This actor is the most crucial of the pack. It manages the peers
    // list, and keeps track of all the network connections.
    let pa = spawn(peers::actor(command_pa_recv, event_pa_recv, event_pa_send.clone(), message_net_send));

    // This actor is responsible for listening on a TCP socket, and send
    // incoming streams to the peers actor.
    let ta = spawn(tcp::acceptor(binding_addr, command_ta_recv, event_pa_send));

    // This actor is responsible for listening on a UDP socket, and send
    // incoming packets to the peers actor.
    //let ua = spawn(udp::actor(binding_addr, commands_ua_recv, event_ua_send));

    wait(500, "[Net  ] Spawning actors");

    let network = Network::new(command_net_send);

    let mut shutdown = Shutdown::new();

    shutdown.add_task(ca);
    shutdown.add_task(pa);
    shutdown.add_task(ta);
    //shutdown.add_task(ua);

    info!("[Net  ] Initialized");

    (network, shutdown, message_net_recv)
}

fn wait(millis: u64, explanation: &str) {
    debug!("{}", explanation);
    task::block_on(task::sleep(Duration::from_millis(millis)));
}