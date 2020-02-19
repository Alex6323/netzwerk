use crate::peers::{Peer, PeerId};

use std::{fmt, ops};

use async_std::sync::Arc;
use async_std::task::JoinHandle;
use crossbeam_channel as mpmc;

const COMMAND_CHAN_CAPACITY: usize = 10;

#[derive(Clone, Debug)]
pub struct Command(Arc<CommandType>);

impl Command {
    pub fn new(t: CommandType) -> Self {
        Self(Arc::new(t))
    }
}

/// `Command`s can be used to control the networking layer from higher layers.
pub enum CommandType {

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

impl ops::Deref for Command {
    type Target = CommandType;

    fn deref(&self) -> &Self::Target {
        &*self.0
    }
}

impl From<CommandType> for Command {
    fn from(t: CommandType) -> Self {
        Self(Arc::new(t))
    }
}

impl fmt::Debug for CommandType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CommandType::AddPeer { peer } => write!(f, "AddPeer command <<{:?}>>", peer.id()),
            CommandType::RemovePeer { peer_id } => write!(f, "RemovePeer command <<{:?}>>", peer_id),
            CommandType::SendBytes { receiver, .. } => write!(f, "SendBytes command <<{:?}>>", receiver),
            CommandType::BroadcastBytes { .. } => write!(f, "Broadcast command"),
            CommandType::Shutdown => write!(f, "Shutdown command"),
        }
    }
}

pub struct Controller {
    sender: CommandSender,
    task_handles: Vec<JoinHandle<()>>,
}

impl Controller {
    pub fn new(sender: CommandSender) -> Self {
        Self {
            sender,
            task_handles: vec![],
        }
    }

    pub fn task_handles(&mut self) -> &mut Vec<JoinHandle<()>> {
        &mut self.task_handles
    }

    pub fn add_task(&mut self, handle: JoinHandle<()>) {
        self.task_handles.push(handle);
    }

    pub fn send(&self, command: CommandType) {
        self.sender.send(command.into()).expect("error sending command");
    }
}

pub(crate) type CommandSender = mpmc::Sender<Command>;
pub type CommandReceiver = mpmc::Receiver<Command>;

pub fn channel() -> (Controller, CommandReceiver) {
    let (sender, receiver) = mpmc::bounded::<Command>(COMMAND_CHAN_CAPACITY);

    (Controller::new(sender), receiver)
}
