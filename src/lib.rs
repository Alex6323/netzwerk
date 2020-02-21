pub use address::{Address, Protocol, Url};
pub use conns::Connections;
pub use config::{Config, ConfigBuilder};
pub use events::{Event, EventSubscriber};
pub use commands::{Command,Controller};
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
//mod message;
mod peers;

use async_std::task;
use log::*;

use std::time::Duration;

pub type Result<T> = std::result::Result<T, errors::Error>;

/// Initializes the networking layer using a config, and returns a `Controller`
/// for the user to interact with the system, and an `EventSink` to receive
/// all events.
pub fn init(config: Config) -> (Controller, EventSubscriber) {

    debug!("[Net  ] Initializing network layer");

    let static_peers = config.peers();
    if static_peers.num() == 0 {
        warn!("[Net  ] No static peers from config found.");
    } else {
        info!("[Net  ] Found {} static peer(s) in config.", static_peers.num())
    }

    let (mut controller, command_receiver) = commands::channel();
    let (event_source, event_sink) = events::channel();

    // TODO: ActorLink is Clone & Copy
    // let link = ActorLink::new(command_receiver, event_source, event_sink);

    let binding_addr = if let Address::Ip(binding_addr) = config.binding_addr {
        binding_addr
    } else {
        error!("Other address types than IP addresses are currently not supported.");
        panic!("wrong address type");
    };

    // TODO: make those channel halfs `Copy` so the code becomes more readable!


    let actor1 = task::spawn(conns::actor::run(command_receiver.clone(), event_source.clone(), event_sink.clone()));
    let actor2 = task::spawn(peers::actor::run(command_receiver.clone(), event_source.clone(), event_sink.clone()));
    wait(500, "[Net  ] Waiting for actors");

    let actor3 = task::spawn(tcp::actor::run(binding_addr, event_source.clone()));
    wait(500, "[Net  ] Waiting for actors");

    controller.add_task(actor1);
    controller.add_task(actor2);
    controller.add_task(actor3);


    (controller, event_sink)
}

fn wait(millis: u64, explanation: &str) {
    info!("{}", explanation);
    task::block_on(task::sleep(Duration::from_millis(millis)));
}