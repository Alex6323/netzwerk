use crate::address::Url;
use crate::conns::Protocol;
use crate::events::{EventSource, EventType};
use crate::commands::{CommandType, CommandReceiver};

use async_std::net::SocketAddr;
use log::*;

use std::collections::HashMap;
use std::ops;

///
pub async fn listen(command_rx: CommandReceiver, event_source: EventSource) {
    debug!("Start listening to peer changes");

    let mut peers = Peers::new();

    while let Ok(command) = command_rx.recv() {
        debug!("New peer command received: {:?}", command);

        match &*command {
            CommandType::AddPeer { peer } => {
                peers.add(peer.clone());
                event_source.send(EventType::PeerAdded { peer: peer.clone() }.into());

                // start (re)connecting task for that new peer

                /*
                peer.set_state(PeerState::Connecting);
                match peer.protocol() {
                    Protocol::Tcp => {
                        if let Url::Tcp(peer_addr) = peer.url {
                            let stream = TcpStream::connect(peer_addr)
                                .await
                                .expect(&format!("error connecting to peer <<{:?}>>", peer_addr));

                                let peer_id = PeerId(stream.peer_addr()
                                    .expect("error unwrapping remote address from TCP stream"));

                                event_prod.send(Event::AddTcpConnection { peer_id, stream }).expect("error sending NewTcpConnection event");

                        }

                    },
                    Protocol::Udp => {
                        // TODO
                    }
                }
                */
            },
            CommandType::RemovePeer { peer_id } => {
                peers.remove(&peer_id);

                event_source.send(EventType::PeerRemoved { peer_id: peer_id.clone() }.into());
            },
            CommandType::Shutdown => {
                drop(peers);
                event_source.send(EventType::Shutdown {}.into());
                break
            }
            _ => (),
        }
    }

    debug!("Peer listener stops listening");
}

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

    pub fn address(&self) -> &Url {
        &self.url
    }

    pub fn protocol(&self) -> Protocol {
        self.url.protocol()
    }

    pub fn is_connected(&self) -> bool {
        self.state.connected()
    }

    pub fn is_connecting(&self) -> bool {
        self.state.connecting()
    }

    pub fn is_idle(&self) -> bool {
        self.state.idle()
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

    pub fn remove(&mut self, id: &PeerId) {
        self.0.remove(id);
    }
}

impl ops::Deref for Peers {
    type Target = HashMap<PeerId, Peer>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PeerState {
    /// We are not connected to that peer.
    NotConnected,
    /// We are trying to connect to that peer.
    Connecting,
    /// We are connected to that peer.
    Connected,
    /// We are connected to that peer, but it is not sending any messages anylonger.
    Idle,
}

impl PeerState {
    pub fn connected(&self) -> bool {
        *self == PeerState::Connected
    }
    pub fn connecting(&self) -> bool {
        *self == PeerState::Connecting
    }
    pub fn idle(&self) -> bool {
        *self == PeerState::Idle
    }
}

impl Default for PeerState {
    fn default() -> Self {
        PeerState::NotConnected
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod peer_id {
        use super::PeerId;

        #[test]
        fn create_peer_id() {
            //let id = PeerId();
        }
    }

    mod peer {
        use super::Peer;
        use async_std::task;

        #[test]
        fn create_new_peer() {
            task::block_on(async {
                let peer = Peer::from_url("tcp://localhost:1337").await;
                assert_eq!(peer.address().to_string(), "127.0.0.1:1337");
            })
        }

        #[test]
        fn new_peer_defaults_to_not_connected() {
            task::block_on(async {
                let peer = Peer::from_address("127.0.0.1:1337").await;
                assert!(!peer.connected(), "Peer should initially default to 'NotConnected'");
            })
        }
    }

    mod peers {
        use super::{Peer, Peers};
        use async_std::net::Ipv4Addr;

        #[test]
        fn create_new_peers() {
            let peers = Peers::new();

            assert_eq!(0, peers.num());
        }

        #[test]
        fn add_peers() {
            let mut peers = Peers::new();

            peers.add(Peer::from_ipv4_address(Ipv4Addr::new(127, 0, 0, 1), Some(1337)));
            peers.add(Peer::from_ipv4_address(Ipv4Addr::new(127, 0, 0, 1), Some(1338)));

            assert_eq!(2, peers.num());
        }

    }
}