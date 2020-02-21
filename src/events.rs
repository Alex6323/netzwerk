use crate::address::Address;
use crate::peers::{Peer, PeerId};
use crate::tcp::TcpConnection;

use std::fmt;
use std::time::Duration;

use async_std::net::UdpSocket;
use async_std::sync::Arc;
use crossbeam_channel as mpmc;

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

    /// Raised when a peer is connected or reconnected via UDP.
    PeerConnectedViaUDP {
        peer_id: PeerId,
        address: Address,
        socket: Arc<UdpSocket>,
    },

    /// Raised when a peer was disconnected.
    PeerDisconnected {
        peer_id: PeerId,
        reconnect: Option<u64>,
    },

    /// Raised when no packet was received from this peer for some time.
    PeerStale {
        peer_id: PeerId,
        duration: Duration,
    },

    /// Raised when an attempt should be made to reconnect with that peer.
    PeerReconnect {
        peer_id: PeerId,
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
}

impl fmt::Debug for Event {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Event::PeerAdded { peer, num_peers } => write!(f, "Peer added: id = {:?}, num = {}", peer.id(), num_peers),
            Event::PeerRemoved { peer_id } => write!(f, "Peer removed: {:?}", peer_id),
            Event::PeerConnectedViaTCP { peer_id, .. } => write!(f, "Peer connected via TCP: {:?}", peer_id),
            Event::PeerConnectedViaUDP { peer_id, .. } => write!(f, "Peer connected via UDP: {:?}", peer_id),
            Event::PeerDisconnected { peer_id, reconnect } => write!(f, "Peer disconnected: {:?}, reconnect: {}", peer_id, reconnect.is_some()),
            Event::PeerStale { peer_id, .. } => write!(f, "Peer is stale: {:?}", peer_id),
            Event::BytesSent { num_bytes, receiver_addr } => write!(f, "Message sent to: {:?} ({} bytes)", receiver_addr, num_bytes),
            Event::BytesReceived { num_bytes, sender_addr, .. } => write!(f, "Message received from: {:?} ({} bytes)", sender_addr, num_bytes),
            _ => Ok(()),
        }
    }
}

pub type EventPublisher = mpmc::Sender<Event>;
pub type EventSubscriber = mpmc::Receiver<Event>;

pub fn channel() -> (EventPublisher, EventSubscriber) {
    mpmc::bounded::<Event>(EVENT_CHAN_CAPACITY)
}
