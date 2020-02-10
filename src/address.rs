use crate::connection::Protocol;
use crate::peer::PeerId;

use async_std::net::{Ipv4Addr, Ipv6Addr, SocketAddr, SocketAddrV4, SocketAddrV6, ToSocketAddrs};

#[derive(Clone, Copy, Debug)]
pub enum Address {
    /// Represents a TCP address.
    Tcp(SocketAddr),

    /// Represents a UDP address.
    Udp(SocketAddr),

    // Note: this list might get extended in the future
}

impl Address {
    pub async fn from_address(address: impl ToSocketAddrs, p: Protocol) -> Self {
        // TODO: proper error handling
        let address = address.to_socket_addrs().await.unwrap().nth(0).unwrap();

        match p {
            Protocol::Tcp => Address::Tcp(address),
            Protocol::Udp => Address::Udp(address),
        }
    }

    pub fn from_ipv4_address(ipv4_address: Ipv4Addr, port: Option<u16>, p: Protocol) -> Self {
        let port = if let Some(port) = port {
            port
        } else {
            0
        };

        let address = SocketAddr::V4(SocketAddrV4::new(ipv4_address, port));

        match p {
            Protocol::Tcp => Address::Tcp(address),
            Protocol::Udp => Address::Udp(address),
        }
    }

    pub fn from_ipv6_address(ipv6_address: Ipv6Addr, port: Option<u16>, p: Protocol) -> Self {
        let port = if let Some(port) = port {
            port
        } else {
            0
        };

        let address = SocketAddr::V6(SocketAddrV6::new(ipv6_address, port, 0, 0));

        match p {
            Protocol::Tcp => Address::Tcp(address),
            Protocol::Udp => Address::Udp(address),
        }
    }

    pub fn protocol(&self) -> Protocol {
        match *self {
            Address::Tcp(_) => Protocol::Tcp,
            Address::Udp(_) => Protocol::Udp,
        }
    }
}
