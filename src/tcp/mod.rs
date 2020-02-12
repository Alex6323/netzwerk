use crate::connections::{Connection, MAX_BUFFER_SIZE, NetIO};
use crate::events::{Event, EventProducer};
use crate::peers::PeerId;
use crate::result;

use async_std::net::{SocketAddr, TcpListener, TcpStream};
use async_std::prelude::*;
use async_trait::async_trait;
use bytes::{Bytes, BytesMut};
use log::*;

pub async fn init(binding_addr: SocketAddr, event_prod: EventProducer) {
    let listener = TcpListener::bind(binding_addr).await.expect("error binding TCP listener");
    debug!("Successfully bound TCP listener to <<{}>>",
        listener.local_addr().expect("error reading local address from UDP socket"));

    debug!("Starting tcp processor");
    let mut incoming = listener.incoming();

    // This loop exits if all `TcpStream`s are dropped.
    while let Some(stream) = incoming.next().await {
        let stream = stream.expect("error unwrapping TCP stream");
        debug!("Successfully connected peer");

        let peer_id = PeerId(stream.peer_addr().expect("error unwrapping remote address from TCP stream"));

        event_prod.send(Event::AddTcpConnection { peer_id, stream }).expect("error sending NewTcpConnection event");
    }
    debug!("Exited tcp processor");
}

pub struct Tcp {
    stream: TcpStream,
}

impl Tcp {
    fn new(stream: TcpStream) -> Self {
        Self { stream }
    }
}

#[async_trait]
impl NetIO for Tcp {
    async fn send(&mut self, bytes: Bytes) -> result::Result<()> {
        // TODO: error propagation
        let _n = self.stream.write(&bytes)
            .await
            .expect("error sending bytes using TCP");

        Ok(())
    }
    async fn recv(&mut self) -> result::Result<Bytes> {
        let mut buffer = BytesMut::with_capacity(MAX_BUFFER_SIZE);
        // TODO: error propagation
        self.stream.read(&mut buffer)
            .await
            .expect("error receiving bytes using TCP");

        Ok(Bytes::from(buffer))
    }
}

impl Connection<Tcp> {
    pub fn new(stream: TcpStream) -> Self {
        Self(Tcp::new(stream))
    }
    pub async fn send(&mut self, bytes: Bytes) -> result::Result<()> {
        Ok(self.0.send(bytes).await?)
    }
    pub async fn recv(&mut self) -> result::Result<Bytes> {
        Ok(self.0.recv().await?)
    }
}

pub type TcpConnection = Connection<Tcp>;