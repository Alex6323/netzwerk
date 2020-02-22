use err_derive::Error;

#[derive(Debug, Error)]
pub enum MessageError {
    #[error(display = "Unknown message with type {}", t)]
    UnknownMessageType {
        t: u8
    },
    #[error(display = "Invalid header length")]
    InvalidHeaderLength(usize),
    #[error(display = "Advertised message length was {}, yet the actual length was {}", advertised, actual)]
    InvalidAdvertisedMessageLength {
        advertised: usize,
        actual: usize,
    },
    #[error(display = "Invalid message length: {}", actual)]
    InvalidMessageLength {
        actual: usize
    },
    #[error(display = "Error sending message using UDP")]
    MessageSend(#[source] std::io::Error),
    #[error(display = "Attempted to send a message to an unknown peer")]
    AttemptedSendingToUnknownPeer,
    #[error(display = "Attempted to receive a message from an unknown peer")]
    AttemptedReceivingFromUnknownPeer,
}

#[derive(Debug, Error)]
pub enum PeerError {
    #[error(display = "Binding to socket failed")]
    SocketBindingFailed(#[source] std::io::Error),
}

#[derive(Debug, Error)]
pub enum Error {
    // TODO
}

#[derive(Debug, Error)]
pub enum SendError {

    #[error(display = "IO Error")]
    IoError(#[source] std::io::Error),

    #[error(display = "Error occurred during sending bytes")]
    SendBytesError,

    #[error(display = "Tried sending to unknown peer")]
    UnknownPeerError,
}

#[derive(Debug, Error)]
pub enum RecvError {

    #[error(display = "IO Error")]
    IoError(#[source] std::io::Error),

    #[error(display = "Error occurred during receiving bytes")]
    RecvBytesError,

    #[error(display = "Tried sending to unknown peer")]
    UnknownPeerError,
}
