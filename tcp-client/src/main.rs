use std::env::args;
use std::sync::Arc;
use tcp_client::{TcpApp, TcpNode};

const BASE_PORT: usize = 9000;
const USAGE: &'static str = "Usage: tcp-client [number of nodes]";

fn main() {
    env_logger::init();
    let args: Vec<String> = args().collect();
    if args.len() != 2 {
        panic!(USAGE);
    }
    let mut rng = ring::rand::SystemRandom::new();
    let n_nodes = args[1].parse::<usize>().unwrap();
    let mut nodes = Vec::with_capacity(n_nodes);
    for i in 0..n_nodes {
        let a = format!("0.0.0.0:{}", BASE_PORT + i);
        nodes.push(Arc::new(Box::new(TcpNode::new(&mut rng, a))));
    }
    for node in nodes.iter() {
        for peer in nodes.iter() {
            if peer.node.get_id() != node.node.get_id() {
                node.node.add_node(peer.clone()).unwrap();
            }
        }
    }
    let mut handles = Vec::with_capacity(n_nodes * 2);
    for node in nodes.iter() {
        let app = TcpApp(node.clone());
        let (handle1, handle2) = app.run();
        handles.push(handle1);
        handles.push(handle2);
    }
    for handle in handles {
        handle.join().unwrap();
    }
}
