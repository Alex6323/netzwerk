pub use address::{Address, Url};
pub use connections::{Connection, Connections, Protocol};
pub use commands::{Command, CommandSender, CommandReceiver};
pub use config::{Config, ConfigBuilder};
pub use events::{Event, EventChannel};
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
mod commands;
mod events;
mod message;
mod peers;

use connections as conns;

use log::*;

pub fn init(config: Config) -> EventChannel {

    debug!("'netzwerk' initializing");

    bind_tcp_listener();
    bind_udp_socket();

    let (event_chan, event_rx) = events::channel();

    conns::start_el(event_rx.clone());
    peers::start_el(event_rx.clone());

    event_chan
}

fn bind_tcp_listener() {
    // TODO
}

fn bind_udp_socket() {
    // TODO
}