use crate::address::Address;
use crate::message::Message;
use crate::peers::{Peer, PeerId};

use async_std::net::{TcpStream, UdpSocket};
use crossbeam_channel as mpmc;
use std::sync::Arc;

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
        socket: Arc<UdpSocket>,
        address: Address,
    },
    DropUdpConnection {
        peer_id: PeerId,
    },
    /*
    SendMessage {
        peer_id: PeerId,
        message: dyn Message,
    },
    BroadcastMessage {
        message: dyn Message,
    },
    */
    Shutdown,
}

pub type EventRx = mpmc::Receiver<Event>;
pub type EventTx = mpmc::Sender<Event>;

pub fn channel() -> (EventTx, EventRx) {
    mpmc::unbounded()
}
