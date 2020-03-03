use crate::address::{Address, Url};
use crate::conns::BytesSender;
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

    /// Raised when an actor was spawned to handle a new connection.
    /// TODO: maybe rename to 'ConnActorStarted'?
    PeerAccepted {
        peer_id: PeerId,
        peer_url: Url,
        sender: BytesSender,
    },

    /// Raised when a peer is connected.
    PeerConnected {
        peer_id: PeerId,
        num_conns: usize,

        /// The Unix timestamp when the connection was established.
        ///
        /// NOTE: If the handshake message (done by some downstream library) reveals
        /// a duplicate connection, then this timestamp can be used to decide which
        /// of those to drop by some agreed policy.
        timestamp: u64,
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
    SendRecvStopped {
        peer_id: PeerId,
    },

    /// Raised when the system should try to (re)connect to a peer after a certain delay.
    TryConnect {
        peer_id: PeerId,

        /// None:    do not try to connect when adding the peer
        /// Some(0): indefinitely try to connect to this peer
        /// Some(n): try n times to connect to this peer
        retry: Option<usize>,
    }
}

impl fmt::Debug for Event {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Event::PeerAdded { peer_id, num_peers } =>
                write!(f, "Event::PeerAdded {{ peer_id = {:?}, num_peers = {} }}", peer_id, num_peers),

            Event::PeerRemoved { peer_id, num_peers } =>
                write!(f, "Event::PeerRemoved  {{ peer_id = {:?}, num_peers = {} }}", peer_id, num_peers),

            Event::PeerAccepted { peer_id, peer_url, .. } =>
                write!(f, "Event::PeerAccepted  {{ peer_id = {:?}, protocol = {:?} }}", peer_id, peer_url.protocol()),

            Event::PeerConnected { peer_id, num_conns, timestamp } =>
                write!(f, "Event::PeerConnected: {{ peer_id = {:?}, num_conns = {}, ts = {} }}", peer_id, num_conns, timestamp),

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

            Event::SendRecvStopped { peer_id } =>
                write!(f, "Event::SendRecvStopped: {{ peer_id = {:?} }})", peer_id),

            Event::TryConnect { peer_id, retry } =>
                write!(f, "Event::TryConnect: {{ peer_id = {:?}, retry_attempts = {:?} }}", peer_id, retry),
        }
    }
}

pub type EventPublisher = mpsc::Sender<Event>;
pub type EventSubscriber = mpsc::Receiver<Event>;

pub fn channel() -> (EventPublisher, EventSubscriber) {
    mpsc::channel(EVENT_CHAN_CAPACITY)
}
