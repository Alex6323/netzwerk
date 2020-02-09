use crate::result;

use async_std::net::{TcpStream, UdpSocket, SocketAddr};
use async_trait::async_trait;
use bytes::Bytes;

const BUFFER_SIZE: usize = 1604;

#[async_trait]
pub trait RawProtocol {
    async fn send(&self, message: Bytes, address: SocketAddr) -> result::Result<()>;
    async fn recv(&mut self) -> result::Result<Bytes>;
}

pub struct Udp {
    socket: UdpSocket,
}

impl Udp {
    fn new(socket: UdpSocket) -> Self {
        // NOTE: the socket must be already bound
        Self { socket }
    }
}

#[async_trait]
impl RawProtocol for Udp {
    async fn send(&self, message: Bytes, address: SocketAddr) -> result::Result<()> {
        //(*self.socket).send_to(&message[..n], &address).await?;
        unimplemented!("UDP send")
    }
    async fn recv(&mut self) -> result::Result<Bytes> {
        let mut buf = vec![0u8; BUFFER_SIZE];
        let (n, peer) = self.socket.recv_from(&mut buf).await?;
        Ok(Bytes::from(buf))
    }
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
impl RawProtocol for Tcp {
    async fn send(&self, message: Bytes, address: SocketAddr) -> result::Result<()> {
        unimplemented!("TCP send")
    }
    async fn recv(&mut self) -> result::Result<Bytes> {
        unimplemented!("TCP recv")
    }
}


pub struct Connection<R: RawProtocol = Tcp>(R);

// default = TCP
impl Connection {
    fn new(stream: TcpStream) -> Self {
        Self(Tcp::new(stream))
    }
    fn send(&self) {}
    fn recv(&self) {}
}

impl Connection<Udp> {
    fn new(udp_socket: UdpSocket) -> Self {
        Self(Udp::new(udp_socket))
    }
    fn send(&self) {}
    fn recv(&self) {}
}


pub type UdpConnection = Connection<Udp>;
pub type TcpConnection = Connection<Tcp>;

#[cfg(test)]
mod tests {
    use super::*;

    mod tcp_conn {
        use super::{Tcp, TcpConnection};

        #[test]
        fn create_tcp_connection() {

        }
    }
}