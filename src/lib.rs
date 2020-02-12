pub use address::{Address, Url};
pub use connections::{Connection, Connections, Protocol};
pub use config::{Config, ConfigBuilder};
pub use events::{Event, EventProducer};
pub use message::Message;
pub use peers::{Peer, PeerId, Peers};
pub use log;

pub mod error;
pub mod util;
pub mod result;
pub mod tcp;
pub mod udp;

mod address;
mod config;
mod connections;
mod events;
mod message;
mod peers;

use connections as conns;

use async_std::prelude::*;
use async_std::{task, task::JoinHandle};
use log::*;
use std::time::Duration;

/// Initializes the networking layer using a config, and returns a `CommandSender`
/// for the user to interact with the system.
pub fn init(config: Config) -> (EventProducer, Vec<JoinHandle<()>>) {
    debug!("Initializing network layer");

    if config.peers().num() == 0 {
        warn!("No static peers from config.");
    };

    let (event_prod, event_cons) = events::channel();

    let socket_addr = if let Address::Ip(socket_addr) = config.binding_addr {
        socket_addr
    } else {
        error!("Other address types than IP addresses are currently not supported.");
        panic!("wrong address type");
    };

    let mut handles = vec![];

    handles.push(task::spawn(conns::start_loop(event_cons.clone())));
    handles.push(task::spawn(peers::start_loop(event_cons.clone())));
    // TEMP?
    task::block_on(async {
        task::sleep(Duration::from_millis(500)).await;
    });
    handles.push(task::spawn(tcp::init(socket_addr, event_prod.clone())));
    handles.push(task::spawn(udp::init(socket_addr, event_prod.clone())));
    // TEMP?
    task::block_on(async {
        task::sleep(Duration::from_millis(500)).await;
    });

    (event_prod, handles)
}

///
pub fn exit() {
    // TODO: make sure to drop the remaining `EventProducer` channels for a graceful shutdown.
}