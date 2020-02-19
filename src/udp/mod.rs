use crate::connections::{Connection, MAX_BUFFER_SIZE, NetIO};
use crate::events::EventProducer;
use crate::result;

use async_std::net::{UdpSocket, SocketAddr};
use async_std::task;
use async_trait::async_trait;
use log::*;

pub async fn init(binding_addr: SocketAddr, event_prod: EventProducer) {
    let socket = UdpSocket::bind(binding_addr).await.expect("error binding UDP socket");
    debug!("Successfully bound UDP socket to <<{}>>",
        socket.local_addr().expect("error reading local address from UDP socket"));
    debug!("Starting udp processor");
    // TODO: send UdpSocketBound message
    debug!("Exited udp processor");
}

pub struct Udp {
    bound_socket: UdpSocket,
    peer_address: SocketAddr,
}

impl Udp {
    fn new(bound_socket: UdpSocket, peer_address: SocketAddr) -> Self {
        Self { bound_socket, peer_address }
    }
}

#[async_trait]
impl NetIO for Udp {

    async fn send(&mut self, bytes: Vec<u8>) -> result::MessageResult<usize> {
        let nb = self.bound_socket.send_to(&bytes, &self.peer_address).await?;

        Ok(nb)
    }

    async fn recv(&mut self) -> result::MessageResult<Vec<u8>> {
        let mut buffer = vec![0u8; MAX_BUFFER_SIZE];

        let (nb, peer_address) = self.bound_socket.recv_from(&mut buffer).await?;

        if peer_address != self.peer_address {
            // NOTE: not sure if this makes sense.
            Ok(Bytes::new())
        } else {
            Ok(Bytes::from(buffer))
        }

    }
}

impl Connection<Udp> {
    pub fn new(udp_socket: UdpSocket, peer_address: SocketAddr) -> Self {
        Self(Udp::new(udp_socket, peer_address))
    }
    pub async fn send(&mut self, bytes: Bytes) -> result::Result<()> {
        Ok(self.0.send(bytes).await?)
    }
    pub async fn recv(&mut self) -> result::Result<Bytes> {
        Ok(self.0.recv().await?)
    }
}

pub type UdpConnection = Connection<Udp>;
