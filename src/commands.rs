use crate::peers::{Peer, PeerId};

use crossbeam_channel as mpmc;
use std::fmt;

pub enum Command {
    AddPeer(Peer),
    RemovePeer(PeerId),
    ConnectToPeer(Peer),
    DisconnectFromPeer(PeerId),
    ReconnectPeer(PeerId),
    Shutdown,
}

impl fmt::Debug for Command {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Command::AddPeer(_) => write!(f, "AddPeer"),
            Command::RemovePeer(_) => write!(f, "RemovePeer"),
            Command::ConnectToPeer(_) => write!(f, "ConnectToPeer"),
            Command::DisconnectFromPeer(_) => write!(f, "DisconnectFromPeer"),
            Command::ReconnectPeer(_) => write!(f, "ReconnectPeer"),
            Command::Shutdown => write!(f, "Shutdown"),
        }
    }
}

pub type CommandSender = mpmc::Sender<Command>;
pub type CommandReceiver = mpmc::Receiver<Command>;

pub fn create_command_channel() -> (CommandSender, CommandReceiver) {
    mpmc::unbounded::<Command>()
}