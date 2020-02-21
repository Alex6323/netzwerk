use crate::address::Address;
use crate::conns::{ConnectionResult, MAX_BUFFER_SIZE, RawConnection};
use crate::events::{Event, EventPublisher as EventPub};
use crate::peers::PeerId;

use async_std::net::{SocketAddr, TcpListener, TcpStream};
use async_std::prelude::*;
use async_std::sync::Arc;
use async_std::task;
use async_trait::async_trait;

use log::*;

#[derive(Clone)]
pub struct TcpConnection(Arc<TcpStream>);

impl TcpConnection {
    pub fn new(stream: TcpStream) -> Self {
        Self(Arc::new(stream))
    }

    pub async fn write(&self, bytes: &Vec<u8>) -> usize {
        let mut stream = &*self.0;
        stream.write(bytes).await.expect("error sending bytes using TCP")
    }

    pub async fn read(&self) {
        // TODO
    }

    pub fn peer_addr(&self) -> SocketAddr {
        (*self.0).peer_addr().expect("error reading peer address from TCP stream")
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

    async fn send(&mut self, bytes: Vec<u8>) -> ConnectionResult<Event> {
        let num_bytes = self.write(&bytes).await;

        Ok(Event::BytesSent {
            num_bytes,
            receiver_addr: Address::new(self.peer_addr())
        }.into())
    }

    async fn recv(&mut self) -> ConnectionResult<Event> {
        let mut buffer = vec![0; MAX_BUFFER_SIZE];
        let mut stream = &*self.0;

        let num_bytes = stream.read(&mut buffer)
            .await
            .expect("error receiving bytes using TCP");

        Ok(Event::BytesReceived {
            num_bytes,
            sender_addr: Address::new(self.peer_addr()),
            bytes: buffer
        }.into())
    }

    fn peer_id(&self) -> PeerId {
        PeerId(self.peer_addr())
    }
}

pub mod actor {
    use super::*;

    fn spawn_and_log_error<F>(fut: F) -> task::JoinHandle<()>
    where
        F: Future<Output = crate::Result<()>> + Send + 'static,
    {
        task::spawn(async move {
            if let Err(e) = fut.await {
                error!("{}", e)
            }
        })
    }

    /// Starts the TCP actor.
    pub async fn run(binding_addr: SocketAddr, event_pub: EventPub) {
        debug!("[TCP  ] Starting TCP actor");

        let listener = TcpListener::bind(binding_addr).await.expect("error binding TCP listener");
        debug!("[TCP  ] Bound to <<{}>>",
            listener.local_addr().expect("error reading local address from TCP socket"));

        debug!("[TCP  ] Accepting TCP clients");
        let mut incoming = listener.incoming();

        // NOTE: This loop should exit if all `TcpStream`s are dropped.
        while let Some(stream) = incoming.next().await {
            let stream = stream.expect("error unwrapping TCP stream");
            debug!("Successfully connected peer");

            let peer_id = PeerId(stream.peer_addr().expect("error creating peer id"));
            let tcp_conn = TcpConnection::new(stream);

            // TODO: create a new actor for each connection which uses `mpsc` channel

            event_pub.send(
                Event::PeerConnectedViaTCP {
                    peer_id,
                    tcp_conn,
                }.into()).expect("error sending event");
        }

        debug!("TCP listener stops listening");
    }
}
