#![recursion_limit="1024"]

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

use std::sync::Arc;
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
pub fn init(config: Config) -> (Network, Shutdown) {
    info!("[Net  ] Initializing network layer");

    let static_peers = config.peers();
    if static_peers.num() == 0 {
        warn!("[Net  ] No static peers from config found.");
    } else {
        info!("[Net  ] Found {} static peer(s) in config.", static_peers.num())
    }

    let binding_addr = if let Address::Ip(binding_addr) = config.binding_addr {
        binding_addr
    } else {
        error!("Other address types than IP addresses are currently not supported.");
        panic!("wrong address type");
    };

    let mut command_dp = CommandDispatcher::new();

    // Peers actor
    let p_commands_recv = command_dp.register("peers");
    let (p_events_send, p_events_recv) = events::channel();

    // TCP actor
    let t_commands_recv = command_dp.register("tcp");
    let (t_events_send, t_events_recv) = events::channel();

    //let u_commands = command_dp.register("udp");
    //let (pub_u, sub_u) = events::channel();

    //let c_commands = command_dp.register("conns");
    //let (pub_c, sub_c) = events::channel();

    //let (dummy_pub, dummy_sub) = events::channel();

    let (net_control_send, net_control_recv) = commands::channel();

    // Start actors
    let m_actor = spawn(commands::actor(command_dp, net_control_recv));
    let p_actor = spawn(peers::actor(p_commands_recv, p_events_recv, p_events_send));
    let t_actor = spawn(tcp::actor(binding_addr, t_commands_recv, t_events_send));
    //let u_actor = spawn(udp::actor(binding_addr, u_commands, pub_u));

    wait(500, "[Net  ] Spawning actors");

    let network = Network::new(net_control_send);

    let mut shutdown = Shutdown::new();

    shutdown.add_task(m_actor);
    shutdown.add_task(p_actor);
    shutdown.add_task(t_actor);
    //shutdown.add_task(u_actor);

    (network, shutdown)
}

fn wait(millis: u64, explanation: &str) {
    debug!("{}", explanation);
    task::block_on(task::sleep(Duration::from_millis(millis)));
}