pub use address::{Address, Url};
pub use broker::{TcpBroker, UdpBroker};
pub use connection::{Connection, Connections, Tcp, Udp, Protocol};
pub use message::Message;
pub use peer::{Peer, PeerId, Peers};

pub mod util;

mod address;
mod broker;
mod config;
mod connection;
mod error;
mod message;
mod peer;
mod result;
mod signal;


/// Initializes the `netzwerk` API.
pub fn init() {
    // TODO
    unimplemented!("netzwerk::init");
}

/// Tries to send a message to a peer.
pub fn try_send_to_peer(peer: Peer, message: Message) {
    // TODO
    unimplemented!("netzwerk::try_send_to_peer")
}

/// Tries to recv a message from a peer.
pub fn try_recv_from_peer(peer: Peer) -> result::Result<Message> {
    // TODO
    unimplemented!("netzwer::try_recv_from_peer")
}