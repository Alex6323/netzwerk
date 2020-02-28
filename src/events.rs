use crate::address::{Address, Protocol};
use crate::conns::ByteSender;
use crate::peers::PeerId;

use std::fmt;
use std::time::Duration;

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
        //address: Address,
        //num_connections: usize,
    },

    /// Raised when a peer was disconnected.
    PeerDisconnected {
        peer_id: PeerId,
        reconnect: Option<u64>,
        //num_connections: usize,
    },

    /// Raised when no packet was received from this peer for some time.
    PeerStalled {
        peer_id: PeerId,
        since: Duration,
    },

    /// Raised when bytes have been sent to a peer.
    BytesSent {
        num_bytes: usize,
        to: PeerId,
    },

    /// Raised when bytes have been sent to all peers.
    BytesBroadcasted {
        num_bytes: usize,
        num_sends: usize,
    },

    /// Raised when bytes have been received.
    BytesReceived {
        num_bytes: usize,
        from: Address,
        bytes: Vec<u8>,
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

            Event::PeerConnected { peer_id } =>
                write!(f, "Event::PeerConnected: {{ peer_id = {:?} }}", peer_id),

            /*
            Event::PeerConnectedOverUdp { peer_id, address } =>
                write!(f, "Event::PeerConnectedOverUdp {{  peer_id = {:?}, address = {:?}", peer_id, address),
            */

            Event::PeerDisconnected { peer_id, reconnect } =>
                write!(f, "Event::PeerDisconnected: {{ peer_id = {:?}, reconnect = {:?} }}", peer_id, reconnect),

            Event::PeerStalled { peer_id, since } =>
                write!(f, "Event::PeerStalled {{ peer_id = {:?}, since = {:?} ms }}", peer_id, since),

            Event::BytesSent { num_bytes, to } =>
                write!(f, "Event::BytesSent: {{ num_bytes = {:?}, to = {:?} }})", num_bytes, to),

            Event::BytesBroadcasted { num_bytes, num_sends } =>
                write!(f, "Event::BytesBroadcasted {{ num_bytes = {:?}, num_sends = {:?} }}", num_bytes, num_sends),

            Event::BytesReceived { num_bytes, from, .. } =>
                write!(f, "Event::BytesReceived {{ num_bytes = {:?}, from = {:?} }}", num_bytes, from),

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
