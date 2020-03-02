use crate::address::{Address, Protocol};
use crate::conns::ByteSender;
use crate::peers::PeerId;

use std::fmt;

use futures::channel::mpsc;

const EVENT_CHAN_CAPACITY: usize = 10000;

/// This Enum holds all events in the networking layer.
#[derive(Clone)]
pub enum Event {

    /// Raised when a peer was added. The layer will try to connect to/reconnect with that peer.
    PeerAdded {
        peer_id: PeerId,
        num_peers: usize,
    },

    /// Raised when a peer was removed. No further attempts will be made to connect to that peer.
    PeerRemoved {
        peer_id: PeerId,
        num_peers: usize,
    },

    /// Raised when a peer was accepted.
    PeerAccepted {
        peer_id: PeerId,
        protocol: Protocol,
        sender: ByteSender,
    },

    /// Raised when a peer is connected.
    PeerConnected {
        peer_id: PeerId,
        num_conns: usize,
    },

    /// Raised when a peer was disconnected.
    PeerDisconnected {
        peer_id: PeerId,
        num_conns: usize,
    },

    /// Raised when no packet was received from this peer for some time.
    PeerStalled {
        peer_id: PeerId,
    },

    /// Raised when bytes have been sent to a peer.
    BytesSent {
        to_peer: PeerId,
        num_bytes: usize,
    },

    /// Raised when bytes have been sent to all peers.
    BytesBroadcasted {
        num_bytes: usize,
        num_conns: usize,
    },

    /// Raised when bytes have been received.
    BytesReceived {
        from_peer: PeerId,
        with_addr: Address,
        num_bytes: usize,
        buffer: Vec<u8>,
    },

    /// Raised by stream-based protocols when a stream ends.
    ///
    /// NOTE: This usually causes a `PeerDisconnected` event as a consequence if this is
    /// the only connection with that peer.
    StreamStopped {
        from_peer: PeerId,
    },

    /// Raised when the system should to try to (re)connect to a peer after a certain delay.
    TryConnect {
        peer_id: PeerId,
    }
}

impl fmt::Debug for Event {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Event::PeerAdded { peer_id, num_peers } =>
                write!(f, "Event::PeerAdded {{ peer_id = {:?}, num_peers = {} }}", peer_id, num_peers),

            Event::PeerRemoved { peer_id, num_peers } =>
                write!(f, "Event::PeerRemoved  {{ peer_id = {:?}, num_peers = {} }}", peer_id, num_peers),

            Event::PeerAccepted { peer_id, protocol, .. } =>
                write!(f, "Event::Accepted  {{ peer_id = {:?}, protocol = {:?} }}", peer_id, protocol),

            Event::PeerConnected { peer_id, num_conns } =>
                write!(f, "Event::PeerConnected: {{ peer_id = {:?}, num_conns = {} }}", peer_id, num_conns),

            Event::PeerDisconnected { peer_id, num_conns } =>
                write!(f, "Event::PeerDisconnected: {{ peer_id = {:?}, num_conns = {} }}", peer_id, num_conns),

            Event::PeerStalled { peer_id } =>
                write!(f, "Event::PeerStalled {{ peer_id = {:?} }}", peer_id),

            Event::BytesSent { to_peer, num_bytes } =>
                write!(f, "Event::BytesSent: {{ to_peer = {:?}, num_bytes = {:?} }})", to_peer, num_bytes),

            Event::BytesBroadcasted { num_bytes, num_conns } =>
                write!(f, "Event::BytesBroadcasted {{ num_bytes = {:?}, num_conns = {:?} }}", num_bytes, num_conns),

            Event::BytesReceived { from_peer, num_bytes, .. } =>
                write!(f, "Event::BytesReceived {{ from_peer = {:?}, num_bytes = {} }}", from_peer, num_bytes),

            Event::StreamStopped { from_peer } =>
                write!(f, "Event::StreamStopped: {{ from_peer = {:?} }})", from_peer),

            Event::TryConnect { peer_id } =>
                write!(f, "Event::TryConnect: {{ peer_id = {:?} }}", peer_id),
        }
    }
}

pub type EventPublisher = mpsc::Sender<Event>;
pub type EventSubscriber = mpsc::Receiver<Event>;

pub fn channel() -> (EventPublisher, EventSubscriber) {
    mpsc::channel(EVENT_CHAN_CAPACITY)
}
