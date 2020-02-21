use crate::address::{Url, Protocol};
use crate::commands::{Command, CommandReceiver};
use crate::errors;
use crate::events::{Event, EventPublisher as EventPub, EventSubscriber as EventSub};
use crate::tcp::TcpConnection;

use async_std::net::{SocketAddr, TcpStream};
use async_std::task;
use crossbeam_channel::select;
use log::*;

use std::collections::HashMap;
use std::result;
use std::time::Duration;

pub type PeerResult<T> = result::Result<T, errors::PeerError>;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct PeerId(pub(crate) SocketAddr);

impl From<Url> for PeerId {
    fn from(url: Url) -> Self {
        match url {
            Url::Tcp(socket_addr) => Self(socket_addr),
            Url::Udp(socket_addr) => Self(socket_addr),

        }
    }
}

#[derive(Clone, Debug)]
pub struct Peer {
    /// The ID of the peer to recognize it across connections.
    id: PeerId,

    /// The URL of the peer, i.e. its address and network protocol.
    url: Url,

    /// The current state of the peer {NotConnected, Connected, ...}.
    state: PeerState,
}

impl Peer {
    pub fn from_url(url: Url) -> Self {
        Self { id: url.into(), url, state: Default::default() }
    }

    pub fn id(&self) -> PeerId {
        self.id
    }

    pub fn url(&self) -> &Url {
        &self.url
    }

    pub fn protocol(&self) -> Protocol {
        self.url.protocol()
    }

    pub fn is_connected(&self) -> bool {
        self.state.connected()
    }

    pub fn is_not_connected(&self) -> bool {
        !self.is_connected()
    }

    pub fn is_stale(&self) -> bool {
        self.state.stale()
    }

    pub fn state(&self) -> &PeerState {
        &self.state
    }

    pub fn set_state(&mut self, new_state: PeerState) {
        self.state = new_state;
    }
}

impl Eq for Peer {}
impl PartialEq for Peer {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

pub struct Peers(HashMap<PeerId, Peer>);

impl Peers {
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    pub fn num(&self) -> usize {
        self.0.len()
    }

    pub fn add(&mut self, peer: Peer) {
        if !self.0.contains_key(&peer.id()) {
            self.0.insert(peer.id(), peer);
        }
    }

    pub fn remove(&mut self, peer_id: &PeerId) {
        self.0.remove(peer_id);
    }

    pub fn get(&self, peer_id: &PeerId) -> Option<&Peer> {
        self.0.get(peer_id)
    }

    pub fn get_mut(&mut self, peer_id: &PeerId) -> Option<&mut Peer> {
        self.0.get_mut(peer_id)
    }

    pub fn keys(&self) -> std::collections::hash_map::Keys<PeerId, Peer> {
        self.0.keys()
    }

    pub fn values(&self) -> std::collections::hash_map::Values<PeerId, Peer> {
        self.0.values()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PeerState {
    /// We are not connected to that peer.
    NotConnected,

    /// We are connected to that peer.
    Connected,

    /// We are connected to that peer, but it is not sending any messages anymore.
    Stale,
}

impl PeerState {
    pub fn connected(&self) -> bool {
        *self == PeerState::Connected
    }
    pub fn stale(&self) -> bool {
        *self == PeerState::Stale
    }
}

impl Default for PeerState {
    fn default() -> Self {
        PeerState::NotConnected
    }
}

pub mod actor {
    use super::*;

    const RECONNECT_COOLDOWN: u64 = 1000;

    /// Starts the peers actor.
    pub async fn run(command_rx: CommandReceiver, event_pub: EventPub, event_sub: EventSub) {
        debug!("[Peers] Starting actor");

        let mut peers = Peers::new();

        loop {
            select! {
                // Handle commands
                recv(command_rx) -> command => {
                    let command = command.expect("error receiving command");
                    debug!("[Peers] Received: {:?}", command);

                    match command {
                        Command::AddPeer { peer } => {
                            peers.add(peer.clone());

                            event_pub.send(Event::PeerAdded { peer: peer.clone(), num_peers: peers.num() }.into())
                                .expect("error sending `PeerAdded` event");
                        },
                        Command::RemovePeer { peer_id } => {
                            peers.remove(&peer_id);

                            event_pub.send(Event::PeerRemoved { peer_id }.into())
                                .expect("error sending `PeerRemoved` event");
                        },
                        Command::Shutdown => {
                            drop(peers);
                            break
                        }
                        _ => (),
                    }
                },

                // handle events
                recv(event_sub) -> event => {
                    let event = event.expect("error receiving event");
                    debug!("[Peers] Received: {:?}", event);

                    match event {
                        Event::PeerAdded { mut peer, .. } => {
                            if peer.is_not_connected() {
                                if let Some(stream) = connect_to_peer(&mut peer) {
                                    event_pub.send(Event::PeerConnectedViaTCP {
                                        peer_id: peer.id(),
                                        tcp_conn: TcpConnection::new(stream),
                                    }.into()).expect("error sending event");
                                } else {
                                    event_pub.send(Event::PeerDisconnected {
                                        peer_id: peer.id(),
                                        reconnect: Some(RECONNECT_COOLDOWN)
                                    }).expect("error sending event");
                                }
                            }
                        },
                        Event::PeerDisconnected { peer_id, reconnect } => {
                            if let Some(duration) = reconnect {
                                let event_pub = event_pub.clone();
                                task::spawn(async move {
                                    task::sleep(Duration::from_millis(duration)).await;
                                    event_pub.send(Event::PeerReconnect { peer_id });
                                });
                            }
                        },
                        Event::PeerReconnect { peer_id } => {
                            if let Some(mut peer) = peers.get_mut(&peer_id) {
                                if peer.is_not_connected() {
                                    if let Some(stream) = connect_to_peer(&mut peer) {
                                        event_pub.send(Event::PeerConnectedViaTCP {
                                            peer_id: peer.id(),
                                            tcp_conn: TcpConnection::new(stream),
                                        }.into())
                                        .expect("error sending event");
                                    } else {
                                        event_pub.send(Event::PeerDisconnected {
                                            peer_id: peer.id(),
                                            reconnect: Some(RECONNECT_COOLDOWN)
                                        }).expect("error sending event");
                                    }
                                }
                            }
                        }
                        _ => (),
                    }
                }
            }
        }

        debug!("[Peers] Stopping actor");
    }

    fn connect_to_peer(peer: &mut Peer) -> Option<TcpStream> {
        match peer.url() {
            // === TCP ===
            Url::Tcp(peer_addr) => {
                /*
                let stream = TcpStream::connect(peer_addr)
                    .await
                    .expect("error connecting to peer");
                */

                // NOTE: unfortunately we have to do this (blocking) atm
                match task::block_on(TcpStream::connect(peer_addr)) {
                    Ok(stream) => {
                        peer.set_state(PeerState::Connected);

                        let peer_id = PeerId(stream.peer_addr()
                            .expect("error unwrapping remote address from TCP stream"));

                        // TODO: can this happen? when? What should we do then?
                        if peer.id() != peer_id {
                            warn!("Something fishy is going on. Non matching peer ids.");
                        }

                        Some(stream)
                    },
                    Err(_) => None,
                }

            },
            // === UDP ===
            Url::Udp(peer_addr) => {
                // TODO
                None
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod peer_id {
        use super::*;

        #[test]
        fn create_peer_id() {
            //let id = PeerId();
        }
    }

    mod peer {
        use super::Peer;
        use crate::address::Url;
        use async_std::task;

        #[test]
        fn create_new_peer() {
            task::block_on(async {
                let peer = Peer::from_url(Url::from_str("tcp://localhost:1337"));
                assert_eq!(peer.url().address().to_string(), "127.0.0.1:1337");
            })
        }

        #[test]
        fn new_peer_defaults_to_not_connected() {
            task::block_on(async {
                let peer = Peer::from_url(Url::from_str("tcp://127.0.0.1:1337"));
                assert!(peer.is_not_connected(), "Peer should initially default to 'NotConnected'");
            })
        }
    }

    mod peers {
        use super::{Peer, Peers};
        use crate::address::{Protocol, Url};
        use async_std::net::Ipv4Addr;

        #[test]
        fn create_new_peers() {
            let peers = Peers::new();

            assert_eq!(0, peers.num());
        }

        #[test]
        fn add_peers() {
            let mut peers = Peers::new();

            peers.add(Peer::from_url(Url::from_ipv4(Ipv4Addr::new(127, 0, 0, 1), Some(1337), Protocol::Udp)));
            peers.add(Peer::from_url(Url::from_ipv4(Ipv4Addr::new(127, 0, 0, 1), Some(1338), Protocol::Tcp)));

            assert_eq!(2, peers.num());
        }

    }
}
