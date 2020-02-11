use crate::error;
use crate::events::{Event, EventRx};
use crate::result;
use crate::peers::PeerId;
use crate::tcp::TcpConnection;
use crate::udp::UdpConnection;

use std::collections::HashMap;

use async_std::task;
use async_trait::async_trait;
use bytes::Bytes;

pub(crate) const MAX_BUFFER_SIZE: usize = 1604;

/// Starts the event loop.
pub fn start_el(event_rx: EventRx) {
    let mut tcp_conns = Connections::new();
    let mut udp_conns = Connections::new();

    task::spawn(async move {
        while let Ok(event) = event_rx.recv() {
            match event {
                Event::NewTcpConnection { peer_id, stream } => tcp_conns.insert(peer_id, TcpConnection::new(stream)),
                Event::DropTcpConnection { peer_id } => tcp_conns.remove(&peer_id),
                Event::NewUdpConnection { peer_id, address, socket } => {
                    udp_conns.insert(peer_id, UdpConnection::new(socket, address.socket_addr().unwrap()))
                },
                Event::BroadcastMessage { bytes } => {
                    // FIXME: error handling
                    tcp_conns.broadcast(bytes.clone()).await.expect("error broadcasting message using TCP");
                    udp_conns.broadcast(bytes).await.expect("error broadcasting message using UDP");
                }
                Event::SendMessage { peer_id, bytes } => {
                    // FIXME: error handling
                    if let Some(conn) = tcp_conns.get_mut(&peer_id) {
                        conn.send(bytes).await.expect("error sending message using TCP");
                    } else if let Some(conn) = udp_conns.get_mut(&peer_id) {
                        conn.send(bytes).await.expect("error sending message using UDP");
                    }
                }
                Event::Shutdown => break,
                _ => (),
            }
        }
    });
}

#[async_trait]
pub trait NetIO {
    async fn send(&mut self, bytes: Bytes) -> result::Result<()>;
    async fn recv(&mut self) -> result::Result<Bytes>;
}

pub struct Connection<N: NetIO>(pub(crate) N);

pub struct Connections<N: NetIO>(pub(crate) HashMap<PeerId, Connection<N>>);

impl<R: NetIO> Connections<R> {
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

    pub async fn broadcast(&mut self, bytes: Bytes) -> result::Result<()> {
        // TODO: proper error handling
        // TODO: handle broken peer connections
        for (_, peer_conn) in &mut self.0 {
            peer_conn.0.send(bytes.clone()).await.expect("error broadcasting to peer");
        }

        Ok(())
    }

    pub async fn send(&mut self, bytes: Bytes, to_peer: &PeerId) -> result::Result<()> {
        if !self.0.contains_key(to_peer) {
            return Err(error::Error::AttemptedSendingToUnknownPeer);
        }

        let peer_conn = self.0.get_mut(to_peer).unwrap();

        Ok(peer_conn.0.send(bytes).await?)
    }

    pub async fn recv(&mut self, from_peer: &PeerId) -> result::Result<Bytes> {
        if !self.0.contains_key(from_peer) {
            return Err(error::Error::AttemptedReceivingFromUnknownPeer);
        }

        let peer_conn = self.0.get_mut(from_peer).unwrap();

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