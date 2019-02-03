use crate::event::event_hash::EventHash;
use failure::Error;

pub type PeerId = Vec<u8>;

pub trait Peer<H>: Send + Sync {
    fn get_sync(&self, pk: PeerId, known: Option<&H>) -> Result<(EventHash, H), Error>;
    fn address(&self) -> String;
    fn id(&self) -> &PeerId;
}
