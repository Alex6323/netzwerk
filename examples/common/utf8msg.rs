pub struct Utf8Message {
    data: String,
}

impl Utf8Message {
    pub fn new(s: &str) -> Self {
        Self {
            data: s.into(),
        }
    }

    pub fn from_bytes(bytes: &[u8]) -> Self {
        Self {
            data: String::from_utf8(bytes.to_vec()).unwrap(),
        }
    }

    pub fn as_bytes(&self) -> Vec<u8> {
        Vec::from(self.data.as_bytes())
    }
}