use crate::connection::{Connections, Tcp, TcpConnection};
use crate::commands::{Command, CommandReceiver};
use crate::peers::{Peer, PeerId};
use crate::events::{Event, EventRx};

use async_std::task;
use log::*;

/// Starts the event loop.
pub fn start_el(event_rx: EventRx) {
    let mut tcp_conns = Connections::new();

    task::spawn(async move {
        while let Ok(event) = event_rx.recv() {
            match event {
                Event::NewTcpConnection { peer_id, stream } => tcp_conns.insert(peer_id, TcpConnection::new(stream)),
                Event::DropTcpConnection { peer_id } => tcp_conns.remove(&peer_id),
                /*
                Event::BroadcastMessage { message } => {
                    // TODO: send message via all TCP connections
                }
                Event::SendMessage { peer_id, message } => {
                    // FIXME: error handling
                    let conn = tcp_conns.get(&peer_id).expect("connection not found");
                }
                */
                Event::Shutdown => break,
                _ => (),
            }
        }
    });
}