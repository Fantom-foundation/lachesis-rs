#![feature(never_type)]

#[macro_use]
extern crate log;

mod dummy_node;
use self::dummy_node::DummyNode;
use lachesis_rs::Node;
use rand;
use std::env::args;
use std::sync::Arc;
use std::thread;
use std::time::Duration;

const USAGE: &'static str = "Usage: dummy [number of nodes]";

fn create_node(rng: &mut ring::rand::SystemRandom) -> DummyNode {
    DummyNode::new(rng)
}

fn spawn_node(node: &Arc<Box<DummyNode>>) -> (thread::JoinHandle<!>, thread::JoinHandle<!>) {
    let answer_thread_node = node.clone();
    let sync_thread_node = node.clone();
    let answer_handler = thread::spawn(move || loop {
        answer_thread_node.node.respond_message(None).unwrap();
        thread::sleep(Duration::from_millis(100));
    });
    let sync_handle = thread::spawn(move || {
        let mut rng = rand::thread_rng();
        let mut counter = 0usize;
        let node_id = sync_thread_node.node.get_id();
        loop {
            if counter % 100 == 0 {
                let head = sync_thread_node.node.get_head().unwrap();
                let (n_rounds, n_events) = sync_thread_node.node.get_stats().unwrap();
                info!(
                    "Node {:?}: Head {:?} Rounds {:?} Pending events {:?}",
                    node_id, head, n_rounds, n_events
                );
            }
            match sync_thread_node.node.run(&mut rng) {
                Ok(_) => {}
                Err(e) => panic!("Error! {}", e),
            };
            counter += 1;
            thread::sleep(Duration::from_millis(100));
        }
    });
    (answer_handler, sync_handle)
}

fn main() {
    env_logger::init();
    let args: Vec<String> = args().collect();
    if args.len() != 2 {
        panic!(USAGE);
    }
    let mut rng = ring::rand::SystemRandom::new();
    let n_nodes = args[1].parse::<usize>().unwrap();
    let mut nodes = Vec::with_capacity(n_nodes);
    for _ in 0..n_nodes {
        nodes.push(Arc::new(Box::new(create_node(&mut rng))));
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
        let (handle1, handle2) = spawn_node(node);
        handles.push(handle1);
        handles.push(handle2);
    }
    for handle in handles {
        handle.join().unwrap();
    }
}
