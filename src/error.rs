use err_derive::Error;

use std::io;

#[derive(Debug, Error)]
pub enum Error {
    #[error(display = "Binding to socket failed")]
    SocketBindingFailed(#[source] io::Error),
    #[error(display = "Attempted to send a message to an unknown peer")]
    AttemptedSendingToUnknownPeer,
    #[error(display = "Attempted to receive a message from an unknown peer")]
    AttemptedReceivingFromUnknownPeer,
    #[error(display = "Message could not be serialized into bytes")]
    MessageSerializationError,
    #[error(display = "Message could not be deserialized into bytes")]
    MessageDeserializationError,
}
