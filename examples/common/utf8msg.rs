use netzwerk::Message;
use netzwerk::errors;
use netzwerk::result;

use std::ops;

pub struct Utf8Message {
    data: String,
}

impl Utf8Message {
    pub fn new(s: &str) -> Self {
        Self {
            data: s.into(),
        }
    }
}

impl Message for Utf8Message {
    fn size_range() -> ops::Range<usize>
    where
        Self: Sized
    {
        0..100
    }

    fn from_bytes(bytes: &[u8]) -> result::MessageResult<Self> {
        Ok(Self {
            data: String::from_utf8(bytes.to_vec()).unwrap(),
        })
    }

    fn as_bytes(&self) -> Vec<u8> {
        Vec::from(self.data.as_bytes())
    }
}
