use netzwerk::Message;
use netzwerk::error;
use netzwerk::result;

use bytes::Bytes;

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
    fn from_bytes(bytes: Bytes) -> result::Result<Self> {
        Ok(Self {
            data: String::from_utf8(bytes.to_vec()).unwrap(),
        })
    }

    fn into_bytes(&self) -> Bytes {
        Bytes::from(Vec::from(self.data.as_bytes()))
    }
}
