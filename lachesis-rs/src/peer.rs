use event::EventHash;
use hashgraph::Hashgraph;

pub type PeerId = Vec<u8>;

pub trait Peer<H: Hashgraph>: Send + Sync {
    fn get_sync(&self, pk: PeerId) -> (EventHash, H);
    fn id(&self) -> &PeerId;
}
