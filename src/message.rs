use bytes::Bytes;

pub enum Message {
    Transaction(Bytes),
}