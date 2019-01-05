extern crate base64;
extern crate bincode;
#[macro_use] extern crate failure;
#[macro_use] extern crate log;
#[macro_use] extern crate proptest;
extern crate rand;
extern crate ring;
extern crate serde;
#[macro_use] extern crate serde_derive;
extern crate untrusted;

macro_rules! get_from_mutex {
    ($resource: expr, $error: ident) => {
        $resource.lock().map_err(|e| $error::from(e))
    }
}

mod errors;
mod event;
mod hashgraph;
mod lachesis;
mod node;
mod peer;
mod printable_hash;
mod round;
mod swirlds;

pub use hashgraph::{BTreeHashgraph, Hashgraph, HashgraphWire};
pub use event::{Event, EventHash};
pub use node::Node;
pub use swirlds::Swirlds;
pub use peer::{Peer, PeerId};
