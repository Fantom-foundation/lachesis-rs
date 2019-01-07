use event::{Event, Parents};
use failure::Error;
use peer::PeerId;
use ring::signature::{verify, ED25519};
use serde::Serialize;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct EventSignature(pub Vec<u8>);

impl EventSignature {
    pub fn verify<P: Parents + Clone + Serialize>(
        &self,
        event: &Event<P>,
        peer: &PeerId,
    ) -> Result<(), Error> {
        let public_key = untrusted::Input::from(peer.as_ref());
        let hash = event.hash()?;
        let msg = untrusted::Input::from(hash.as_ref());
        let signature = untrusted::Input::from(self.0.as_ref());
        verify(&ED25519, public_key, msg, signature).map_err(|e| Error::from(e))
    }
}

impl AsRef<[u8]> for EventSignature {
    fn as_ref(&self) -> &[u8] {
        self.0.as_ref()
    }
}
