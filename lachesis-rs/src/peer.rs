use event::EventHash;
use hashgraph::Hashgraph;
use std::cell::RefCell;
use std::rc::Rc;

pub type PeerId = Vec<u8>;

pub trait Peer {
    fn get_sync(&self, pk: PeerId) -> (EventHash, Rc<RefCell<Hashgraph>>);
    fn send_sync(&self, msg: (EventHash, Rc<RefCell<Hashgraph>>));
    fn id(&self) -> &PeerId;
}
