use event::EventHash;
use hashgraph::Hashgraph;

pub type PeerId = Vec<u8>;

pub trait Peer {
    fn get_sync(&self, pk: PeerId) -> (EventHash, Hashgraph);
    fn send_sync(&self, msg: (EventHash, Hashgraph));
    fn id(&self) -> &PeerId;
}
