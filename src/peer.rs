use crate::address::Address;
use crate::connection::Protocol;

use async_std::net::{Ipv4Addr, Ipv6Addr, SocketAddr, SocketAddrV4, SocketAddrV6, ToSocketAddrs};

use std::collections::HashMap;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct PeerId(SocketAddr);

impl From<Address> for PeerId {
    fn from(addr: Address) -> Self {
        match addr {
            Address::Tcp(socket_addr) => Self(socket_addr),
            Address::Udp(socket_addr) => Self(socket_addr),

        }
    }
}

pub struct Peer {
    /// The ID of the peer to recognize it across connections.
    id: PeerId,
    /// The address of the peer, e.g. a socket address.
    address: Address,
    /// The current state of the peer {NotConnected, Connected, ...}.
    state: PeerState,
}

impl Peer {
    pub fn from_address(address: Address) -> Self {
        Self { id: address.into(), address, state: Default::default() }
    }

    pub fn id(&self) -> PeerId {
        self.id
    }

    pub fn address(&self) -> &Address {
        &self.address
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

#[derive(Debug, Eq, PartialEq)]
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
                let peer = Peer::from_address("127.0.0.1:1337").await;
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