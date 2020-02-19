//pub mod mpmc;

use async_std::net::{SocketAddr, ToSocketAddrs};
use async_std::task;

pub fn to_single_socket_address(addr: impl ToSocketAddrs) -> SocketAddr {
    // TODO: proper error handling
    task::block_on(async {
        return addr.to_socket_addrs().await.unwrap().nth(0).unwrap();
    })
}