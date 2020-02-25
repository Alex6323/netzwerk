use crate::address::Address;
use crate::conns::{MAX_BUFFER_SIZE, SendResult, RawConnection, RecvResult};
use crate::events::{Event, EventPublisher as EventPub};
use crate::errors::{RecvError, SendError};
use crate::peers::PeerId;

use async_std::net::{SocketAddr, UdpSocket};
use async_std::prelude::*;
use async_std::sync::Arc;
use async_std::task;
use async_trait::async_trait;
use futures::sink::SinkExt;
use futures::{select, FutureExt};
use log::*;

/// Represents a UDP connection.
#[derive(Clone)]
pub struct UdpConnection {
    socket: Arc<UdpSocket>,
    peer_addr: SocketAddr,
}

impl UdpConnection {

    /// Creates a new TCP connection from a TCP stream instance.
    pub fn new(socket: UdpSocket, peer_addr: SocketAddr) -> Self {
        Self {
            socket: Arc::new(socket),
            peer_addr,
        }
    }

    /// Returns peer address associated with this connection.
    pub fn peer_addr(&self) -> SocketAddr {
        self.peer_addr
    }
}

impl Drop for UdpConnection {
    fn drop(&mut self) {
        // TODO
    }
}

impl Eq for UdpConnection {}
impl PartialEq for UdpConnection {
    fn eq(&self, other: &Self) -> bool {
        self.peer_addr() == other.peer_addr()
    }
}

#[async_trait]
impl RawConnection for UdpConnection {

    fn peer_id(&self) -> PeerId {
        PeerId(self.peer_addr())
    }

    async fn send(&mut self, bytes: Vec<u8>) -> SendResult<Event> {
        let num_bytes = self.socket.send_to(&bytes, &self.peer_addr()).await?;

        Ok(Event::BytesSent {
            num_bytes,
            receiver_addr: Address::new(self.peer_addr())
        }.into())
    }

    async fn recv(&mut self) -> RecvResult<Event> {
        let mut bytes = vec![0; MAX_BUFFER_SIZE];
        let mut socket = &*self.socket;

        let (num_bytes, peer_addr) = socket.recv_from(&mut bytes).await?;
        if peer_addr == self.peer_addr() {
            Ok(Event::BytesReceived {
                num_bytes,
                sender_addr: Address::new(peer_addr),
                bytes
            }.into())
        } else {
            Err(RecvError::UnknownPeerError)
        }
    }
}

/// Starts the UDP socket actor.
pub async fn actor(binding_addr: SocketAddr, event_prod: EventPub) {
    debug!("[UDP  ] Starting UDP socket actor");

    let socket = UdpSocket::bind(binding_addr).await.expect("error binding UDP socket");
    debug!("[UDP  ] Bound to {}",
        socket.local_addr().expect("error reading local address from UDP socket"));

    loop {
        let mut buf = vec![0; 1024];
        let (num_bytes, peer_addr) = socket.recv_from(&mut buf).await
            .expect("error receiving from UDP socket");

        info!("Received {} bytes from {}", num_bytes, peer_addr);

    }

    debug!("[UDP  ] Stopping UDP socket actor");
}

pub async fn connect(peer: &mut Peer) -> Option<()> {
    if let Url::Udp(peer_addr) = peer.url() {
        info!("[UDP  ] Trying to connect to peer: {:?} via UDP", peer.id());

        // TODO
        // A connection is considered established if we are receiving messages from that peer.
    } else {
        None
    }
}
