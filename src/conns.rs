use crate::errors::{SendError, RecvError};
use crate::events::Event;
use crate::peers::PeerId;

use async_trait::async_trait;
use futures::channel::mpsc;

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

pub struct Connections<C: RawConnection>(pub(crate) HashMap<PeerId, C>);

impl<C: RawConnection> Connections<C> {
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    pub fn num(&self) -> usize {
        self.0.len()
    }

    pub fn insert(&mut self, peer_id: PeerId, conn: C) {
        self.0.insert(peer_id, conn);
    }

    pub fn remove(&mut self, peer_id: &PeerId) -> bool {
        self.0.remove(peer_id).is_some()
    }

    pub fn get(&self, peer_id: &PeerId) -> Option<&C> {
        self.0.get(peer_id)
    }

    pub fn get_mut(&mut self, peer_id: &PeerId) -> Option<&mut C> {
        self.0.get_mut(peer_id)
    }

    pub async fn broadcast(&mut self, bytes: Vec<u8>) -> SendResult<()> {
        for (_, peer_conn) in &mut self.0 {
            peer_conn.send(bytes.clone()).await.expect("error broadcasting to peer");
        }
        Ok(())
    }

    pub async fn send(&mut self, bytes: Vec<u8>, to_peer: &PeerId) -> SendResult<Event> {
        let peer_conn = if !self.0.contains_key(to_peer) {
            return Err(SendError::UnknownPeer);
        } else {
            self.0.get_mut(to_peer).unwrap()
        };

        Ok(peer_conn.send(bytes).await?)
    }

    pub async fn recv(&mut self, from_peer: &PeerId) -> RecvResult<Event> {
        let peer_conn = if !self.0.contains_key(from_peer) {
            return Err(RecvError::UnknownPeer);
        } else {
            self.0.get_mut(from_peer).unwrap()
        };

        Ok(peer_conn.recv().await?)
    }
}

pub type ByteSender = mpsc::Sender<Vec<u8>>;
pub type ByteReceiver = mpsc::Receiver<Vec<u8>>;

pub fn channel() -> (ByteSender, ByteReceiver) {
    mpsc::channel(WIRE_CHAN_CAPACITY)
}