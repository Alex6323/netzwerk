use crate::address::{Url, Protocol};
use crate::commands::{Command, CommandReceiver as CommandRx};
use crate::conns::{self, ByteSender};
use crate::errors;
use crate::events::{Event, EventPublisher as EventPub, EventSubscriber as EventSub};
use crate::tcp;

use async_std::net::{IpAddr, SocketAddr};
use async_std::task::{self, spawn};
use async_std::prelude::*;
use futures::{select, FutureExt};
use futures::sink::SinkExt;
use log::*;

use std::collections::HashMap;
use std::result;
use std::time::Duration;

pub type PeerResult<T> = result::Result<T, errors::PeerError>;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct PeerId(pub(crate) IpAddr);

impl From<Url> for PeerId {
    fn from(url: Url) -> Self {
        match url {
            Url::Tcp(socket_addr) => Self(socket_addr.ip()),
            Url::Udp(socket_addr) => Self(socket_addr.ip()),
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

const RECONNECT_COOLDOWN: u64 = 5000;

/// Starts the peers actor.
pub async fn actor(mut command_rx: CommandRx, mut event_sub: EventSub, mut event_sub2: EventSub, mut event_pub: EventPub) {
    debug!("[Peers] Starting actor");

    let mut peers = Peers::new();
    let mut tcp_conns: HashMap<PeerId, ByteSender> = HashMap::new();
    let mut udp_conns: HashMap<PeerId, ByteSender> = HashMap::new();

    loop {
        select! {
            // === handle commands ===
            command = command_rx.next().fuse() => {
                let command = command.expect("[Peers] Error receiving command");
                debug!("[Peers] {:?}", command);

                match command {
                    Command::AddPeer { mut peer } => {
                        peers.add(peer.clone());

                        event_pub.send(Event::PeerAdded {
                            peer_id: peer.id(),
                            num_peers: peers.num() }.into()).await;

                        // Immediatedly try to connect to that peer
                        event_pub.send(Event::TryConnect {
                            peer_id: peer.id(),
                        }).await;
                    },
                    Command::RemovePeer { peer_id } => {
                        peers.remove(&peer_id);
                        tcp_conns.remove(&peer_id);
                        udp_conns.remove(&peer_id);

                        event_pub.send(Event::PeerRemoved { peer_id }.into()).await;
                    },
                    Command::SendBytes { to, bytes } => {
                        let nb = bytes.len();

                        if let Some(sender) = tcp_conns.get_mut(&to) {
                            sender.send(bytes).await;
                            info!("[Peers] Send {:?} bytes to {:?} over TCP", nb, to);

                        } else if let Some(sender) = udp_conns.get_mut(&to) {
                            sender.send(bytes).await;
                            info!("[Peers] Send {:?} bytes to {:?} over UDP", nb, to);

                        } else {
                            warn!("[Peers] No connection with peer {:?}", to);
                        }
                    },
                    Command::BroadcastBytes { bytes } => {
                        if tcp_conns.len() > 0 {
                            // TODO: send concurrently
                            for (_, sender) in tcp_conns.iter_mut() {
                                sender.send(bytes.clone()).await;
                            }
                            info!("[Peers] Broadcasted {:?} bytes to {:?} TCP peers",
                                bytes.len(), tcp_conns.len());
                        } else

                        if udp_conns.len() > 0 {
                            for (_, sender) in udp_conns.iter_mut() {
                                sender.send(bytes.clone()).await;
                            }
                            info!("[Peers] Broadcasted {:?} bytes to {:?} UDP peers",
                                bytes.len(), udp_conns.len());
                        } else {
                            warn!("[Peers] No connections available for broadcast.");
                        }
                    },
                    Command::Shutdown => {
                        drop(tcp_conns);
                        drop(udp_conns);
                        drop(peers);
                        break
                    }
                }
            },

            // === handle peer events ===
            peer_event = event_sub.next().fuse() => {
                if let Some(peer_event) = peer_event {
                    debug!("[Peers] {:?}", peer_event);

                    match peer_event {
                        Event::PeerAdded { .. } => (),
                        Event::PeerRemoved { .. } => (),
                        Event::PeerAccepted { peer_id, protocol, sender } => {
                            match protocol {
                                Protocol::Tcp => {
                                    if tcp_conns.contains_key(&peer_id) {
                                        continue;
                                    }
                                    tcp_conns.insert(peer_id, sender);
                                },
                                Protocol::Udp => {
                                    if udp_conns.contains_key(&peer_id) {
                                        continue;
                                    }
                                    udp_conns.insert(peer_id, sender);
                                }
                            }
                            event_pub.send(Event::PeerConnected { peer_id }).await
                                .expect("[Peers] Error sending event");
                        }
                        Event::PeerConnected { peer_id } => {

                            if let Some(peer) = peers.get_mut(&peer_id) {
                                peer.set_state(PeerState::Connected);
                            } else {
                                error!("[Peers] Peer list is out-of-sync. This should never happen.")
                            }
                        },
                        Event::PeerDisconnected { peer_id, reconnect } => {
                            if let Some(peer) = peers.get_mut(&peer_id) {

                                peer.set_state(PeerState::NotConnected);

                                if let Some(after) = reconnect {
                                    raise_event_after_delay(Event::TryConnect { peer_id }, after, &event_pub);
                                }
                            } else {
                                error!("[Peers] Peer list is out-of-sync.")
                            }
                        },
                        Event::PeerStalled { peer_id, .. } => {
                            if let Some(peer) = peers.get_mut(&peer_id) {
                                peer.set_state(PeerState::Stalled);
                            } else {
                                error!("[Peers] Peer list is out-of-sync. This should never happen.");
                            }
                        },
                        Event::BytesSent { .. } => (),
                        Event::BytesBroadcasted { .. } => (),
                        Event::BytesReceived { .. } => (),
                        Event::TryConnect { peer_id } => {
                            // ^^^ You read wrong...it's *Try*, not Bit!!! Now feel ashamed of yourself!
                            if let Some(mut peer) = peers.get_mut(&peer_id) {
                                if peer.is_connected() {
                                    continue;
                                }

                                match peer.url() {
                                    Url::Tcp(peer_addr) => {
                                        if let Some(conn) = tcp::try_connect(&peer.id(), peer_addr).await {

                                            if tcp_conns.contains_key(&peer_id) {
                                                drop(conn);
                                                continue;
                                            }

                                            let (sender, receiver) = conns::channel();

                                            tcp_conns.insert(peer.id(), sender);

                                            spawn(tcp::conn_actor(conn, receiver, event_pub.clone()));

                                            event_pub.send(Event::PeerConnected { peer_id: peer.id() }).await
                                                .expect("[Peers] Error sending 'PeerConnected' event");

                                        } else {
                                            debug!("[Peers] Connection attempt failed. Retrying in {} ms", RECONNECT_COOLDOWN);

                                            raise_event_after_delay(Event::TryConnect { peer_id }, RECONNECT_COOLDOWN, &event_pub);

                                        }
                                    },
                                    Url::Udp(peer_addr) => {
                                        // TODO
                                    }
                                }
                            }
                        }
                    }
                }
            }
            // === handle tcp events ===
            tcp_event = event_sub2.next().fuse() => {
                if let Some(tcp_event) = tcp_event {
                    debug!("[Peers] {:?}", tcp_event);

                    match tcp_event {
                        Event::PeerAccepted { peer_id, protocol, sender } => {
                            match protocol {
                                Protocol::Tcp => {
                                    tcp_conns.insert(peer_id, sender);
                                },
                                _ =>  {
                                    udp_conns.insert(peer_id, sender);
                                }
                            }
                            event_pub.send(Event::PeerConnected { peer_id }).await
                                .expect("[Peers] Error sending 'PeerConnected' event");
                        }
                        _ => (),
                    }
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
