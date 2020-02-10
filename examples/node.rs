// Run this with `cargo r --example node`

use netzwerk::api;
use netzwerk::{Address, Peer, Peers, Connections, Protocol, Tcp, Udp, TcpBroker};

use config::Config;

use async_std::task;
use crossbeam_channel as mpmc;

mod config;

struct NetzwerkNode {
    peers: Peers,
    tcp: TcpBroker,
}

impl NetzwerkNode {
    pub async fn from_config(config: Config) -> Self {
        let mut peers = Peers::new();
        peers.add(Peer::from_address(Address::new("127.0.0.1:1337", Protocol::Tcp).await));
        peers.add(Peer::from_address(Address::new("127.0.0.1:1338", Protocol::Tcp).await));

        Self {
            peers,
            tcp: TcpBroker::new(),
        }
    }

    pub fn broadcast(&mut self) {

    }
}

fn main() {
    api::init();

    let config = Config::builder().build();
    task::block_on(async {
        let mut node = NetzwerkNode::from_config(config).await;
        println!("created node");
    });

}