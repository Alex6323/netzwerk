use crate::connections::Protocol;
use crate::util;

use async_std::net::{Ipv4Addr, Ipv6Addr, SocketAddr, SocketAddrV4, SocketAddrV6, ToSocketAddrs};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Address {
    Ip(SocketAddr),
}

impl Address {
    pub fn new(addr: impl ToSocketAddrs) -> Self {
        let addr = util::to_single_socket_address(addr);
        Address::Ip(addr)
    }

    pub fn port(&self) -> Option<u16> {
        if let Address::Ip(socket_addr) = *self {
            Some(socket_addr.port())
        } else {
            None
        }
    }

    pub fn socket_addr(&self) -> Option<SocketAddr> {
        if let Address::Ip(socket_addr) = *self {
            Some(socket_addr)
        } else {
            None
        }
    }

    pub fn is_ipv4(&self) -> bool {
        if let Address::Ip(socket_addr) = *self {
            socket_addr.is_ipv4()
        } else {
            false
        }
    }

    pub fn is_ipv6(&self) -> bool {
        if let Address::Ip(socket_addr) = *self {
            socket_addr.is_ipv6()
        } else {
            false
        }
    }
}

// TODO: use 'url' crate for parsing.

#[derive(Clone, Copy, Debug)]
pub enum Url {
    /// Represents a TCP url ("tcp://...").
    Tcp(SocketAddr),
    /// Represents a UDP url ("udp://...").
    Udp(SocketAddr),
}

impl Url {
    // TODO: `impl ToSocketAddrs` won't be enough for other protocols. We'll probably have to resort
    // to `&str` eventually.
    pub fn new(addr: impl ToSocketAddrs, proto: Protocol) -> Self {
        let address = util::to_single_socket_address(addr);

        match proto {
            Protocol::Tcp => Url::Tcp(address),
            Protocol::Udp => Url::Udp(address),
        }
    }

    pub fn from_url_str(url: &str) -> Self {
        let proto_addr: Vec<&str> = url.split_terminator("://").collect();
        assert_eq!(2, proto_addr.len());

        let proto: Protocol = proto_addr[0].into();
        Url::new(proto_addr[1], proto)
    }

    pub fn from_ipv4(ipv4_address: Ipv4Addr, port: Option<u16>, proto: Protocol) -> Self {
        let port = if let Some(port) = port {
            port
        } else {
            0
        };

        let address = SocketAddr::V4(SocketAddrV4::new(ipv4_address, port));

        match proto {
            Protocol::Tcp => Url::Tcp(address),
            Protocol::Udp => Url::Udp(address),
        }
    }

    pub fn from_ipv6(ipv6_address: Ipv6Addr, port: Option<u16>, proto: Protocol) -> Self {
        let port = if let Some(port) = port {
            port
        } else {
            0
        };

        let address = SocketAddr::V6(SocketAddrV6::new(ipv6_address, port, 0, 0));

        match proto {
            Protocol::Tcp => Url::Tcp(address),
            Protocol::Udp => Url::Udp(address),
        }
    }

    pub fn protocol(&self) -> Protocol {
        match *self {
            Url::Tcp(_) => Protocol::Tcp,
            Url::Udp(_) => Protocol::Udp,
        }
    }
}
