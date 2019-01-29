#![feature(test)]

#[macro_use]
extern crate failure;
#[macro_use]
extern crate log;
#[macro_use]
extern crate proptest;
#[macro_use]
extern crate serde_derive;
extern crate json;
extern crate test;

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

#[cfg(test)]
mod tests {

    use crate::hashgraph::Hashgraph;
    use crate::printable_hash::PrintableHash;
    use crate::tcp_server::{TcpApp, TcpNode};
    use crate::{BTreeHashgraph, Event};
    //use std::env::args;
    use std::sync::Arc;
    use std::thread::sleep;
    use std::time::Duration;

    const BASE_PORT: usize = 9000;
    //const USAGE: &'static str = "Usage: tcp-client [number of nodes]";

    use test::Bencher;
    //use rand::Rng;
    use rand::prelude::*;
    //use std::mem::replace;

    use crate::event::parents::ParentsPair;
    //use crate::peer::Peer;

    fn test_hashgraph() -> BTreeHashgraph {
        let event1 = Event::new(vec![b"42".to_vec()], None, Vec::new());
        let hash1 = event1.hash().unwrap();
        let event2 = Event::new(vec![b"fish".to_vec()], None, vec![1]);
        let hash2 = event2.hash().unwrap();
        let event3 = Event::new(
            vec![b"ford prefect".to_vec()],
            Some(ParentsPair(hash1.clone(), hash2.clone())),
            Vec::new(),
        );
        let hash3 = event3.hash().unwrap();
        let event4 = Event::new(vec![b"42".to_vec()], None, vec![1]);
        let hash4 = event4.hash().unwrap();
        let event5 = Event::new(
            vec![b"ford prefect".to_vec()],
            Some(ParentsPair(hash3.clone(), hash4.clone())),
            Vec::new(),
        );
        let hash5 = event5.hash().unwrap();
        let event6 = Event::new(vec![b"42".to_vec()], None, vec![2]);
        let hash6 = event6.hash().unwrap();
        let event7 = Event::new(
            vec![b"ford prefect".to_vec()],
            Some(ParentsPair(hash5.clone(), hash6.clone())),
            Vec::new(),
        );
        let hash7 = event7.hash().unwrap();
        let mut hashgraph = BTreeHashgraph::new();
        hashgraph.insert(hash1.clone(), event1.clone());
        hashgraph.insert(hash2.clone(), event2.clone());
        hashgraph.insert(hash3.clone(), event3.clone());
        hashgraph.insert(hash4.clone(), event4.clone());
        hashgraph.insert(hash5.clone(), event5.clone());
        hashgraph.insert(hash6.clone(), event6.clone());
        hashgraph.insert(hash7.clone(), event7.clone());
        hashgraph
    }

    #[bench]
    fn setup_random_hashmap(b: &mut Bencher) {
        let mut val: u32 = 0;
        let mut rng = rand::thread_rng();
        let mut map = std::collections::HashMap::new();

        b.iter(|| {
            map.insert(rng.gen::<u8>() as usize, val);
            val += 1;
        })
    }

    #[bench]
    fn tcp(b: &mut Bencher) {
        env_logger::init();
        let mut rng = ring::rand::SystemRandom::new();
        let n_nodes = 4;
        let mut nodes = Vec::with_capacity(n_nodes);
        for i in 0..n_nodes {
            let a = format!("0.0.0.0:{}", BASE_PORT + i);
            nodes.push(Arc::new(Box::new(TcpNode::new(&mut rng, a))));
        }
        println!("* nodes are created");

        for node in nodes.iter() {
            for peer in nodes.iter() {
                if peer.node.get_id() != node.node.get_id() {
                    node.node.add_node(peer.clone()).unwrap();
                }
            }
        }
        println!("* nodes are peered");

        let mut handles = Vec::with_capacity(n_nodes * 2);
        for node in nodes.iter() {
            let app = TcpApp(node.clone());
            let (handle1, handle2) = app.run();
            handles.push(handle1);
            handles.push(handle2);
        }
        println!("* nodes are started");

        b.bench(|_| {
            nodes[0]
                .node
                .sync(nodes[0].node.get_head().unwrap(), test_hashgraph());
            //            for node in nodes.iter() {
            //                for peer in nodes.iter() {
            //                    if peer.node.get_id() != node.node.get_id() {
            //                        println!("*synch node");
            //                        node.get_sync(peer.node.get_id(), Some(&test_hashgraph()));
            //                   }
            //                }
            //            }

            let mut not_done = true;
            while not_done {
                not_done = false;
                for node in nodes.iter() {
                    println!("* hashgraph {:?}", node.node.get_hashgraph());
                    let node_id = node.node.get_id().printable_hash();
                    let (n_rounds, n_events) = node.node.get_stats().unwrap();
                    not_done = not_done || (n_events > 0);
                    println!(
                        "* node {:?} stats: rounds:{:?}; pending events:{:?}",
                        node_id, n_rounds, n_events
                    );
                }
                sleep(Duration::from_millis(100));
            }
        });

        println!("* handling for nodes is on");

        for handle in handles {
            handle.join().unwrap();
        }
    }
}
