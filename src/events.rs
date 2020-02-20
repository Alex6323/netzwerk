use crate::address::Address;
use crate::peers::{Peer, PeerId};

use std::{fmt, ops};
use std::time::Duration;

use async_std::net::{TcpStream, UdpSocket};
use async_std::sync::Arc;
use crossbeam_channel as mpmc;

const EVENT_CHAN_CAPACITY: usize = 10000;

//#[derive(Clone, Debug)]
//pub struct Event(Arc<EventType>);

/// This Enum holds all events in the networking layer.
#[derive(Clone)]
pub enum Event {

    /// Raised when a peer was added. The layer will try to connect to/reconnect with that peer.
    PeerAdded {
        peer: Peer,
    },

    /// Raised when a peer was removed. No further attempts will be made to connect to that peer.
    PeerRemoved {
        peer_id: PeerId,
    },

    /// Raised when a peer is connected or reconnected via TCP.
    PeerConnectedViaTCP {
        peer_id: PeerId,
        stream: Arc<TcpStream>,
    },

    /// Raised when a peer is connected or reconnected via UDP.
    PeerConnectedViaUDP {
        peer_id: PeerId,
        address: Address,
        socket: Arc<UdpSocket>,
    },
    /// Raised when a peer was disconnected.
    PeerDisconnected {
        peer_id: PeerId
    },

    /// Raised when no packet was received from this peer for some time.
    PeerIdle {
        peer_id: PeerId,
        duration: Duration,
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

/*
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
*/

impl fmt::Debug for Event {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Event::PeerAdded { peer } => write!(f, "Peer added: {:?}", peer.id()),
            Event::PeerRemoved { peer_id } => write!(f, "Peer removed: {:?}", peer_id),
            Event::PeerConnectedViaTCP { peer_id, .. } => write!(f, "Peer connected via TCP: {:?}", peer_id),
            Event::PeerConnectedViaUDP { peer_id, .. } => write!(f, "Peer connected via UDP: {:?}", peer_id),
            Event::PeerDisconnected { peer_id } => write!(f, "Peer disconnected: {:?}", peer_id),
            Event::PeerIdle { peer_id, .. } => write!(f, "Peer has been idle: {:?}", peer_id),
            Event::MessageSent { num_bytes, receiver_addr } => write!(f, "Message sent to: {:?} ({} bytes)", receiver_addr, num_bytes),
            Event::MessageReceived { num_bytes, sender_addr, .. } => write!(f, "Message received from: {:?} ({} bytes)", sender_addr, num_bytes),
            _ => Ok(()),
        }
    }
}

pub type EventSource = mpmc::Sender<Event>;
pub type EventSink = mpmc::Receiver<Event>;

pub fn channel() -> (EventSource, EventSink) {
    mpmc::bounded::<Event>(EVENT_CHAN_CAPACITY)
}
