use crate::error;
use crate::message::Message;
use crate::result;
use crate::peer::PeerId;

use async_std::net::{TcpStream, UdpSocket, SocketAddr};
use async_trait::async_trait;
use bytes::Bytes;
use std::collections::HashMap;

const BUFFER_SIZE: usize = 1604;

#[async_trait]
pub trait RawIO {
    async fn send(&self, message: Message) -> result::Result<()>;
    async fn recv(&mut self) -> result::Result<Message>;
}

pub struct Udp {
    /// Handle to the underlying `UdpSocket` being used.
    socket: UdpSocket,

    /// Address to send messages to.
    peer_address: SocketAddr,
}

impl Udp {
    fn new(socket: UdpSocket, peer_address: SocketAddr) -> Self {
        // NOTE: the socket must be already bound
        Self { socket, peer_address }
    }
}

#[async_trait]
impl RawIO for Udp {
    async fn send(&self, message: Message) -> result::Result<()> {
        //(*self.socket).send_to(&message[..n], &self.address).await?;

        unimplemented!("UDP send")
    }
    async fn recv(&mut self) -> result::Result<Message> {
        let mut buf = vec![0u8; BUFFER_SIZE];
        let (n, peer_addr) = self.socket.recv_from(&mut buf).await?;
        // TODO: ignore, if `peer_addr` doesn't correspond to
        Ok(Message::Transaction(Bytes::from(buf)))
    }
}

pub struct Tcp {
    /// Handle to the underlying `TcpStream` being used.
    stream: TcpStream,
}

impl Tcp {
    fn new(stream: TcpStream) -> Self {
        Self { stream }
    }
}

#[async_trait]
impl RawIO for Tcp {
    async fn send(&self, message: Message) -> result::Result<()> {
        unimplemented!("TCP send")
    }
    async fn recv(&mut self) -> result::Result<Message> {
        unimplemented!("TCP recv")
    }
}

///
pub struct Connection<R: RawIO>(R);

pub type UdpConnection = Connection<Udp>;
pub type TcpConnection = Connection<Tcp>;

impl Connection<Tcp> {
    /// Creates a new TCP connection.
    pub fn new(stream: TcpStream) -> Self {
        Self(Tcp::new(stream))
    }
    pub async fn send(&self, message: Message) -> result::Result<()> {
        Ok(self.0.send(message).await?)
    }
    pub async fn recv(&mut self) -> result::Result<Message> {
        Ok(self.0.recv().await?)
    }
}

impl Connection<Udp> {
    /// Creates a new UDP connection.
    pub fn new(udp_socket: UdpSocket, peer_address: SocketAddr) -> Self {
        Self(Udp::new(udp_socket, peer_address))
    }
    pub async fn send(&self, message: Message) -> result::Result<()> {
        Ok(self.0.send(message).await?)
    }
    pub async fn recv(&mut self) -> result::Result<Message> {
        Ok(self.0.recv().await?)
    }
}

/// A registry for all established connections.
pub struct Connections<R: RawIO>(HashMap<PeerId, Connection<R>>);

impl<R: RawIO> Connections<R> {
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    async fn send(&self, message: Message, to_peer: PeerId) -> result::Result<()> {
        if !self.0.contains_key(&to_peer) {
            return Err(error::Error::AttemptedSendingToUnknownPeer);
        }

        let peer_conn = self.0.get(&to_peer).unwrap();
        Ok(peer_conn.0.send(message).await?)
    }

    async fn recv(&mut self, from_peer: PeerId) -> result::Result<Message> {
        if !self.0.contains_key(&from_peer) {
            return Err(error::Error::AttemptedReceivingFromUnknownPeer);
        }

        let mut peer_conn = self.0.get_mut(&from_peer).unwrap();
        Ok(peer_conn.0.recv().await?)
    }
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Protocol {
    Tcp,
    Udp,
}

#[cfg(test)]
mod tests {
    use super::*;

    mod udp {
        use super::{Udp, UdpConnection};

        #[test]
        fn create_udp_connection() {

        }
    }

    mod tcp {
        use super::{Tcp, TcpConnection};

        #[test]
        fn create_tcp_connection() {

        }
    }
}