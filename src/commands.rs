use crate::peers::{Peer, PeerId};

use std::fmt;

use futures::channel::mpsc;
use futures::sink::SinkExt;
use futures::prelude::*;
use log::*;

use std::collections::HashMap;

// NOTE: For now, we really want commands to be executed with high backpressure
const COMMAND_CHAN_CAPACITY: usize = 1;

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
        to: PeerId,
        bytes: Vec<u8>,
    },

    /// Sends bytes to all connected peers.
    BroadcastBytes {
        bytes: Vec<u8>,
    },

    /// Shuts down the system.
    Shutdown,

    /*
    UpdatePeer {
        peer_id: PeerId,
        f: FnMut(&mut Peer),
    },

    RequestInfo {
        f: Fn(&Peer) -> Response,
        response: futures::channel::oneshot::Sender<Response>,
    }
    */
}

struct Response {
    //TODO
}

impl fmt::Debug for Command {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Command::AddPeer { peer } =>
                write!(f, "Command::AddPeer {{ peer_id = {:?} }} ", peer.id()),

            Command::RemovePeer { peer_id } =>
                write!(f, "Command::RemovePeer {{ peer_id = {:?} }}", peer_id),

            Command::SendBytes { to, .. } =>
                write!(f, "Command::SendBytes {{ to = {:?} }}", to),

            Command::BroadcastBytes { .. } =>
                write!(f, "Command::BroadcastBytes"),

            Command::Shutdown =>
                write!(f, "Command::Shutdown"),
        }
    }
}

pub(crate) type CommandSender = mpsc::Sender<Command>;
pub(crate) type CommandReceiver = mpsc::Receiver<Command>;

pub(crate) fn channel() -> (CommandSender, CommandReceiver) {
    mpsc::channel(COMMAND_CHAN_CAPACITY)
}

pub struct CommandDispatcher {
    senders: HashMap<String, CommandSender>,
}

impl CommandDispatcher {

    pub fn new() -> Self {
        Self { senders: HashMap::new() }
    }

    pub fn register(&mut self, address: &str) -> CommandReceiver {
        let (sender, receiver) = channel();

        self.senders.insert(address.to_string(), sender);

        receiver
    }

    pub fn send<'a>(&'a mut self, command: Command) -> CommandSend<'a> {
        CommandSend {
            senders: &mut self.senders,
            command,
        }
    }
}

// TEMP
type CommandSendResult = std::result::Result<(), Box<dyn std::error::Error>>;

/// Makes using the `CommandDispatcher` more convenient.
/// Examples:
/// ```
/// use Command::*;
/// use Actor::*;
/// command_dp.send(Shutdown).to(All);
/// command_dp.send(AddPeer { peer }).to(One("peers"));
/// command_dp.send(RemovePeer { peer_id }).to(Many("peers", "conns"));
/// ```
pub struct CommandSend<'a> {
    senders: &'a mut HashMap<String, CommandSender>,
    command: Command,
}

impl<'a> CommandSend<'a> {
    pub async fn to(self, actor: Actor<'a>) {
        match actor {
            Actor::One(id) => {
                if !self.senders.contains_key(&id[..]) {
                    return;
                }
                let sender = self.senders.get_mut(&id[..]).unwrap();
                sender.send(self.command).await.expect("[Cmnds] Error sending command to one actor");
            }
            Actor::Many(ids) => {
                // TODO: those commands can be dispatched concurrently
                for id in ids {
                    if self.senders.contains_key(&id[..]) {
                        let sender = self.senders.get_mut(&id[..]).unwrap();
                        sender.send(self.command.clone()).await.expect("[Cmnds] Error sending command to many actors");
                    }
                }
            }
            Actor::All => {
                // TODO: those commands can be dispatched concurrently
                for (_, sender) in self.senders {
                    sender.send(self.command.clone()).await.expect("[Cmnds] Error sending command to all actors");
                }
            },
        }
    }
}

pub(crate) const TCP: &'static str = "tcp";
pub(crate) const UDP: &'static str = "udp";
pub(crate) const PEERS: &'static str = "peers";

pub enum Actor<'a> {
    One(&'a str),
    Many(Vec<&'a str>),
    All,
}

/// Starts the `commands` actor. Its purpose is to receive `Command`s from the user
/// and dispatch them using internal knowledge about which actor needs to respond to
/// which command. If this is not done this way, then either all commands would need
/// to be broadcasted to all actors, which is an overhead we cannot accept, or we
/// would need to leak internal details as the user would need to know which actors
/// exist in the system.
pub async fn actor(mut command_dp: CommandDispatcher, mut command_rx: CommandReceiver) {
    use Actor::*;
    use Command::*;
    debug!("[Cmnds] Starting actor");

    while let Some(command) = command_rx.next().await {
        debug!("[Cmnds] {:?}", command);

        let actors = match command {
            AddPeer { .. } => {
                One("peers")
            },
            RemovePeer { .. } => {
                One("peers")
            },
            SendBytes { .. } => {
                One("peers")
            },
            BroadcastBytes { .. } => {
                One("peers")
            }
            Shutdown => {
                command_dp.send(Shutdown).to(All).await;
                break;
            }
        };

        command_dp.send(command).to(actors).await;
    }

    debug!("[Cmnds] Stopping actor");
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_std::task;

    fn create_command_dispatcher() {
        let mut command_dp = CommandDispatcher::new();

        let actor_a_rx = command_dp.register("actor_a");
        let actor_b_rx = command_dp.register("actor_b");

        task::block_on(async {
            command_dp.send(Command::Shutdown).to(Actor::All).await;
        });
    }
}