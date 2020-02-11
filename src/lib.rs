pub use address::{Address, Url};
pub use connection::{Connection, Connections, Tcp, Udp, Protocol};
pub use commands::{Command, CommandSender, CommandReceiver};
pub use config::Config;
pub use events::{Event, EventTx};
pub use message::Message;
pub use peers::{Peer, PeerId, Peers};
pub use log;

pub mod error;
pub mod util;
pub mod result;

mod address;
mod broker;
mod config;
mod connection;
mod commands;
mod events;
mod message;
mod peers;

/// Initializes the `netzwerk` API.
pub fn init(config: config::Config) -> EventTx {
    // TODO: remove this!
    use broker as conns;

    let (event_tx, event_rx) = events::channel();

    conns::start_el(event_rx.clone());
    peers::start_el(event_rx.clone());

    event_tx
}

/// Tries to send a message to a peer.
pub fn try_send_to_peer(peer: Peer, message: impl Message) {
    // TODO
    unimplemented!("netzwerk::try_send_to_peer")
}

/// Tries to recv a message from a peer.
pub fn try_recv_from_peer<M: Message>(peer: Peer) -> result::Result<M> {
    // TODO
    unimplemented!("netzwer::try_recv_from_peer")
}