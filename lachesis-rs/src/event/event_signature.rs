use crate::event::parents::Parents;
use crate::event::Event;
use crate::peer::PeerId;
use failure::Error;
use ring::signature::{verify, ED25519};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt::{self, Debug};

#[derive(Clone, Deserialize, Serialize)]
pub struct EventSignature(
    #[serde(serialize_with = "serialize_array")]
    #[serde(deserialize_with = "deserialize_array")]
    pub [u8; 64],
);

impl EventSignature {
    pub fn new(digest: &[u8]) -> EventSignature {
        let mut a: [u8; 64] = [0; 64];
        a.copy_from_slice(&digest[0..64]);
        EventSignature(a)
    }
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

fn serialize_array<S, T>(array: &[T], serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
    T: Serialize,
{
    array.serialize(serializer)
}

fn deserialize_array<'de, D>(deserializer: D) -> Result<[u8; 64], D::Error>
where
    D: Deserializer<'de>,
{
    let mut result: [u8; 64] = [0; 64];
    let slice: Vec<u8> = Deserialize::deserialize(deserializer)?;
    if slice.len() != 64 {
        return Err(::serde::de::Error::custom("input slice has wrong length"));
    }
    result.copy_from_slice(&slice);
    Ok(result)
}

impl AsRef<[u8]> for EventSignature {
    fn as_ref(&self) -> &[u8] {
        self.0.as_ref()
    }
}

impl Debug for EventSignature {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        self.0[..].fmt(formatter)
    }
}

impl Eq for EventSignature {}

impl PartialEq for EventSignature {
    #[inline]
    fn eq(&self, other: &EventSignature) -> bool {
        self.0[..] == other.0[..]
    }
    #[inline]
    fn ne(&self, other: &EventSignature) -> bool {
        self.0[..] != other.0[..]
    }
}
