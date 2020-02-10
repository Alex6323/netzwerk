use crate::peer::PeerId;
use crate::connection::Protocol;

pub enum Signal {
    /// A peer has been added to the peer list. Accept connection, and attempt to (re)connect.
    AddedPeer(PeerId),
    /// A peer has been removed from the peer list. Ignore connection attempts, and don't (re)connect anylonger.
    RemovedPeer(PeerId),
    /// A peer has been successfully connected.
    ConnectedPeer(PeerId, Protocol),
    /// A peer has been disconnected for some reason.
    DisconnectedPeer(PeerId),
    /// The system is about to shutdown. Disconnect from all peers.
    Termination,
}