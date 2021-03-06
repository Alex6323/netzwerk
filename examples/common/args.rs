use netzwerk::{
    Address,
    Config,
    Url,
};

use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "example node", about = "Proof-of-Concept for the 'netzwerk' crate")]
pub struct Args {
    #[structopt(long)]
    pub bind: String,

    #[structopt(long)]
    pub peers: Vec<String>,

    #[structopt(long)]
    pub msg: String,
}

impl Args {
    pub fn make_config(&self) -> Config {
        let mut peers = vec![];
        for peer in &self.peers {
            peers.push(Url::from_str(&peer));
        }

        Config {
            binding_addr: Address::new(self.bind.clone()),
            peers,
        }
    }
}