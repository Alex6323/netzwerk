use crate::result;

use std::ops;

pub trait Message {
    fn size_range() -> ops::Range<usize>
    where
        Self: Sized;

    fn from_bytes(bytes: &[u8]) -> result::MessageResult<Self>
    where
        Self: Sized;

    fn as_bytes(&self) -> Vec<u8>;
}