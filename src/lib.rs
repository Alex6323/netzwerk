pub use address::{Address, Url};
pub use conns::{Connection, Connections, Protocol};
pub use config::{Config, ConfigBuilder};
pub use events::{Event, EventSink};
pub use commands::{Command, CommandType, Controller};
pub use message::Message;
pub use peers::{Peer, PeerId, Peers};
pub use log;

pub mod errors;
pub mod utils;
pub mod result;
pub mod tcp;
//pub mod udp;

mod address;
mod config;
mod conns;
mod commands;
mod events;
mod message;
mod peers;

use async_std::task;
use log::*;
use std::time::Duration;

/// Initializes the networking layer using a config, and returns a `CommandSender`
/// for the user to interact with the system.
pub fn init(config: Config) -> (Controller, EventSink) {

    debug!("Initializing network layer");

    let num_static_peers = config.peers().num();
    if num_static_peers == 0 {
        warn!("No static peers from config found.");
    } else {
        info!("Found {} static peer(s) in config.", num_static_peers)
    }

    let (mut controller, command_receiver) = commands::channel();
    let (event_source, event_sink) = events::channel();

    let socket_addr = if let Address::Ip(socket_addr) = config.binding_addr {
        socket_addr
    } else {
        error!("Other address types than IP addresses are currently not supported.");
        panic!("wrong address type");
    };

    controller.add_task(task::spawn(conns::listen(command_receiver.clone(), event_source.clone())));
    controller.add_task(task::spawn(peers::listen(command_receiver.clone(), event_source.clone())));
    wait(500);

    controller.add_task(task::spawn(tcp::listen(socket_addr, event_source.clone())));
    wait(500);

    (controller, event_sink)
}

fn wait(millis: u64) {
    task::block_on(task::sleep(Duration::from_millis(millis)));
}