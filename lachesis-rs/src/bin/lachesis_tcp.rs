extern crate lachesis_rs;

use lachesis_rs::tcp_server::{TcpApp, TcpNode, TcpPeer};
use std::env::args;
use std::sync::Arc;

const BASE_PORT: usize = 9000;
const USAGE: &'static str = "Usage: tcp-client [number of nodes] [consensus-algorithm]";

/**
 * Main lachesis-rs TCP client entrypoint. Starts multiple TCP node peers.
 */
fn main() {
    env_logger::init();
    let args: Vec<String> = args().collect();
    if args.len() != 3 {
        panic!(USAGE);
    }
    let mut rng = ring::rand::SystemRandom::new();
    let n_nodes = args[1].parse::<usize>().unwrap();
    let algorithm = args[2].clone();
    let mut nodes = Vec::with_capacity(n_nodes);
    let mut peers = Vec::with_capacity(n_nodes);
    for i in 0..n_nodes {
        let a = format!("0.0.0.0:{}", BASE_PORT + i);
        let node = TcpNode::new(&mut rng, a.clone()).unwrap();
        peers.push(TcpPeer {
            address: a,
            id: node.node.get_id().clone(),
        });
        nodes.push(Arc::new(node));
    }
    for node in nodes.iter() {
        for peer in peers.iter() {
            if peer.id.clone() != node.node.get_id() {
                node.node.add_node(Arc::new(peer.clone())).unwrap();
            }
        }
    }
    let mut handles = Vec::with_capacity(n_nodes * 2);
    for node in nodes {
        let app = TcpApp::new(node.clone());
        let (handle1, handle2) = app.run().unwrap();
        handles.push(handle1);
        handles.push(handle2);
    }
    for handle in handles {
        handle.join().unwrap();
    }
}
