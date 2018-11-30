use lachesis_rs::{BTreeHashgraph, EventHash, HashgraphWire, Node, Peer, PeerId};
use ring::rand::SystemRandom;
use ring::signature;


fn create_node(rng: &mut SystemRandom) -> Node<DummyNode, BTreeHashgraph> {
    let hashgraph = BTreeHashgraph::new();
    let pkcs8_bytes = signature::Ed25519KeyPair::generate_pkcs8(rng).unwrap();
    let kp = signature::Ed25519KeyPair::from_pkcs8(untrusted::Input::from(&pkcs8_bytes)).unwrap();
    Node::new(kp, hashgraph).unwrap()
}

pub struct DummyNode {
    id: PeerId,
    pub node: Node<DummyNode, BTreeHashgraph>,
}

impl DummyNode {
    pub fn new(rng: &mut SystemRandom) -> DummyNode {
        let node = create_node(rng);
        let id = node.get_id();
        DummyNode {
            id,
            node,
        }
    }
}

impl Peer<BTreeHashgraph> for DummyNode {
    fn get_sync(&self, _pk: PeerId) -> (EventHash, BTreeHashgraph) {
        let (eh, wire): (EventHash, HashgraphWire) = self.node.respond_message().unwrap();
        let hashgraph = BTreeHashgraph::from(wire);
        (eh, hashgraph)
    }
    fn id(&self) -> &PeerId {
        &self.id
    }
}
