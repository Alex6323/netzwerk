use async_std::net::{SocketAddr, ToSocketAddrs};
use async_std::task;

use std::time::{SystemTime, UNIX_EPOCH};

pub fn to_single_socket_address(addr: impl ToSocketAddrs) -> SocketAddr {
    // TODO: proper error handling
    task::block_on(async {
        return addr.to_socket_addrs().await.unwrap().nth(0).unwrap();
    })
}

pub fn timestamp_millis() -> u64 {
    let unix_time = SystemTime::now().duration_since(UNIX_EPOCH)
        .expect("system clock error");

    unix_time.as_secs() * 1000 + u64::from(unix_time.subsec_millis())
}