use crate::address::{Address, Url, Protocol};
use crate::commands::{Command, CommandReceiver as CommandRx};
use crate::conns::{BytesSender, RawConnection, RECONNECT_COOLDOWN};
use crate::errors;
use crate::events::{Event, EventPublisher as EventPub, EventSubscriber as EventSub};
use crate::tcp;

use async_std::task::{self, spawn};
use async_std::prelude::*;
use futures::{select, FutureExt};
use futures::sink::SinkExt;
use log::*;

use std::collections::HashMap;
use std::fmt;
use std::result;
use std::time::Duration;

pub type PeerResult<T> = result::Result<T, errors::PeerError>;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct PeerId(pub(crate) Address);

impl From<Address> for PeerId {
    fn from(addr: Address) -> Self {
        Self(addr)
    }
}

impl From<Url> for PeerId {
    fn from(url: Url) -> Self {
        use Url::*;

        match url {
            Tcp(socket_addr) |
            Udp(socket_addr) => Self(Address::Ip(socket_addr)),
            _ => panic!("Unsupported protocol"),
        }
    }
}

impl fmt::Display for PeerId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
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

    /// The timestamp of the last packet received from this peer.
    last_recv: u64,
}

impl Peer {
    pub fn from_url(url: Url) -> Self {
        Self { id: url.into(), url, state: Default::default(), last_recv: 0 }
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

    pub fn is_stalled(&self) -> bool {
        self.state.stalled()
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

    pub fn remove(&mut self, peer_id: &PeerId) -> bool {
        self.0.remove(peer_id).is_some()
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

    pub fn contains(&self, peer_id: &PeerId) -> bool {
        self.0.contains_key(peer_id)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PeerState {
    /// We are not connected to that peer.
    NotConnected,

    /// We are connected to that peer.
    Connected,

    /// We are connected to that peer, but it is not sending any messages anymore.
    Stalled,
}

impl PeerState {
    pub fn connected(&self) -> bool {
        *self == PeerState::Connected
    }
    pub fn stalled(&self) -> bool {
        *self == PeerState::Stalled
    }
}

impl Default for PeerState {
    fn default() -> Self {
        PeerState::NotConnected
    }
}

/// Starts the peers actor.
pub async fn actor(mut command_rx: CommandRx, mut event_sub: EventSub, mut event_pub: EventPub, mut net_pub: EventPub) {
    debug!("[Peers] Starting actor");

    let mut server_peers = Peers::new();
    let mut client_peers = Peers::new();

    let mut tcp_conns: HashMap<PeerId, BytesSender> = HashMap::new();
    let mut udp_conns: HashMap<PeerId, BytesSender> = HashMap::new();

    loop {

        select! {
            // === Handle commands ===
            command = command_rx.next().fuse() => {
                if command.is_none() {
                    debug!("[Peers] Commands channel closed");
                    break;
                }
                let command = command.unwrap();
                info!("[Peers] {:?}", command);

                match command {
                    Command::AddPeer { mut peer } => {
                        server_peers.add(peer.clone());

                        let num_peers = server_peers.num() + client_peers.num();

                        event_pub.send(Event::PeerAdded {
                            peer_id: peer.id(),
                            num_peers }).await;

                        // Immediatedly try to connect to that peer
                        // TODO: make this optional
                        event_pub.send(Event::TryConnect { peer_id: peer.id() }).await;
                    },
                    Command::RemovePeer { peer_id } => {
                        server_peers.remove(&peer_id);
                        client_peers.remove(&peer_id);

                        // TODO: check if this correctly ends the connection actor
                        tcp_conns.remove(&peer_id);
                        udp_conns.remove(&peer_id);

                        let num_peers = server_peers.num() + client_peers.num();

                        event_pub.send(Event::PeerRemoved { peer_id, num_peers }).await;
                    },
                    Command::SendBytes { to_peer, bytes } => {
                        let num_bytes = bytes.len();

                        if let Some(sender) = tcp_conns.get_mut(&to_peer) {
                            sender.send(bytes).await;

                        } else if let Some(sender) = udp_conns.get_mut(&to_peer) {
                            sender.send(bytes).await;

                        } else {
                            warn!("[Peers] No connection with peer {:?}", to_peer);
                            continue;
                        }

                        //event_pub.send(Event::BytesSent { to_peer, num_bytes }).await;
                    },
                    Command::BroadcastBytes { bytes } => {
                        let mut num_conns = 0;

                        // TODO: send concurrently
                        // TODO: do not clone the bytes (use Arc!)
                        for (_, sender) in tcp_conns.iter_mut() {
                            sender.send(bytes.clone()).await;
                            num_conns += 1;
                        }

                        // TODO: send concurrently
                        // TODO: do not clone the bytes (use Arc!)
                        for (_, sender) in udp_conns.iter_mut() {
                            sender.send(bytes.clone()).await;
                            num_conns += 1;
                        }

                        if num_conns == 0 {
                            warn!("[Peers] No connections available for broadcast.");
                            continue;
                        }

                        // FIXME: this never shows in the logs. Why?
                        event_pub.send(Event::BytesBroadcasted {
                            num_bytes: bytes.len(),
                            num_conns,
                        });
                    },
                    Command::Shutdown => {
                        drop(tcp_conns);
                        drop(udp_conns);
                        drop(server_peers);
                        drop(client_peers);
                        break;
                    }
                }
            },

            // === Handle peer events ===
            peer_event = event_sub.next().fuse() => {

                // TODO: replace this with 'unwrap_or_else(|| break)'
                if peer_event.is_none() {
                    debug!("[Peers] Event channel closed");
                    break;
                }
                let peer_event = peer_event.unwrap();
                    info!("[Peers] {:?}", peer_event);

                match peer_event {
                    Event::PeerAdded { peer_id, num_peers } => {
                        // notify user
                        net_pub.send(Event::PeerAdded { peer_id, num_peers }).await;
                    },
                    Event::PeerRemoved { peer_id, num_peers } => {
                        // notify user
                        net_pub.send(Event::PeerRemoved { peer_id, num_peers }).await;
                    },
                    Event::PeerAccepted { peer_id, peer_url, sender } => {
                        use Protocol::*;

                        let protocol = peer_url.protocol();

                        // NOTE: we let users deal with duplicate peers because for that
                        // handshakes are required.
                        if !server_peers.contains(&peer_id) &&
                           !client_peers.contains(&peer_id) {
                            client_peers.add(Peer::from_url(peer_url));
                        }

                        // TODO: use Entry API
                        match protocol {
                            Tcp => {
                                if !tcp_conns.contains_key(&peer_id) {
                                    tcp_conns.insert(peer_id, sender);
                                }
                            },
                            Udp => {
                                if !udp_conns.contains_key(&peer_id) {
                                    udp_conns.insert(peer_id, sender);
                                }
                            },
                            _ => (),
                        }

                        let num_conns = tcp_conns.len() + udp_conns.len();

                        event_pub.send(Event::PeerConnected { peer_id, num_conns }).await
                            .expect("[Peers] Error sending 'PeerConnected' event");
                    }
                    Event::PeerConnected { peer_id, num_conns } => {

                        let mut peer = {
                            if let Some(peer) = server_peers.get_mut(&peer_id) {
                                peer
                            } else if let Some(peer) = client_peers.get_mut(&peer_id) {
                                peer
                            } else {
                                error!("[Peers] Peer lists is out-of-sync. This should never happen.");
                                continue;
                            }
                        };

                        peer.set_state(PeerState::Connected);

                        // notify user
                        net_pub.send(Event::PeerConnected { peer_id, num_conns }).await;
                    },
                    Event::SendRecvStopped { peer_id } => {

                        tcp_conns.remove(&peer_id);
                        udp_conns.remove(&peer_id);

                        // try to reconnect to server peers
                        if let Some(peer) = server_peers.get_mut(&peer_id) {

                            peer.set_state(PeerState::NotConnected);

                            raise_event_after_delay(Event::TryConnect { peer_id }, RECONNECT_COOLDOWN, &event_pub);

                        } else if client_peers.contains(&peer_id) {

                            client_peers.remove(&peer_id);

                        } else {
                            error!("[Peers] Peer lists is out-of-sync. This should never happen.");
                            continue;
                        }

                        let num_conns = tcp_conns.len() + udp_conns.len();

                        event_pub.send(Event::PeerDisconnected { peer_id, num_conns }).await;

                    },
                    Event::PeerDisconnected { peer_id, num_conns } => {

                        // notify user
                        net_pub.send(Event::PeerDisconnected { peer_id, num_conns }).await;
                    }
                    Event::PeerStalled { peer_id } => {

                        let mut peer = {
                            if let Some(peer) = server_peers.get_mut(&peer_id) {
                                peer
                            } else if let Some(peer) = client_peers.get_mut(&peer_id) {
                                peer
                            } else {
                                error!("[Peers] Peer lists is out-of-sync. This should never happen.");
                                continue;
                            }
                        };

                        peer.set_state(PeerState::Stalled);

                        // notify user
                        net_pub.send(Event::PeerStalled { peer_id }).await;
                    },
                    Event::BytesSent { to_peer, num_bytes, .. } => {},
                    Event::BytesBroadcasted { num_bytes, num_conns } => {}
                    Event::BytesReceived { from_peer, with_addr, num_bytes, buffer } => {

                        // notify user
                        net_pub.send(Event::BytesReceived { from_peer, with_addr, num_bytes, buffer }).await;
                    },
                    Event::TryConnect { peer_id } => {

                        let mut peer = {
                            if let Some(peer) = server_peers.get_mut(&peer_id) {
                                peer
                            } else if let Some(peer) = client_peers.get_mut(&peer_id) {
                                peer
                            } else {
                                error!("[Peers] Peer lists is out-of-sync. This should never happen.");
                                continue;
                            }
                        };

                        // NOTE: this event happens after a certain time interval, so once it is raised
                        // the peer might already be connected.
                        if peer.is_connected() {
                            continue;
                        }

                        use Url::*;
                        match peer.url() {
                            Tcp(peer_addr) => {

                                if !tcp::connect(&peer.id(), peer_addr, event_pub.clone()).await {

                                    info!("[Peers] Connection attempt failed. Retrying in {} ms", RECONNECT_COOLDOWN);

                                    raise_event_after_delay(Event::TryConnect { peer_id }, RECONNECT_COOLDOWN, &event_pub);

                                }
                            },
                            Udp(peer_addr) => {
                                // TODO
                            },
                            _ => (),
                        }
                    },
                }
            }
        }
    }

    debug!("[Peers] Stopping actor");
}

fn raise_event_after_delay(event: Event, after: u64, event_pub: &EventPub) {
    let mut event_pub = event_pub.clone();

    // finished once it has waited and send the event
    spawn(async move {
        task::sleep(Duration::from_millis(after)).await;

        event_pub.send(event).await
            .expect("[Peers] Error sending event after delay");
    });
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
