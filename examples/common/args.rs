use netzwerk::{
    Address,
    Config,
    Url,
};

use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "example node", about = "an example node")]
pub struct Args {
    #[structopt(long)]
    pub id: String,

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
            peers.push(Url::from_url_str(&peer));
        }

        Config {
            id: self.id.clone(),
            binding_addr: Address::new(self.bind.clone()),
            peers,
        }
    }
}
/*
impl From<Args> for Config {
    fn from(args: Args) -> Self {
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
}
*/