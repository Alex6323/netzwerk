pub use address::{Address, Url};
pub use broker::{TcpBroker, UdpBroker};
pub use connection::{Connection, Connections, Tcp, Udp, Protocol};
pub use message::Message;
pub use peer::{Peer, PeerId, Peers};

pub mod error;
pub mod util;
pub mod result;

mod address;
mod broker;
mod config;
mod connection;
mod message;
mod peer;
mod signal;


/// Initializes the `netzwerk` API.
pub fn init() {
    // TODO
    unimplemented!("netzwerk::init");
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