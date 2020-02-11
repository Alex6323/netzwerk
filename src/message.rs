use crate::result;

use bytes::Bytes;

pub trait Message {
    fn from_bytes(bytes: Bytes) -> result::Result<Self> where Self: Sized;
    fn into_bytes(&self) -> Bytes;
}