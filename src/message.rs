use crate::result;

use bytes::Bytes;

pub trait Message: Sized + Send + 'static {
    fn from_bytes(bytes: Bytes) -> result::Result<Self>;
    fn into_bytes(&self) -> Bytes;
}