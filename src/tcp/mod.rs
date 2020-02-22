use crate::address::Address;
use crate::conns::{MAX_BUFFER_SIZE, RecvResult, RawConnection, SendResult};
use crate::commands::{Command, CommandReceiver as CommandRx};
use crate::events::{Event, EventPublisher as EventPub};
use crate::peers::PeerId;

use async_std::net::{SocketAddr, TcpListener, TcpStream};
use async_std::prelude::*;
use async_std::sync::Arc;
use async_std::task;
use async_trait::async_trait;
use futures::sink::SinkExt;
use futures::{select, FutureExt};
use log::*;

/// Represents a TCP connection.
#[derive(Clone)]
pub struct TcpConnection(Arc<TcpStream>);

impl TcpConnection {

    /// Creates a new TCP connection from a TCP stream instance.
    pub fn new(stream: TcpStream) -> Self {
        Self(Arc::new(stream))
    }

    /// Returns peer address associated with this connection.
    pub fn peer_addr(&self) -> SocketAddr {
        (*self.0).peer_addr()
            .expect("error reading peer address from TCP stream")
    }
}

impl Drop for TcpConnection {
    fn drop(&mut self) {
        self.0.shutdown(std::net::Shutdown::Both).expect("error shutting TCP stream down");
    }
}

impl Eq for TcpConnection {}
impl PartialEq for TcpConnection {
    fn eq(&self, other: &Self) -> bool {
        self.peer_addr() == other.peer_addr()
    }
}

#[async_trait]
impl RawConnection for TcpConnection {

    fn peer_id(&self) -> PeerId {
        PeerId(self.peer_addr())
    }

    async fn send(&mut self, bytes: Vec<u8>) -> SendResult<Event> {
        let mut stream = &*self.0;
        //let num_bytes = stream.write(&bytes).await
            //.expect("error sending bytes over TCP");

        stream.write_all(&bytes).await
            .expect("error sending bytes over TCP");

        Ok(Event::BytesSent {
            num_bytes: bytes.len(),
            receiver_addr: Address::new(self.peer_addr())
        }.into())
    }

    async fn recv(&mut self) -> RecvResult<Event> {
        let mut bytes = vec![0; MAX_BUFFER_SIZE];
        let mut stream = &*self.0;

        let num_bytes = stream.read(&mut bytes).await
            .expect("error receiving bytes over TCP");

        Ok(Event::BytesReceived {
            num_bytes,
            sender_addr: Address::new(self.peer_addr()),
            bytes
        }.into())
    }
}

pub mod actor {
    use super::*;

    /// Starts the TCP socket actor.
    pub async fn run(binding_addr: SocketAddr, mut command_rx: CommandRx, mut event_pub: EventPub) {
        debug!("[TCP  ] Starting TCP socket actor");

        let listener = TcpListener::bind(binding_addr).await.expect("error binding TCP listener");
        debug!("[TCP  ] Bound to {}",
            listener.local_addr().expect("error reading local address from TCP socket"));

        debug!("[TCP  ] Accepting TCP clients");
        // NOTE: 'fuse' ensures 'None' forever after the first 'None'
        let mut incoming = listener.incoming();
        //let mut command_rx = command_rx.fuse();

        loop {
            select! {
                // Handle connection requests
                stream = incoming.next().fuse() => {
                    if let Some(stream) = stream {
                        let stream = stream.expect("error unwrapping TCP stream");
                        debug!("[TCP  ] Successfully connected peer");

                        let peer_id = PeerId(stream.peer_addr().expect("error creating peer id"));
                        let tcp_conn = TcpConnection::new(stream);

                        // TODO: create a new actor for each connection which uses `mpsc` channel
                        event_pub.send(
                            Event::PeerConnectedViaTCP {
                                peer_id,
                                tcp_conn,
                            }.into()).await.expect("error sending event");
                    }
                },
                // Handle API commands
                command = command_rx.next().fuse() => {
                    if let Some(command) = command {
                        match command {
                            Command::Shutdown => {
                                debug!("[TCP  ] Received shutdown command");
                                break;
                            },
                            _ => (),
                        }
                    }
                },
            }
        }

        debug!("[TCP  ] Stopping TCP socket actor");
    }
}
