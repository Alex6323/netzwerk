use super::args::Args;

use netzwerk::{Address, Peer, Peers, Protocol, Url};
use netzwerk::util;

use async_std::net::ToSocketAddrs;
use structopt::StructOpt;

pub struct NodeConfig {
    pub id: String,
    pub binding_addr: Address,
    peers: Vec<Url>,
}

impl NodeConfig {
    pub fn from_args() -> Self {
        let args = Args::from_args();
        let mut peers = vec![];
        for peer in &args.peers {
            peers.push(Url::from_url_str(&peer));
        }

        Self {
            id: args.id,
            binding_addr: Address::new(args.bind),
            peers,
        }
    }

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

    pub fn binding_address(&self) -> Address {
        //Address::
        unimplemented!()
    }
}

pub struct NodeConfigBuilder {
    id: Option<String>,
    binding_addr: Option<Address>,
    peers: Vec<Url>,
}

impl NodeConfigBuilder {
    pub fn new() -> Self {
        Self {
            id: None,
            binding_addr: None,
            peers: vec![],
        }
    }

    pub fn with_identifier(mut self, id: &str) -> Self {
        self.id.replace(id.into());
        self
    }

    pub fn with_binding(mut self, binding_addr: impl ToSocketAddrs) -> Self {
        let binding_addr = util::to_single_socket_address(binding_addr);
        self.binding_addr.replace(Address::Socket(binding_addr));
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
            binding_addr: self.binding_addr.unwrap_or(Address::new("localhost:1337")),
            peers: self.peers,
        }
    }
}