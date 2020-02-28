use crate::address::{Address, Protocol};
use crate::conns::{ByteReceiver, MAX_BUFFER_SIZE, RawConnection, RECONNECT_COOLDOWN, RecvResult, self, SendResult};
use crate::commands::{Command, CommandReceiver as CommandRx};
use crate::errors::{ConnectionError, RecvError, SendError};
use crate::events::{Event, EventPublisher as EventPub};
use crate::peers::PeerId;

use async_std::net::{IpAddr, SocketAddr, TcpListener, TcpStream, Shutdown};
use async_std::prelude::*;
use async_std::task::spawn;
use async_trait::async_trait;
use futures::sink::SinkExt;
use futures::{select, FutureExt};
use log::*;

use std::result;

pub(crate) type ConnectionResult<T> = result::Result<T, ConnectionError>;

/// Represents a TCP connection.
pub struct TcpConnection {

    /// The peer id this connection is associated with.
    peer_id: PeerId,

    /// The underlying TCP stream instance.
    stream: TcpStream,

    /// The local address.
    local_addr: SocketAddr,

    /// The remote address.
    remote_addr: SocketAddr,
}

impl TcpConnection {

    /// Creates a new TCP connection from a TCP stream instance.
    pub fn new(stream: TcpStream) -> ConnectionResult<Self> {
        let local_addr = stream.local_addr()?;
        let remote_addr = stream.peer_addr()?;
        let peer_id = PeerId(remote_addr.ip());

        Ok(Self {
            peer_id,
            stream,
            local_addr,
            remote_addr,
        })
    }

    /// Returns the local address of this connection.
    pub fn local_addr(&self) -> SocketAddr {
        self.local_addr
    }

    /// Returns the remote address of this connection.
    pub fn remote_addr(&self) -> SocketAddr {
        self.remote_addr
    }

    /// Returns, whether this connection is broken.
    pub fn is_broken(&self) -> bool {
        // NOTE: is there a better way to detect broken connections?
        self.stream.peer_addr().is_err()
    }

    /// Returns, whether this connection is still healty.
    pub fn is_not_broken(&self) -> bool {
        !self.is_broken()
    }

    /// Updates local and remote address in case of Ip change.
    /// TODO: or will that always break the connection? Probably. Check that.
    pub fn update(&mut self) -> ConnectionResult<()> {
        self.local_addr = self.stream.local_addr()?;
        self.remote_addr = self.stream.peer_addr()?;
        Ok(())
    }

    /// Shuts down the reader and writer halves of the underlying stream.
    /// TODO: should be only shutdown the writing half, so we can still read the
    /// messages that are in the buffer? Check that.
    pub fn shutdown(&mut self) -> ConnectionResult<()> {
        Ok(self.stream.shutdown(Shutdown::Both)?)
    }
}

impl Drop for TcpConnection {
    fn drop(&mut self) {
        match self.shutdown() {
            Ok(()) => (),
            Err(e) => {
                warn!("[TCP  ] Error shutting down TCP stream");
                warn!("[TCP  ] Error was: {:?}", e);
            }
        }
    }
}

impl Eq for TcpConnection {}
impl PartialEq for TcpConnection {
    fn eq(&self, other: &Self) -> bool {
        // NOTE: do we need more than this comparison?
        self.remote_addr.ip() == other.remote_addr.ip()
    }
}

#[async_trait]
impl RawConnection for TcpConnection {

    fn peer_id(&self) -> PeerId {
        self.peer_id
    }

    async fn send(&mut self, bytes: Vec<u8>) -> SendResult<Event> {

        if self.is_not_broken() {

            self.stream.write_all(&bytes).await?;

            Ok(Event::BytesSent {
                num_bytes: bytes.len(),
                to: self.peer_id(),
            })
        } else {
            Err(SendError::SendBytes)
        }
    }

    async fn recv(&mut self) -> RecvResult<Event> {

        if self.is_not_broken() {

            let mut bytes = vec![0; MAX_BUFFER_SIZE];

            let num_bytes = self.stream.read(&mut bytes).await?;

            Ok(Event::BytesReceived {
                num_bytes,
                from: Address::new(self.remote_addr()),
                bytes
            })
        } else {
            Err(RecvError::RecvBytes)
        }
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

                    let conn = match TcpConnection::new(stream) {
                        Ok(conn) => conn,
                        Err(e) => {
                            warn!["TCP  ] Error creating TCP connection from stream"];
                            warn!["TCP  ] Error was: {:?}", e];
                            continue;
                        }
                    };

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
                    debug!("[TCP  ] {:?}", command);
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

            match TcpConnection::new(stream) {
                Ok(conn) => Some(conn),
                Err(e) => {
                    warn!("[TCP  ] Connection immediatedly broke");
                    warn!("[TCP  ] Error was: {:?}", e);
                    None
                }
            }
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

    loop {

        select! {

            bytes_in = conn.recv().fuse() => {
                if let Ok(bytes_in) = bytes_in {

                    event_pub.send(bytes_in).await.
                        expect("[TCP  ] Error receiving bytes");

                } else {
                    debug!("[TCP  ] Incoming byte stream stopped (from {:?})", conn.peer_id());

                    event_pub.send(Event::PeerDisconnected {
                        peer_id: conn.peer_id(),
                        reconnect: Some(RECONNECT_COOLDOWN),
                    }).await.expect("TCP  ] Error publ. Event::PeerDisconnected");

                    break;
                }
            },

            bytes_out = bytes_rx.next().fuse() => {
                if let Some(bytes_out) = bytes_out {
                    let event = conn.send(bytes_out).await
                        .expect("[TCP  ] Error sending bytes");

                    event_pub.send(event).await
                        .expect("[TCP  ] Error published event");

                } else {
                    break;
                }
            }
        }
    }

    debug!("[TCP  ] Stopping connection actor for {:?}", conn.peer_id());
}
