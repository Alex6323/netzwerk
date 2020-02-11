use netzwerk::{Address, Peer, Peers, Protocol, Url};
use netzwerk::util;

use async_std::net::ToSocketAddrs;

pub struct NodeConfig {
    pub id: String,
    pub host: Address,
    peers: Vec<Url>,
}

impl NodeConfig {
    pub fn builder() -> NodeConfigBuilder {
        NodeConfigBuilder::new()
    }

    pub fn peers(&self) -> Peers {
        let mut peers = Peers::new();
        for url in &self.peers {
            peers.add(Peer::from_url(*url));
        }
        peers
    }
}

pub struct NodeConfigBuilder {
    id: Option<String>,
    host: Option<Address>,
    peers: Vec<Url>,
}

impl NodeConfigBuilder {
    pub fn new() -> Self {
        Self {
            id: None,
            host: None,
            peers: vec![],
        }
    }

    pub fn with_identifier(mut self, id: &str) -> Self {
        self.id.replace(id.into());
        self
    }

    pub fn with_host_binding(mut self, host_addr: impl ToSocketAddrs) -> Self {
        let host = util::to_single_socket_address(host_addr);
        self.host.replace(Address::Socket(host));
        self
    }

    pub fn with_peer(mut self, peer_url: &str) -> Self {
        let url = Url::from_url_str(peer_url);
        self.peers.push(url);
        self
    }

    pub fn build(self) -> NodeConfig {
        NodeConfig {
            id: self.id.unwrap_or(Address::new("localhost:1337").port().to_string()),
            host: self.host.unwrap_or(Address::new("localhost:1337")),
            peers: self.peers,
        }
    }
}