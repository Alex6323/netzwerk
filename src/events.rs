use crate::address::Address;
use crate::peers::{Peer, PeerId};

use async_std::net::{TcpStream, UdpSocket};
use bytes::Bytes;
use crossbeam_channel as mpmc;

pub enum Event {
    NewPeer {
        peer: Peer
    },
    DropPeer {
        peer_id: PeerId
    },
    NewTcpConnection {
        peer_id: PeerId,
        stream: TcpStream
    },
    DropTcpConnection {
        peer_id: PeerId,
    },
    NewUdpConnection {
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

pub type EventRx = mpmc::Receiver<Event>;
pub type EventTx = mpmc::Sender<Event>;

pub fn channel() -> (EventTx, EventRx) {
    mpmc::unbounded()
}
