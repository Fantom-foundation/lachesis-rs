#[macro_use]
extern crate failure;
#[macro_use]
extern crate log;
#[macro_use]
extern crate proptest;
#[macro_use]
extern crate serde_derive;
extern crate json;

macro_rules! get_from_mutex {
    ($resource: expr, $error: ident) => {
        $resource.lock().map_err(|e| $error::from(e))
    };
}

mod errors;
mod event;
mod hashgraph;
mod lachesis;
mod node;
mod peer;
mod printable_hash;
mod round;
mod server;
mod swirlds;
pub mod tcp_server;

pub use crate::event::{event_hash::EventHash, Event};
pub use crate::hashgraph::{BTreeHashgraph, Hashgraph, HashgraphWire};
pub use crate::lachesis::Lachesis;
pub use crate::node::Node;
pub use crate::peer::{Peer, PeerId};
pub use crate::server::Server;
pub use crate::swirlds::Swirlds;
