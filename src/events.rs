use crate::address::Address;
use crate::peers::{Peer, PeerId};

use std::fmt;

use async_std::net::{TcpStream, UdpSocket};
use bytes::Bytes;
use crossbeam_channel as mpmc;

pub enum Event {
    AddPeer {
        peer: Peer
    },
    DropPeer {
        peer_id: PeerId
    },
    AddTcpConnection {
        peer_id: PeerId,
        stream: TcpStream
    },
    DropTcpConnection {
        peer_id: PeerId,
    },
    AddUdpConnection {
        peer_id: PeerId,
        address: Address,
        socket: UdpSocket,
    },
    DropUdpConnection {
        peer_id: PeerId,
    },
    SendMessage {
        peer_id: PeerId,
        bytes: Bytes,
    },
    BroadcastMessage {
        bytes: Bytes,
    },
    Shutdown,
}

impl fmt::Debug for Event {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Event::AddPeer { peer } => write!(f, "AddPeer <<{:?}>>", peer.id()),
            Event::DropPeer { peer_id } => write!(f, "DropPeer <<{:?}>>", peer_id),
            Event::AddTcpConnection { peer_id, .. } => write!(f, "AddTcpConnection <<{:?}>>", peer_id),
            Event::DropTcpConnection { peer_id } => write!(f, "DropTcpConnection <<{:?}>>", peer_id),
            Event::AddUdpConnection { peer_id, .. } => write!(f, "AddUdpConnection <<{:?}>>", peer_id),
            Event::DropUdpConnection { peer_id } => write!(f, "DropUdpConnection <<{:?}>>", peer_id),
            Event::SendMessage { peer_id, bytes } => write!(f, "SendMessage <<{:?}>>, <<{} bytes>>", peer_id, bytes.len()),
            Event::BroadcastMessage { bytes } => write!(f, "BroadcastMessage <<{} bytes>>", bytes.len()),
            Event::Shutdown => write!(f, "Shutdown"),
        }
    }
}
pub type EventProducer = mpmc::Sender<Event>;
pub type EventReceiver = mpmc::Receiver<Event>;

pub fn channel() -> (EventProducer, EventReceiver) {
    mpmc::unbounded::<Event>()
}
