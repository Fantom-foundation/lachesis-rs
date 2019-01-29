use crate::event::event_hash::EventHash;
use crate::hashgraph::{BTreeHashgraph, HashgraphWire};
use crate::node::Node;
use crate::peer::{Peer, PeerId};
use crate::swirlds::Swirlds;
use bincode::serialize;
use failure::Error;
use ring::rand::SystemRandom;
use ring::signature;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::Arc;
use std::thread::{sleep, spawn, JoinHandle};
use std::time::Duration;

fn create_node(rng: &mut SystemRandom) -> Result<Swirlds<TcpNode, BTreeHashgraph>, Error> {
    let hashgraph = BTreeHashgraph::new();
    let pkcs8_bytes = signature::Ed25519KeyPair::generate_pkcs8(rng)?;
    let kp = signature::Ed25519KeyPair::from_pkcs8(untrusted::Input::from(&pkcs8_bytes))?;
    Swirlds::new(kp, hashgraph)
}

pub struct TcpNode {
    id: PeerId,
    pub access_address: String,
    pub node: Swirlds<TcpNode, BTreeHashgraph>,
}

impl TcpNode {
    pub fn new(rng: &mut SystemRandom, access_address: String) -> Result<TcpNode, Error> {
        let node = create_node(rng)?;
        let id = node.get_id();
        Ok(TcpNode {
            access_address,
            id,
            node,
        })
    }
}

impl Peer<BTreeHashgraph> for TcpNode {
    fn get_sync(
        &self,
        pk: PeerId,
        _k: Option<&BTreeHashgraph>,
    ) -> Result<(EventHash, BTreeHashgraph), Error> {
        let peer = self.node.get_peer(&pk)?;
        let mut buffer = Vec::new();
        let mut stream = TcpStream::connect(&peer.access_address)?;
        let mut last_received = 0;
        while last_received == 0 {
            last_received = stream.read_to_end(&mut buffer)?;
        }
        let (eh, wire): (EventHash, HashgraphWire) = bincode::deserialize(&buffer)?;
        let hashgraph = BTreeHashgraph::from(wire);
        Ok((eh, hashgraph))
    }
    fn id(&self) -> &PeerId {
        &self.id
    }
}

pub struct TcpApp(pub Arc<Box<TcpNode>>);

impl TcpApp {
    pub fn new(n: Arc<Box<TcpNode>>) -> TcpApp {
        TcpApp(n)
    }

    pub fn run(self) -> Result<(JoinHandle<()>, JoinHandle<()>), Error> {
        let answer_thread_node = self.0.clone();
        let sync_thread_node = self.0.clone();
        let answer_handle = spawn(move || {
            let listener = TcpListener::bind(&answer_thread_node.access_address).unwrap();
            for stream_result in listener.incoming() {
                let mut stream = stream_result.unwrap();
                let message = answer_thread_node.node.respond_message(None).unwrap();
                let payload = serialize(&message).unwrap();
                stream.write(&payload).unwrap();
            }
            ()
        });
        let sync_handle = spawn(move || {
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
                sleep(Duration::from_millis(100));
            }
        });
        Ok((answer_handle, sync_handle))
    }
}
