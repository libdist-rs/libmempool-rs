use fnv::FnvHashMap;
use std::net::SocketAddr;

mod transaction;
pub use transaction::*;

mod id;
pub use id::*;

mod round;
pub use round::*;

/// Returns a Map of Id -> SocketAddr
///
/// The Ids are from 0..num_nodes
///
/// The socket addresses are from 127.0.0.1:base_port..base_port+num+nodes
pub fn get_peers(
    num_nodes: usize,
    base_port: u16,
) -> FnvHashMap<Id, SocketAddr> {
    let mut peers = FnvHashMap::default();
    for i in 0..num_nodes {
        peers.insert(
            i.into(),
            format!("127.0.0.1:{}", base_port + (i as u16))
                .parse()
                .unwrap(),
        );
    }
    peers
}
