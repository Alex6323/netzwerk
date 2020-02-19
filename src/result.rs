pub type MessageResult<T> = std::result::Result<T, crate::errors::MessageError>;
pub type PeerResult<T> = std::result::Result<T, crate::errors::PeerError>;