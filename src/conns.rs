use crate::errors::{SendError, RecvError};
use crate::events::Event;
use crate::peers::PeerId;

use async_trait::async_trait;
use futures::channel::mpsc;

use std::collections::HashMap;
use std::result;

pub(crate) const MAX_BUFFER_SIZE: usize = 1604;
pub(crate) const RECONNECT_COOLDOWN: u64 = 5000;

pub type SendResult<T> = result::Result<T, SendError>;
pub type RecvResult<T> = result::Result<T, RecvError>;

const WIRE_CHAN_CAPACITY: usize = 10000;

// NOTE: using this attribute implies heap allocation.
#[async_trait]
pub trait RawConnection {

    fn peer_id(&self) -> PeerId;

    async fn send(&mut self, bytes: Vec<u8>) -> SendResult<Event>;

    async fn recv(&mut self) -> RecvResult<Event>;
}

pub struct Connections<C: RawConnection>(pub(crate) HashMap<PeerId, C>);

impl<C: RawConnection> Connections<C> {
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    pub fn num(&self) -> usize {
        self.0.len()
    }

    pub fn insert(&mut self, peer_id: PeerId, conn: C) {
        self.0.insert(peer_id, conn);
    }

    pub fn remove(&mut self, peer_id: &PeerId) -> bool {
        self.0.remove(peer_id).is_some()
    }

    pub fn get(&self, peer_id: &PeerId) -> Option<&C> {
        self.0.get(peer_id)
    }

    pub fn get_mut(&mut self, peer_id: &PeerId) -> Option<&mut C> {
        self.0.get_mut(peer_id)
    }

    pub async fn broadcast(&mut self, bytes: Vec<u8>) -> SendResult<()> {
        for (_, peer_conn) in &mut self.0 {
            peer_conn.send(bytes.clone()).await.expect("error broadcasting to peer");
        }
        Ok(())
    }

    pub async fn send(&mut self, bytes: Vec<u8>, to_peer: &PeerId) -> SendResult<Event> {
        let peer_conn = if !self.0.contains_key(to_peer) {
            return Err(SendError::UnknownPeer);
        } else {
            self.0.get_mut(to_peer).unwrap()
        };

        Ok(peer_conn.send(bytes).await?)
    }

    pub async fn recv(&mut self, from_peer: &PeerId) -> RecvResult<Event> {
        let peer_conn = if !self.0.contains_key(from_peer) {
            return Err(RecvError::UnknownPeer);
        } else {
            self.0.get_mut(from_peer).unwrap()
        };

        Ok(peer_conn.recv().await?)
    }
}

pub type ByteSender = mpsc::Sender<Vec<u8>>;
pub type ByteReceiver = mpsc::Receiver<Vec<u8>>;

pub fn channel() -> (ByteSender, ByteReceiver) {
    mpsc::channel(WIRE_CHAN_CAPACITY)
}

/*
pub async fn actor(mut command_rx: CommandRx, mut conn_rx: EventSub, mut conns_tx: EventPub) {
    debug!("[Conns] Starting actor");

    let mut tcp_conns: Connections<TcpConnection> = Connections::new();
    //let mut udp_conns = Connections::new();

    loop {
        select! {
            // handle commands
            command = command_rx.next().fuse() => {
                let command = command.expect("error receiving command");
                debug!("[Conns] Received: {:?}", command);

                match command {
                    Command::RemovePeer { peer_id } => {
                        // === TCP ===
                        // when removing the connections associated sockets will be closed automatically (RAII)
                        let was_removed = tcp_conns.remove(&peer_id);
                        if was_removed {
                            conns_tx.send(Event::PeerDisconnected { peer_id, reconnect: None }.into());
                        }
                        // === UDP ===
                        /*
                        let was_removed = udp_conns.remove(&peer_id);
                        if was_removed {
                            conns_tx.send(Event::PeerDisconnected { peer_id, reconnect: None }.into());
                        }
                        */

                    },
                    Command::SendBytes { receiver, bytes } => {
                        // === TCP ===
                        // FIXME: error handling
                        if let Some(conn) = tcp_conns.get_mut(&receiver) {
                            conn.send(bytes.clone()).await.expect("error sending message using TCP");

                        }
                        // === UDP ===
                        /*
                        else if let Some(conn) = udp_conns.get_mut(&receiver) {
                            conn.send(bytes.clone()).await.expect("error sending message using UDP");
                        }
                        */
                    }
                    Command::BroadcastBytes { bytes } => {
                        // === TCP ===
                        // FIXME: error handling
                        if tcp_conns.num() > 0 {
                            tcp_conns.broadcast(bytes.clone()).await.expect("error broadcasting message using TCP");
                        }

                        // === UDP ===
                        /*
                        if udp_conns.num() > 0 {
                            //udp_conns.broadcast(bytes.clone()).await.expect("error broadcasting message using UDP");
                        }
                        */
                    }
                    Command::Shutdown => {
                        drop(tcp_conns);
                        //drop(udp_conns);
                        break
                    },
                    _ => (),
                }
            },

            // handle events
            event = conn_rx.next().fuse() => {
                let event = event.expect("error receiving event");
                debug!("[Conns] Received: {:?}", event);

                match event {
                    Event::PeerConnectedViaTCP { peer_id, tcp_conn } => {
                        // TODO
                        info!("received PeerConnectedViaTCP event");

                    }
                    _ => (),
                }
            }
        }
    }

    debug!("[Conns] Stopping actor");
}
*/
