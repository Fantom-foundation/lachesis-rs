use crate::event::event_hash::EventHash;
use failure::Error;

pub type PeerId = Vec<u8>;

pub trait Peer<H>: Send + Sync {
    fn get_sync(&self, pk: PeerId, known: Option<&H>) -> Result<(EventHash, H), Error>;
    fn id(&self) -> &PeerId;
}
