//use super::args::Args;

use crate::address::{Address, Url};
use crate::peers::{Peer, Peers};
use crate::util;

use async_std::net::ToSocketAddrs;
//use structopt::StructOpt;

#[derive(Clone)]
pub struct Config {
    pub id: String,
    pub binding_addr: Address,
    pub peers: Vec<Url>,
}

impl Config {

    pub fn builder() -> ConfigBuilder {
        ConfigBuilder::new()
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

pub struct ConfigBuilder {
    id: Option<String>,
    binding_addr: Option<Address>,
    peers: Vec<Url>,
}

impl ConfigBuilder {
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
        self.binding_addr.replace(Address::Ip(binding_addr));
        self
    }

    pub fn with_peer(mut self, peer_url: &str) -> Self {
        let url = Url::from_url_str(peer_url);
        self.peers.push(url);
        self
    }

    pub fn build(self) -> Config {
        Config {
            id: self.id.unwrap_or(Address::new("localhost:1337").port().unwrap().to_string()),
            binding_addr: self.binding_addr.unwrap_or(Address::new("localhost:1337")),
            peers: self.peers,
        }
    }
}