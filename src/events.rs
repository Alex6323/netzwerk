use crate::address::Address;
use crate::peers::{Peer, PeerId};

use std::{fmt, ops};

use async_std::net::{TcpStream, UdpSocket};
use async_std::sync::Arc;
use crossbeam_channel as mpmc;

const EVENT_CHAN_CAPACITY: usize = 10000;

#[derive(Clone, Debug)]
pub struct Event(Arc<EventType>);

/// This Enum holds all events in the networking layer.
pub enum EventType {

    /// Raised when a peer was added. The layer will try to connect/reconnect to that peer.
    PeerAdded {
        peer: Peer,
    },

    /// Raised when a peer was removed. No further attempts will be made to connect to that peer.
    PeerRemoved {
        peer_id: PeerId,
    },

    /// Raised when a tcp peer is connected (again).
    TcpPeerConnected {
        peer_id: PeerId,
        stream: TcpStream,
    },

    /// Raised when a tcp peer is disconnected.
    TcpPeerDisconnected {
        peer_id: PeerId
    },

    /// Raised when a new UDP connection is established.
    UdpPeerConnected {
        peer_id: PeerId,
        address: Address,
        socket: UdpSocket,
    },

    /// Raised when a UDP connection has been dropped.
    UdpPeerDisconnected {
        peer_id: PeerId,
    },

    /// Raised when no packet was received from this TCP peer for some time.
    TcpPeerUnresponsive {
        peer_id: PeerId,
    },

    /// Raised when no packet was received from this UDP peer for some time.
    UdpPeerUnresponsive {
        peer_id: PeerId,
    },

    /// Raised when a message has been sent.
    MessageSent {
        num_bytes: usize,
        receiver_addr: Address,
    },

    /// Raised when a messge has been sent to all peers.
    MessageBroadcasted {
        num_bytes: usize,
    },

    /// Raised when a message has been received.
    MessageReceived {
        num_bytes: usize,
        sender_addr: Address,
        bytes: Vec<u8>,
    },

    /// Raised when the system has been commanded to shutdown.
    Shutdown,
}

impl ops::Deref for Event {
    type Target = EventType;

    fn deref(&self) -> &Self::Target {
        &*self.0
    }
}

impl From<EventType> for Event {
    fn from(t: EventType) -> Self {
        Self(Arc::new(t))
    }
}

impl fmt::Debug for EventType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EventType::PeerAdded { peer } => write!(f, "Peer added: {:?}", peer.id()),
            EventType::PeerRemoved { peer_id } => write!(f, "Peer removed: {:?}", peer_id),
            EventType::TcpPeerConnected { peer_id, .. } => write!(f, "TCP peer connected: {:?}", peer_id),
            EventType::TcpPeerDisconnected { peer_id } => write!(f, "TCP peer disconnected: {:?}", peer_id),
            EventType::UdpPeerConnected { peer_id, .. } => write!(f, "UDP peer connected: {:?}", peer_id),
            EventType::UdpPeerDisconnected { peer_id } => write!(f, "UDP peer disconnected: {:?}", peer_id),
            EventType::TcpPeerUnresponsive { peer_id, .. } => write!(f, "TCP peer unresponsive: {:?}", peer_id),
            EventType::UdpPeerUnresponsive { peer_id } => write!(f, "UDP peer unresponsive: {:?}", peer_id),
            EventType::MessageSent { num_bytes, receiver_addr } => write!(f, "Message sent to: {:?} ({} bytes)", receiver_addr, num_bytes),
            EventType::MessageReceived { num_bytes, sender_addr, .. } => write!(f, "Message received from: {:?} ({} bytes)", sender_addr, num_bytes),
            EventType::Shutdown => write!(f, "Shutdown signal received"),
            _ => Ok(()),
        }
    }
}

pub type EventSource = mpmc::Sender<Event>;
pub type EventSink = mpmc::Receiver<Event>;

pub fn channel() -> (EventSource, EventSink) {
    mpmc::bounded::<Event>(EVENT_CHAN_CAPACITY)
}
