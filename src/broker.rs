use crate::connection::{Connection, Connections, Tcp, Udp};

use crossbeam_channel as mpmc;
use futures::channel as mpsc;

/// Manages all TCP connections. Runs its own async event loop to process
/// incoming and outgoing TCP packets.
pub struct TcpBroker {
    conns: Connections<Tcp>,
}

impl TcpBroker {
    pub fn new() -> Self {
        Self {
            conns: Connections::new(),
        }
    }
    pub async fn run(&self) {
       // TODO
    }
}

/// Manages all UDP connections. Runs its own async event loop to process
/// incoming and outging UDP packets.
pub struct UdpBroker {
    conns: Connections<Udp>,
}

impl UdpBroker {
    pub fn new() -> Self {
        Self {
            conns: Connections::new(),
        }
    }

    pub async fn run(&self) {
        // TODO
    }
}