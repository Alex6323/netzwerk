use async_std::net::SocketAddr;

pub enum Address {
    Udp(SocketAddr),
    Tcp(SocketAddr),
    // Note: this list might get extended in the future
}