use crate::address::Address;
use crate::peers::{Peer, PeerId};
use crate::tcp::TcpConnection;

use std::fmt;
use std::time::Duration;

use futures::channel::mpsc;

const EVENT_CHAN_CAPACITY: usize = 10000;

/// This Enum holds all events in the networking layer.
#[derive(Clone)]
pub enum Event {

    /// Raised when a peer was added. The layer will try to connect to/reconnect with that peer.
    PeerAdded {
        peer: Peer,
        num_peers: usize,
    },

    /// Raised when a peer was removed. No further attempts will be made to connect to that peer.
    PeerRemoved {
        peer_id: PeerId,
    },

    /// Raised when a peer is connected or reconnected via TCP.
    PeerConnectedViaTCP {
        peer_id: PeerId,
        tcp_conn: TcpConnection,
    },

    /*
    /// Raised when a peer is connected or reconnected via UDP.
    PeerConnectedViaUDP {
        peer_id: PeerId,
        address: Address,
        udp_conn: UdpConnection,
    },
    */

    /// Raised when a peer was disconnected.
    PeerDisconnected {
        peer_id: PeerId,
        reconnect: Option<u64>,
    },

    /// Raised when no packet was received from this peer for some time.
    PeerStalled {
        peer_id: PeerId,
        since: Duration,
    },

    /// Raised when bytes have been sent to a peer.
    BytesSent {
        num_bytes: usize,
        receiver_addr: Address,
    },

    /// Raised when bytes have been sent to all peers.
    BytesBroadcasted {
        num_bytes: usize,
    },

    /// Raised when bytes have been received.
    BytesReceived {
        num_bytes: usize,
        sender_addr: Address,
        bytes: Vec<u8>,
    },

    /// Raised when the system should to try to reconnect with a peer.
    TryReconnect {
        peer_id: PeerId,
    }
}

impl fmt::Debug for Event {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Event::PeerAdded { peer, num_peers } => write!(f, "Peer added: id = {:?}, num = {}", peer.id(), num_peers),
            Event::PeerRemoved { peer_id } => write!(f, "Peer removed: {:?}", peer_id),
            Event::PeerConnectedViaTCP { peer_id, .. } => write!(f, "Peer connected via TCP: {:?}", peer_id),
            //Event::PeerConnectedViaUDP { peer_id, .. } => write!(f, "Peer connected via UDP: {:?}", peer_id),
            Event::PeerDisconnected { peer_id, reconnect } => write!(f, "Peer disconnected: {:?}, reconnect: {}", peer_id, reconnect.is_some()),
            Event::PeerStalled { peer_id, .. } => write!(f, "Peer stalled: {:?}", peer_id),
            Event::BytesSent { num_bytes, receiver_addr } => write!(f, "Message sent to: {:?} ({} bytes)", receiver_addr, num_bytes),
            Event::BytesReceived { num_bytes, sender_addr, .. } => write!(f, "Message received from: {:?} ({} bytes)", sender_addr, num_bytes),
            Event::TryReconnect { peer_id } => write!(f, "TryReconnect: {:?}", peer_id),
            _ => Ok(()),
        }
    }
}

pub type EventPublisher = mpsc::Sender<Event>;
pub type EventSubscriber = mpsc::Receiver<Event>;

pub fn channel() -> (EventPublisher, EventSubscriber) {
    mpsc::channel(EVENT_CHAN_CAPACITY)
}
