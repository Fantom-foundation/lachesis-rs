use bincode::{deserialize, serialize};
use lachesis_rs::{BTreeHashgraph, EventHash, Hashgraph, HashgraphWire, Node, Peer, PeerId, PeerMessage};
use std::cell::RefCell;
use std::sync::mpsc::{Sender, Receiver};
use std::rc::Rc;

struct DummyNode {
    id: PeerId,
    node: Node<DummyNode>,
    msg_sender: Sender<PeerMessage>,
    internal_channel: (Sender<Vec<u8>>, Receiver<Vec<u8>>),
}

impl Peer for DummyNode {
    fn get_sync(&self, _pk: PeerId) -> (EventHash, Rc<RefCell<Hashgraph>>) {
        self.msg_sender.send(PeerMessage::Sync(self.id.clone()));
        let payload = self.internal_channel.1.recv().unwrap();
        let (eh, wire): (EventHash, HashgraphWire) = deserialize(&payload).unwrap();
        let hashgraph = BTreeHashgraph::from(wire);
        (eh, Rc::new(RefCell::new(hashgraph)))
    }

    fn send_sync(&self, msg: (EventHash, HashgraphWire)) {
        let payload = serialize(&msg).unwrap();
        self.internal_channel.0.send(payload).unwrap();
    }
    fn id(&self) -> &PeerId {
        &self.id
    }
}