use crate::commands::{Command, CommandReceiver};
use crate::errors;
use crate::events::{Event, EventPublisher as EventPub, EventSubscriber as EventSub};
use crate::peers::PeerId;
use crate::tcp::TcpConnection;

use async_trait::async_trait;

use std::collections::HashMap;
use std::result;

use log::*;
use crossbeam_channel::select;


pub(crate) const MAX_BUFFER_SIZE: usize = 1604;

pub type ConnectionResult<T> = result::Result<T, errors::ConnectionError>;

// NOTE: using this attribute implies heap allocation.
#[async_trait]
pub trait RawConnection {
    fn peer_id(&self) -> PeerId;

    // TODO: SendResult
    async fn send(&mut self, bytes: Vec<u8>) -> ConnectionResult<Event>;

    // TODO: RecvResult
    async fn recv(&mut self) -> ConnectionResult<Event>;
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

    pub async fn broadcast(&mut self, bytes: Vec<u8>) -> ConnectionResult<()> {
        for (_, peer_conn) in &mut self.0 {
            peer_conn.send(bytes.clone()).await.expect("error broadcasting to peer");
        }
        Ok(())
    }

    pub async fn send(&mut self, bytes: Vec<u8>, to_peer: &PeerId) -> ConnectionResult<Event> {
        if !self.0.contains_key(to_peer) {
            return Err(errors::ConnectionError::UnknownPeer);
        }

        let peer_conn = self.0.get_mut(to_peer).unwrap();

        Ok(peer_conn.send(bytes).await?)
    }

    pub async fn recv(&mut self, from_peer: &PeerId) -> ConnectionResult<Event> {
        if !self.0.contains_key(from_peer) {
            return Err(errors::ConnectionError::UnknownPeer);
        }

        let peer_conn = self.0.get_mut(from_peer).unwrap();

        Ok(peer_conn.recv().await?)
    }
}

pub mod actor {
    use super::*;

    pub async fn run(command_rx: CommandReceiver, event_pub: EventPub, event_sub: EventSub) {
        debug!("[Conns] Starting actor");

        let mut tcp_conns: Connections<TcpConnection> = Connections::new();
        //let mut udp_conns = Connections::new();

        loop {
            select! {
                // handle commands
                recv(command_rx) -> command => {
                    let command = command.expect("error receiving command");
                    debug!("[Conns] Received: {:?}", command);

                    match command {
                        Command::RemovePeer { peer_id } => {
                            // === TCP ===
                            // when removing the connections associated sockets will be closed automatically (RAII)
                            let was_removed = tcp_conns.remove(&peer_id);
                            if was_removed {
                                event_pub.send(Event::PeerDisconnected { peer_id, reconnect: None }.into());
                            }
                            // === UDP ===
                            /*
                            let was_removed = udp_conns.remove(&peer_id);
                            if was_removed {
                                event_pub.send(Event::PeerDisconnected { peer_id, reconnect: None }.into());
                            }
                            */

                        },
                        Command::SendBytes { receiver, bytes } => {
                            // === TCP ===
                            // FIXME: error handling
                            if let Some(conn) = tcp_conns.get_mut(&receiver) {
                                //conn.send(bytes.clone()).await.expect("error sending message using TCP");

                            }
                            // === UDP ===
                            /*
                            else if let Some(conn) = udp_conns.get_mut(&receiver) {
                                //conn.send(bytes.clone()).await.expect("error sending message using UDP");
                            }
                            */
                        }
                        Command::BroadcastBytes { bytes } => {
                            // === TCP ===
                            // FIXME: error handling
                            if tcp_conns.num() > 0 {
                                //tcp_conns.broadcast(bytes.clone()).await.expect("error broadcasting message using TCP");
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
                recv(event_sub) -> event => {
                    let event = event.expect("error receiving event");
                    debug!("[Conns] Received: {:?}", event);

                    match event {
                        Event::PeerConnectedViaTCP { peer_id, tcp_conn } => {
                            // TODO

                        }
                        _ => (),
                    }
                }
            }
        }

        debug!("[Conns] Stopping actor");
    }
}
