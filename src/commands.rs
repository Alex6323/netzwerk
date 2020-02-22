use crate::peers::{Peer, PeerId};

use std::fmt;

use async_std::task;
use futures::channel::mpsc;
use futures::sink::SinkExt;

const COMMAND_CHAN_CAPACITY: usize = 10;

/// `Command`s can be used to control the networking layer from higher layers.
#[derive(Clone)]
pub enum Command {

    /// Adds a peer to the system.
    AddPeer {
        peer: Peer
    },

    /// Removes a peer from the system.
    RemovePeer {
        peer_id: PeerId
    },

    /// Sends bytes to a connected peer.
    SendBytes {
        receiver: PeerId,
        bytes: Vec<u8>,
    },

    /// Sends bytes to all connected peers.
    BroadcastBytes {
        bytes: Vec<u8>,
    },

    /// Shuts down the system.
    Shutdown,
}

impl fmt::Debug for Command {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Command::AddPeer { peer } => write!(f, "AddPeer command <<{:?}>>", peer.id()),
            Command::RemovePeer { peer_id } => write!(f, "RemovePeer command <<{:?}>>", peer_id),
            Command::SendBytes { receiver, .. } => write!(f, "SendBytes command <<{:?}>>", receiver),
            Command::BroadcastBytes { .. } => write!(f, "BroadcastBytes command"),
            Command::Shutdown => write!(f, "Shutdown command"),
        }
    }
}

pub(crate) type CommandSender = mpsc::Sender<Command>;
pub type CommandReceiver = mpsc::Receiver<Command>;

pub struct CommandDispatcher {
    senders: Vec<CommandSender>,
}

impl CommandDispatcher {

    pub fn new() -> Self {
        Self { senders: vec![] }
    }

    pub fn recv(&mut self) -> CommandReceiver {
        let (sender, receiver) = mpsc::channel(1);

        self.senders.push(sender);

        receiver
    }

    pub async fn dispatch(&mut self, command: Command) {
        // TODO: those commands can be dispatched concurrently
        for sender in &mut self.senders {
            sender.send(command.clone()).await.expect("error sending command");
        }
    }
}