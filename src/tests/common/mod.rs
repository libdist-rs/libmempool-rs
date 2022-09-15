use fnv::FnvHashMap;
use std::net::SocketAddr;

mod transaction;
pub use transaction::*;

mod id;
pub use id::*;

mod round;
pub use round::*;

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
