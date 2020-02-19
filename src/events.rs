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
        peer: Peer
    },

    /// Raised when a peer was removed. No further attempts will be made to connect to that peer.
    PeerRemoved {
        peer_id: PeerId
    },

    /// Raised when a new TCP connection is established.
    TcpConnectionEstablished {
        peer_id: PeerId,
        stream: TcpStream
    },

    /// Raised when a TCP connection has been dropped.
    TcpConnectionDropped {
        peer_id: PeerId,
    },

    /// Raised when a new UDP connection is established.
    UdpConnectionEstablished {
        peer_id: PeerId,
        address: Address,
        socket: UdpSocket,
    },

    /// Raised when a UDP connection has been dropped.
    UdpConnectionDropped {
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
            EventType::PeerAdded { peer } => write!(f, "AddPeer <<{:?}>>", peer.id()),
            EventType::PeerRemoved { peer_id } => write!(f, "DropPeer <<{:?}>>", peer_id),
            EventType::TcpConnectionEstablished { peer_id, .. } => write!(f, "AddTcpConnection <<{:?}>>", peer_id),
            EventType::TcpConnectionDropped { peer_id } => write!(f, "DropTcpConnection <<{:?}>>", peer_id),
            EventType::UdpConnectionEstablished { peer_id, .. } => write!(f, "AddUdpConnection <<{:?}>>", peer_id),
            EventType::UdpConnectionDropped { peer_id } => write!(f, "DropUdpConnection <<{:?}>>", peer_id),
            /*
            EventType::MessageSent { peer_id, bytes } => write!(f, "SendMessage <<{:?}>>, <<{} bytes>>", peer_id, bytes.len()),
            EventType::MessageReceived { bytes } => write!(f, "BroadcastMessage <<{} bytes>>", bytes.len()),
            */
            EventType::Shutdown => write!(f, "Shutdown"),
            _ => Ok(()),
        }
    }
}

pub type EventSource = mpmc::Sender<Event>;
pub type EventSink = mpmc::Receiver<Event>;

pub fn channel() -> (EventSource, EventSink) {
    mpmc::bounded::<Event>(EVENT_CHAN_CAPACITY)
}
