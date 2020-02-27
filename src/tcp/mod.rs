use crate::address::{Address, Protocol};
use crate::conns::{ByteReceiver, MAX_BUFFER_SIZE, RecvResult, RawConnection, self, SendResult};
use crate::commands::{Command, CommandReceiver as CommandRx};
use crate::events::{Event, EventPublisher as EventPub};
use crate::peers::PeerId;

use async_std::net::{SocketAddr, TcpListener, TcpStream};
use async_std::prelude::*;
use async_std::sync::Arc;
use async_std::task::spawn;
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
            .expect("[TCP  ] Error reading peer address from TCP stream")
    }
}

impl Drop for TcpConnection {
    fn drop(&mut self) {
        self.0.shutdown(std::net::Shutdown::Both)
            .expect("[TCP  ] Error shutting TCP stream down");
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
        PeerId(self.peer_addr().ip())
    }

    async fn send(&mut self, bytes: Vec<u8>) -> SendResult<Event> {
        let mut stream = &*self.0;

        stream.write_all(&bytes).await
            .expect("[TCP  ] Error writing bytes to stream");

        Ok(Event::BytesSent {
            num_bytes: bytes.len(),
            to: Address::new(self.peer_addr())
        }.into())
    }

    async fn recv(&mut self) -> RecvResult<Event> {
        let mut bytes = vec![0; MAX_BUFFER_SIZE];
        let mut stream = &*self.0;

        let num_bytes = stream.read(&mut bytes).await
            .expect("[TCP  ] Error reading bytes from stream");

        Ok(Event::BytesReceived {
            num_bytes,
            from: Address::new(self.peer_addr()),
            bytes
        }.into())
    }
}

/// Starts the TCP socket actor.
pub async fn actor(binding_addr: SocketAddr, mut command_rx: CommandRx, mut event_pub: EventPub) {
    debug!("[TCP  ] Starting actor");

    let listener = TcpListener::bind(binding_addr).await.expect("error binding TCP listener");
    debug!("[TCP  ] Bound to {}",
        listener.local_addr().expect("error reading local address from TCP socket"));

    debug!("[TCP  ] Accepting TCP clients");
    let mut incoming = listener.incoming();

    // NOTE: 'fuse' ensures 'None' forever after the first 'None'
    loop {
        select! {
            // Handle connection requests
            stream = incoming.next().fuse() => {
                if let Some(stream) = stream {
                    let stream = stream
                        .expect("[TCP  ] Error unwrapping incoming stream");

                    debug!("[TCP  ] Connection established (Incoming)");

                    let peer_id = PeerId(stream.peer_addr()
                        .expect("[TCP  ] Error creating peer id from stream")
                        .ip());

                    let conn = TcpConnection::new(stream);

                    let (sender, receiver) = conns::channel();

                    spawn(conn_actor(conn, receiver, event_pub.clone()));

                    event_pub.send(
                        Event::PeerAccepted {
                            peer_id,
                            protocol: Protocol::Tcp,
                            sender,
                        }).await
                        .expect("[TCP  ] Error sending PeerAccepted event");
                }
            },
            // Handle API commands
            command = command_rx.next().fuse() => {
                if let Some(command) = command {
                    debug!("[TCP  ] Received {:?}", command);
                    match command {
                        Command::Shutdown => {
                            break;
                        },
                        _ => (),
                    }
                }
            },
        }
    }

    debug!("[TCP  ] Stopping actor");
}

pub async fn try_connect(peer_id: &PeerId, peer_addr: &SocketAddr) -> Option<TcpConnection> {
    info!("[TCP  ] Trying to connect to peer: {:?} over TCP", peer_id);

    match TcpStream::connect(peer_addr).await {
        Ok(stream) => {
            info!("[TCP  ] Connection established (Outgoing)");

            let peer_id = PeerId(stream.peer_addr()
                .expect("[TCP  ] Error reading peer address from stream")
                .ip());

            Some(TcpConnection::new(stream))
        },
        Err(e) => {
            warn!("[TCP  ] Connection attempt failed (Peer offline?)");
            warn!("[TCP  ] Error was: {:?}", e);
            None
        }
    }
}

pub async fn conn_actor(mut conn: TcpConnection, mut bytes_rx: ByteReceiver, mut event_pub: EventPub) {
    debug!("[TCP  ] Starting connection actor");

    // NOTE: currently the only way to break out of this loop is if all senders are dropped
    loop {

        select! {
            bytes_in = conn.recv().fuse() => {
                if let Ok(bytes_in) = bytes_in {
                    event_pub.send(bytes_in).await.expect("[TCP  ] Error receiving bytes")
                }
            },

            bytes_out = bytes_rx.next().fuse() => {
                if let Some(bytes_out) = bytes_out {
                    let event = conn.send(bytes_out).await.expect("[TCP  ] Error sending bytes");

                    event_pub.send(event).await
                        .expect("[TCP  ] Error published event");
                }
            }
        }
    }

    debug!("[TCP  ] Stopping connection actor");
}
