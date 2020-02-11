use crate::connections::{Connection, MAX_BUFFER_SIZE, NetIO};
use crate::result;

use async_std::net::TcpStream;
use async_std::prelude::*;
use async_trait::async_trait;
use bytes::{Bytes, BytesMut};

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