use async_std::net::{Ipv4Addr, Ipv6Addr, SocketAddr, SocketAddrV4, SocketAddrV6, ToSocketAddrs};

use std::collections::HashMap;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct PeerId(SocketAddr);

pub struct Peer {
    id: PeerId,
    address: SocketAddr,
}

impl Peer {
    pub async fn from_address(address: impl ToSocketAddrs) -> Self {
        // TODO: proper error handling
        let address = address.to_socket_addrs().await.unwrap().nth(0).unwrap();
        Self { id: PeerId(address), address }
    }
    pub fn from_ipv4_address(ipv4_address: Ipv4Addr) -> Self {
        // TODO: port
        let address = SocketAddr::V4(SocketAddrV4::new(ipv4_address, 0));
        Self { id: PeerId(address), address }
    }
    pub fn from_ipv6_address(ipv6_address: Ipv6Addr) -> Self {
        // TODO: port
        let address = SocketAddr::V6(SocketAddrV6::new(ipv6_address, 0, 0, 0));
        Self { id: PeerId(address), address }
    }
    pub fn id(&self) -> PeerId {
        self.id
    }
    pub fn address(&self) -> &SocketAddr {
        &self.address
    }
}

pub struct Peers(HashMap<PeerId, Peer>);

impl Peers {
    pub fn num(&self) -> usize {
        self.0.len()
    }

    pub fn add(&mut self, peer: Peer) {
        if !self.0.contains_key(&peer.id()) {
            self.0.insert(peer.id(), peer);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod peer {
        use super::Peer;

        #[test]
        fn create_new_peer() {
            //let peer = Peer::new(SocketAddr::"127.0.0.1:1337".into());
            //Ipv4Addr::LOCALHO
        }
    }
}