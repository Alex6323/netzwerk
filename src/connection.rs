use crate::error;
use crate::message::Message;
use crate::result;
use crate::peers::PeerId;

use async_std::net::{TcpStream, UdpSocket, SocketAddr};
use async_trait::async_trait;
use bytes::Bytes;
use futures::channel::mpsc;
use std::collections::HashMap;
use std::ops;

// TODO: move Tcp/Udp stuff in their respective sub-modules.

const BUFFER_SIZE: usize = 1604;

#[async_trait]
pub trait RawIO {
    async fn send(&self, message: impl Message) -> result::Result<()>;
    async fn recv<M: Message>(&mut self) -> result::Result<M>;
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
    async fn send(&self, message: impl Message) -> result::Result<()> {
        //(*self.socket).send_to(&message[..n], &self.address).await?;

        unimplemented!("UDP send")
    }
    async fn recv<M: Message>(&mut self) -> result::Result<M> {
        let mut buf = vec![0u8; BUFFER_SIZE];
        let (n, peer_addr) = self.socket.recv_from(&mut buf).await?;

        // TODO: ignore, if `peer_addr` doesn't correspond to

        Ok(Message::from_bytes(Bytes::from(buf)).unwrap())
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
    async fn send(&self, message: impl Message) -> result::Result<()> {
        unimplemented!("TCP send")
    }
    async fn recv<M: Message>(&mut self) -> result::Result<M> {
        unimplemented!("TCP recv")
    }
}

///
pub struct Connection<R: RawIO>(R);

pub type UdpConnection = Connection<Udp>;
pub type TcpConnection = Connection<Tcp>;

impl Connection<Tcp> {
    pub fn new(stream: TcpStream) -> Self {
        Self(Tcp::new(stream))
    }
    pub async fn send(&self, message: impl Message) -> result::Result<()> {
        Ok(self.0.send(message).await?)
    }
    pub async fn recv<M: Message>(&mut self) -> result::Result<M> {
        Ok(self.0.recv().await?)
    }
}

impl Connection<Udp> {
    pub fn new(udp_socket: UdpSocket, peer_address: SocketAddr) -> Self {
        Self(Udp::new(udp_socket, peer_address))
    }
    pub async fn send(&self, message: impl Message) -> result::Result<()> {
        Ok(self.0.send(message).await?)
    }
    pub async fn recv<M: Message>(&mut self) -> result::Result<M> {
        Ok(self.0.recv().await?)
    }
}

/// A registry for all established connections.
pub struct Connections<R: RawIO>(HashMap<PeerId, Connection<R>>);

impl<R: RawIO> Connections<R> {
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    pub fn num(&self) -> usize {
        self.0.len()
    }

    pub fn insert(&mut self, peer_id: PeerId, conn: Connection<R>) {
        self.0.insert(peer_id, conn);
    }

    pub fn remove(&mut self, peer_id: &PeerId) {
        self.0.remove(peer_id);
    }

    pub fn get(&self, peer_id: &PeerId) -> Option<&Connection<R>> {
        self.0.get(peer_id)
    }

    pub fn get_mut(&mut self, peer_id: &PeerId) -> Option<&mut Connection<R>> {
        self.0.get_mut(peer_id)
    }

    pub fn broadcast(&self, message: impl Message) {
        // TODO: send message using all connections
    }

    async fn send(&self, message: impl Message, to_peer: PeerId) -> result::Result<()> {
        if !self.0.contains_key(&to_peer) {
            return Err(error::Error::AttemptedSendingToUnknownPeer);
        }

        let peer_conn = self.0.get(&to_peer).unwrap();
        Ok(peer_conn.0.send(message).await?)
    }

    async fn recv<M: Message>(&mut self, from_peer: PeerId) -> result::Result<M> {
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

impl From<&str> for Protocol {
    fn from(s: &str) -> Self {
        match s {
            "tcp" => Self::Tcp,
            "udp" => Self::Udp,
            _ => panic!("Unknown protocol specifier"),
        }
    }
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