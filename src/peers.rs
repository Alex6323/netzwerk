use crate::address::Url;
use crate::conns::Protocol;

use async_std::net::SocketAddr;

use std::collections::HashMap;

pub mod actor {
    use super::{PeerId, Peers, PeerState, Url};

    use crate::events::{Event, EventSink, EventSource};
    use crate::commands::{Command, CommandReceiver};

    use async_std::net::TcpStream;
    use async_std::sync::Arc;
    use async_std::task;

    use log::*;
    use crossbeam_channel::select;

    /// Starts the peers actor.
    pub async fn run(command_rx: CommandReceiver, event_src: EventSource, event_snk: EventSink) {
        debug!("Start listening to peer changes");

        let mut peers = Peers::new();

        loop {
            select! {
                // handle commands
                recv(command_rx) -> command => {
                    let command = command.expect("error receiving command");
                    debug!("New peer command received: {:?}", command);

                    match command {
                        Command::AddPeer { peer } => {
                            peers.add(peer.clone());

                            event_src.send(Event::PeerAdded { peer: peer.clone() }.into())
                                .expect("error sending `PeerAdded` event");
                        },
                        Command::RemovePeer { peer_id } => {
                            peers.remove(&peer_id);

                            event_src.send(Event::PeerRemoved { peer_id }.into())
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
                recv(event_snk) -> event => {
                    let event = event.expect("error receiving event");
                    debug!("New peer event received: {:?}", event);

                    match event {
                        Event::PeerAdded { peer } => {

                            if peer.is_not_connected() {
                                match peer.url() {
                                    // === TCP ===
                                    Url::Tcp(peer_addr) => {
                                        // NOTE: we are now actively trying to connect to that peer
                                        if let Some(peer) = peers.get_mut(&peer.id()) {
                                            peer.set_state(PeerState::Connecting);
                                        }

                                        let stream = task::block_on(TcpStream::connect(peer_addr))
                                            .expect("error connecting to peer");

                                        /*
                                        let stream = TcpStream::connect(peer_addr)
                                            .await
                                            .expect("error connecting to peer");
                                        */

                                        if let Some(peer) = peers.get_mut(&peer.id()) {
                                            peer.set_state(PeerState::Connected);
                                        }

                                        let peer_id = PeerId(stream.peer_addr()
                                            .expect("error unwrapping remote address from TCP stream"));

                                        // TODO: can this happen? when? What should we do then?
                                        if peer.id() != peer_id {
                                            warn!("Something fishy is going on. Non matching peer ids.");
                                        }

                                        event_src.send(Event::PeerConnectedViaTCP { peer_id, stream: Arc::new(stream) }.into())
                                            .expect("error sending NewTcpConnection event");
                                    },
                                    // === UDP ===
                                    Url::Udp(peer_addr) => {
                                        // TODO
                                    }
                                }
                            }
                        }
                        //TODO: reconnect if peer is still added in the list, but the connection dropped
                        //EventType::PeerCheck
                        _ => (),
                    }
                }
            }
        }

        debug!("Peer listener stops listening");
    }
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

    pub fn is_connecting(&self) -> bool {
        self.state.connecting()
    }

    pub fn is_idle(&self) -> bool {
        self.state.idle()
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
}

/*
impl ops::Deref for Peers {
    type Target = HashMap<PeerId, Peer>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
*/

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
        use super::*;

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