use event::Event;
use failure::Error;
use peer::PeerId;
use ring::signature::{ED25519, Signature, verify};
use serde::ser::{Serialize, SerializeStruct, Serializer};
use std::fmt;

#[derive(Clone)]
pub struct EventSignature(pub Signature);

impl EventSignature {
    pub fn verify(&self, event: &Event, peer: &PeerId) -> Result<(), Error> {
        let public_key = untrusted::Input::from(peer.as_ref());
        let hash = event.hash()?;
        let msg = untrusted::Input::from(hash.as_ref());
        let signature = untrusted::Input::from(self.0.as_ref());
        verify(&ED25519, public_key, msg, signature).map_err(|e| Error::from(e))
    }
}

impl Serialize for EventSignature {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
        let mut s = serializer.serialize_struct("EventId", 1)?;
        s.serialize_field("data", self.0.as_ref())?;
        s.end()
    }
}

impl AsRef<[u8]> for EventSignature {
    fn as_ref(&self) -> &[u8] {
        self.0.as_ref()
    }
}

impl fmt::Debug for EventSignature {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self.0.as_ref())
    }
}

impl PartialEq for EventSignature {
    fn eq(&self, other: &EventSignature) -> bool {
        self.0.as_ref() == other.0.as_ref()
    }
}
impl Eq for EventSignature {}
