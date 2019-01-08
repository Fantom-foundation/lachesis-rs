use crate::event::event_hash::EventHash;
use crate::peer::PeerId;

pub trait PrintableHash: Sized + AsRef<[u8]> {
    fn printable_hash(&self) -> String {
        base64::encode(self)[..8].to_owned()
    }
}

impl PrintableHash for EventHash {}
impl PrintableHash for PeerId {}
