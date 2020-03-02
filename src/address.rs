use crate::utils;

use async_std::net::{Ipv4Addr, Ipv6Addr, SocketAddr, SocketAddrV4, SocketAddrV6, ToSocketAddrs};

use std::fmt;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Address {
    Ip(SocketAddr),
    Other(()),
}

impl Address {
    pub fn new(addr: impl ToSocketAddrs) -> Self {
        let addr = utils::to_single_socket_address(addr);
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

impl fmt::Display for Address {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Address::Ip(socket_addr) => write!(f, "{}", socket_addr),
            _ => write!(f, "{}", "other")
        }
    }
}

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
        let address = utils::to_single_socket_address(addr);

        match proto {
            Protocol::Tcp => Url::Tcp(address),
            Protocol::Udp => Url::Udp(address),
        }
    }

    pub fn from_str(url: &str) -> Self {
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

    pub fn address(&self) -> Address {
        match *self {
            Url::Tcp(socket_addr) => Address::Ip(socket_addr),
            Url::Udp(socket_addr) => Address::Ip(socket_addr),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Protocol {
    Tcp,
    Udp,
}

impl Protocol {
    pub fn is_tcp(&self) -> bool {
        *self == Protocol::Tcp
    }

    pub fn is_udp(&self) -> bool {
        *self == Protocol::Udp
    }
}

impl From<&str> for Protocol {
    fn from(s: &str) -> Self {
        match s {
            "tcp" => Self::Tcp,
            "udp" => Self::Udp,
            _ => panic!("Unknown protocol specifier"),
        }
    }
}