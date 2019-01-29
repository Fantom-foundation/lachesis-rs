use failure::Error;
use lachesis_rs::{BTreeHashgraph, EventHash, HashgraphWire, Node, Peer, PeerId, Swirlds};
use ring::rand::SystemRandom;
use ring::signature;

fn create_node(rng: &mut SystemRandom) -> Result<Swirlds<DummyNode, BTreeHashgraph>, Error> {
    let hashgraph = BTreeHashgraph::new();
    let pkcs8_bytes = signature::Ed25519KeyPair::generate_pkcs8(rng)
        .map_err(|e| Error::from_boxed_compat(Box::new(e)))?;
    let kp = signature::Ed25519KeyPair::from_pkcs8(untrusted::Input::from(&pkcs8_bytes))
        .map_err(|e| Error::from_boxed_compat(Box::new(e)))?;
    Swirlds::new(kp, hashgraph)
}

pub struct DummyNode {
    id: PeerId,
    pub node: Swirlds<DummyNode, BTreeHashgraph>,
}

impl DummyNode {
    pub fn new(rng: &mut SystemRandom) -> Result<DummyNode, Error> {
        match create_node(rng) {
            Ok(node) => Ok(DummyNode {
                id: node.get_id(),
                node,
            }),
            Err(e) => Err(e),
        }
    }
}

impl Peer<BTreeHashgraph> for DummyNode {
    fn get_sync(
        &self,
        _pk: PeerId,
        _h: Option<&BTreeHashgraph>,
    ) -> Result<(EventHash, BTreeHashgraph), Error> {
        let (eh, wire): (EventHash, HashgraphWire) = self.node.respond_message(None)?;
        let hashgraph = BTreeHashgraph::from(wire);
        Ok((eh, hashgraph))
    }
    fn id(&self) -> &PeerId {
        &self.id
    }
}
