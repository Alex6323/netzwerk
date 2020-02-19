use crate::errors;
use crate::events::{Event, EventSource, EventType};
use crate::commands::{CommandReceiver, CommandType};
use crate::result;
use crate::peers::PeerId;
use crate::tcp::TcpConnection;
//use crate::udp::UdpConnection;

use std::collections::HashMap;

use async_trait::async_trait;
use log::*;

pub(crate) const MAX_BUFFER_SIZE: usize = 1604;

pub async fn listen(command_rx: CommandReceiver, event_source: EventSource) {
    debug!("Start listening to connection changes");

    let mut tcp_conns = Connections::new();
    let mut udp_conns = Connections::new();

    while let Ok(command) = command_rx.recv() {
        debug!("New connection command received: {:?}", command);

        match &*command {
            CommandType::RemovePeer { peer_id } => {
                // when removing the connections associated sockets will be closed automatically (RAII)
                let was_removed = tcp_conns.remove(&peer_id);
                if was_removed {
                    event_source.send(EventType::TcpPeerDisconnected { peer_id: *peer_id }.into());
                }

                let was_removed = udp_conns.remove(&peer_id);
                if was_removed {
                    event_source.send(EventType::UdpPeerDisconnected { peer_id: *peer_id }.into());
                }

            },
            CommandType::SendBytes { receiver, bytes } => {
                // FIXME: error handling
                if let Some(conn) = tcp_conns.get_mut(receiver) {
                    conn.send(bytes.clone()).await.expect("error sending message using TCP");

                } else if let Some(conn) = udp_conns.get_mut(receiver) {
                    conn.send(bytes.clone()).await.expect("error sending message using UDP");
                }
            }
            CommandType::BroadcastBytes { bytes } => {
                // FIXME: error handling
                if tcp_conns.num() > 0 {
                    tcp_conns.broadcast(bytes.clone()).await.expect("error broadcasting message using TCP");
                }

                if udp_conns.num() > 0 {
                    udp_conns.broadcast(bytes.clone()).await.expect("error broadcasting message using UDP");
                }
            }
            CommandType::Shutdown => {
                drop(tcp_conns);
                drop(udp_conns);
                event_source.send(EventType::Shutdown {}.into());
                break
            },
            _ => (),
        }
    }

    debug!("Connection listener stops listening");
}

#[async_trait]
pub trait NetIO {
    async fn send(&mut self, bytes: Vec<u8>) -> result::MessageResult<Event>;
    async fn recv(&mut self) -> result::MessageResult<Event>;
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

    pub fn remove(&mut self, peer_id: &PeerId) -> bool {
        self.0.remove(peer_id).is_some()
    }

    pub fn get(&self, peer_id: &PeerId) -> Option<&Connection<R>> {
        self.0.get(peer_id)
    }

    pub fn get_mut(&mut self, peer_id: &PeerId) -> Option<&mut Connection<R>> {
        self.0.get_mut(peer_id)
    }

    pub async fn broadcast(&mut self, bytes: Vec<u8>) -> result::MessageResult<()> {
        // TODO: proper error handling
        // TODO: handle broken peer connections
        for (_, peer_conn) in &mut self.0 {
            peer_conn.0.send(bytes.clone()).await.expect("error broadcasting to peer");
        }

        Ok(())
    }

    pub async fn send(&mut self, bytes: Vec<u8>, to_peer: &PeerId) -> result::MessageResult<Event> {
        if !self.0.contains_key(to_peer) {
            return Err(errors::MessageError::AttemptedSendingToUnknownPeer);
        }

        let peer_conn = self.0.get_mut(to_peer).unwrap();

        Ok(peer_conn.0.send(bytes).await?)
    }

    pub async fn recv(&mut self, from_peer: &PeerId) -> result::MessageResult<Event> {
        if !self.0.contains_key(from_peer) {
            return Err(errors::MessageError::AttemptedReceivingFromUnknownPeer);
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

impl Protocol {
    pub fn is_tcp(&self) -> bool {
        *self == Protocol::Tcp
    }

    pub fn is_udp(&self) -> bool {
        *self == Protocol::Udp
    }
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