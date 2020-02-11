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
}