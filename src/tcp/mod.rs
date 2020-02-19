use crate::address::{Address, Url};
use crate::conns::{Connection, MAX_BUFFER_SIZE, NetIO};
use crate::events::{Event, EventSink, EventSource, EventType};
use crate::peers::{PeerId, Peers};
use crate::result;

use async_std::net::{SocketAddr, TcpListener, TcpStream};
use async_std::prelude::*;
use async_std::task;
use async_trait::async_trait;
use log::*;

use std::time::Duration;

pub async fn listen(binding_addr: SocketAddr, mut event_source: EventSource) {
    debug!("TCP module starts listening");

    let listener = TcpListener::bind(binding_addr).await.expect("error binding TCP listener");
    debug!("Successfully bound TCP listener to <<{}>>",
        listener.local_addr().expect("error reading local address from TCP socket"));

    debug!("Start accepting TCP clients");
    let mut incoming = listener.incoming();

    // NOTE: This loop should exit if all `TcpStream`s are dropped.
    while let Some(stream) = incoming.next().await {
        let stream = stream.expect("error unwrapping TCP stream");
        debug!("Successfully connected peer");

        handle_new_stream(stream, &mut event_source);
    }

    debug!("TCP listener stops listening");
}

pub async fn connect(event_sink: EventSink, mut event_source: EventSource) {
    debug!("TCP module starts re/connecting to unconnected peers");

    // Wait 5 seconds to check again if all peers are connected or need to be reconnected
    // TODO: check if there is a stable delay stream in async_std
    loop {
        task::sleep(Duration::from_millis(1000)).await;
        for (peer_id, peer) in &*initial_peers {
            if !peer.is_connected() {
                if let Url::Tcp(peer_addr) = peer.url() {
                    let stream = TcpStream::connect(peer_addr).await.expect("error connecting to peer");

                    handle_new_stream(stream, &mut event_source);

                } else if let Url::Udp(_peer_addr) = peer.url() {
                    // TODO
                }
            }
        }
    }
}

fn handle_new_stream(stream: TcpStream, event_source: &mut EventSource) {
    let peer_id = PeerId(stream.peer_addr().expect("error unwrapping remote address from TCP stream"));

    event_source.send(EventType::TcpPeerConnected { peer_id, stream }.into()).expect("error sending NewTcpConnection event");

}

pub struct Tcp {
    stream: TcpStream,
}

impl Tcp {
    fn new(stream: TcpStream) -> Self {
        Self { stream }
    }
}

impl Drop for Tcp {
    fn drop(&mut self) {
        self.stream.shutdown(std::net::Shutdown::Both).expect("error shutting TCP stream down");
    }
}

#[async_trait]
impl NetIO for Tcp {

    async fn send(&mut self, bytes: Vec<u8>) -> result::MessageResult<Event> {
        // TODO: error propagation
        let num_bytes = self.stream.write(&bytes)
            .await
            .expect("error sending bytes using TCP");

        Ok(EventType::MessageSent {
            num_bytes,
            receiver_addr: Address::new(self.stream.peer_addr().unwrap())
        }.into())
    }

    async fn recv(&mut self) -> result::MessageResult<Event> {
        let mut buffer = vec![0; MAX_BUFFER_SIZE];

        let num_bytes = self.stream.read(&mut buffer)
            .await
            .expect("error receiving bytes using TCP");

        Ok(EventType::MessageReceived {
            num_bytes,
            sender_addr: Address::new(self.stream.peer_addr().unwrap()),
            bytes: buffer
        }.into())
    }
}

impl Connection<Tcp> {
    pub fn new(stream: TcpStream) -> Self {
        Self(Tcp::new(stream))
    }
    pub async fn send(&mut self, bytes: Vec<u8>) -> result::MessageResult<Event> {
        Ok(self.0.send(bytes).await?)
    }
    pub async fn recv(&mut self) -> result::MessageResult<Event> {
        Ok(self.0.recv().await?)
    }
}

pub type TcpConnection = Connection<Tcp>;