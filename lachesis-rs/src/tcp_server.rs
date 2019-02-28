use crate::event::event_hash::EventHash;
use crate::hashgraph::{BTreeHashgraph, HashgraphWire};
use crate::lachesis::opera::{Opera, OperaWire};
use crate::lachesis::Lachesis;
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

fn create_lachesis_node(rng: &mut SystemRandom) -> Result<Lachesis<TcpPeer>, Error> {
    let pkcs8_bytes = signature::Ed25519KeyPair::generate_pkcs8(rng)?;
    let kp = signature::Ed25519KeyPair::from_pkcs8(untrusted::Input::from(&pkcs8_bytes))?;
    Ok(Lachesis::new(3, kp))
}

fn create_swirlds_node(rng: &mut SystemRandom) -> Result<Swirlds<TcpPeer, BTreeHashgraph>, Error> {
    let hashgraph = BTreeHashgraph::new();
    let pkcs8_bytes = signature::Ed25519KeyPair::generate_pkcs8(rng)?;
    let kp = signature::Ed25519KeyPair::from_pkcs8(untrusted::Input::from(&pkcs8_bytes))?;
    Swirlds::new(kp, hashgraph)
}

pub struct TcpNode<N: Node> {
    pub address: String,
    pub node: N,
}

impl TcpNode<Lachesis<TcpPeer>> {
    pub fn new_lachesis(
        rng: &mut SystemRandom,
        address: String,
    ) -> Result<TcpNode<Lachesis<TcpPeer>>, Error> {
        let node = create_lachesis_node(rng)?;
        Ok(TcpNode { address, node })
    }
}

impl TcpNode<Swirlds<TcpPeer, BTreeHashgraph>> {
    pub fn new(
        rng: &mut SystemRandom,
        address: String,
    ) -> Result<TcpNode<Swirlds<TcpPeer, BTreeHashgraph>>, Error> {
        let node = create_swirlds_node(rng)?;
        Ok(TcpNode { address, node })
    }
}

#[derive(Clone)]
pub struct TcpPeer {
    pub address: String,
    pub id: PeerId,
}

impl Peer<BTreeHashgraph> for TcpPeer {
    fn get_sync(
        &self,
        _pk: PeerId,
        _k: Option<&BTreeHashgraph>,
    ) -> Result<(EventHash, BTreeHashgraph), Error> {
        let mut buffer = Vec::new();
        let mut stream = TcpStream::connect(&self.address.clone())?;
        let mut last_received = 0;
        while last_received == 0 {
            last_received = stream.read_to_end(&mut buffer)?;
        }
        let (eh, wire): (EventHash, HashgraphWire) = bincode::deserialize(&buffer)?;
        let hashgraph = BTreeHashgraph::from(wire);
        Ok((eh, hashgraph))
    }
    fn address(&self) -> String {
        self.address.clone()
    }
    fn id(&self) -> &PeerId {
        &self.id
    }
}

impl Peer<Opera> for TcpPeer {
    fn get_sync(&self, _pk: PeerId, _k: Option<&Opera>) -> Result<(EventHash, Opera), Error> {
        let mut buffer = Vec::new();
        let mut stream = TcpStream::connect(&self.address.clone())?;
        let mut last_received = 0;
        while last_received == 0 {
            last_received = stream.read_to_end(&mut buffer)?;
        }
        let (eh, wire): (EventHash, OperaWire) = bincode::deserialize(&buffer)?;
        Ok((eh, wire.into_opera()))
    }
    fn address(&self) -> String {
        self.address.clone()
    }
    fn id(&self) -> &PeerId {
        &self.id
    }
}

pub struct TcpApp(Arc<TcpNode<Swirlds<TcpPeer, BTreeHashgraph>>>);

impl TcpApp {
    pub fn new(n: Arc<TcpNode<Swirlds<TcpPeer, BTreeHashgraph>>>) -> TcpApp {
        TcpApp(n)
    }

    pub fn run(self) -> Result<(JoinHandle<()>, JoinHandle<()>), Error> {
        let answer_thread_node = self.0.clone();
        let sync_thread_node = self.0.clone();
        let answer_handle = spawn(move || {
            let listener = TcpListener::bind(&answer_thread_node.address).unwrap();
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
