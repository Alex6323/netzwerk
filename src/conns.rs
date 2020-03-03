use crate::errors::{SendError, RecvError};
use crate::events::Event;
use crate::peers::PeerId;

use async_trait::async_trait;
use futures::channel::mpsc;
use futures::sink::SinkExt;

use std::collections::HashMap;
use std::result;

pub(crate) const MAX_BUFFER_SIZE: usize = 1604;
pub(crate) const RECONNECT_COOLDOWN: u64 = 5000;

pub type SendResult<T> = result::Result<T, SendError>;
pub type RecvResult<T> = result::Result<T, RecvError>;

const WIRE_CHAN_CAPACITY: usize = 10000;

// NOTE: using this attribute implies heap allocation.
#[async_trait]
pub trait RawConnection {

    fn peer_id(&self) -> PeerId;

    async fn send(&mut self, bytes: Vec<u8>) -> SendResult<Event>;

    async fn recv(&mut self) -> RecvResult<Event>;
}

pub struct Conns(pub(crate) HashMap<PeerId, BytesSender>);

impl Conns {
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    pub fn num(&self) -> usize {
        self.0.len()
    }

    pub fn add(&mut self, peer_id: PeerId, sender: BytesSender) {
        self.0.insert(peer_id, sender);
    }

    pub fn remove(&mut self, peer_id: &PeerId) -> bool {
        self.0.remove(peer_id).is_some()
    }

    pub fn get(&mut self, peer_id: &PeerId) -> Option<&mut BytesSender> {
        self.0.get_mut(peer_id)
    }

    pub async fn broadcast(&mut self, bytes: Vec<u8>) {
        for (_, sender) in self.0.iter_mut() {
            // TODO: do not clone (use Arc)
            sender.send(bytes.clone()).await;
        }
    }

    pub async fn send(&mut self, bytes: Vec<u8>, to_peer: &PeerId) {
        if let Some(sender) = self.get(&to_peer) {
            sender.send(bytes).await;
        }
    }
}

pub type BytesSender = mpsc::Sender<Vec<u8>>;
pub type BytesReceiver = mpsc::Receiver<Vec<u8>>;

pub fn channel() -> (BytesSender, BytesReceiver) {
    mpsc::channel(WIRE_CHAN_CAPACITY)
}