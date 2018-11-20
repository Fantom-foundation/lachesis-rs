use ring::digest::Digest;
use serde::ser::{Serialize, Serializer};
use std::hash::{Hash, Hasher};
use std::cmp::Ordering;

#[derive(Clone, Copy)]
pub struct EventHash(pub Digest);

impl Serialize for EventHash {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
        self.0.as_ref().serialize(serializer)
    }
}

impl Hash for EventHash {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.as_ref().hash(state);
    }
}

impl PartialEq for EventHash {
    fn eq(&self, other: &EventHash) -> bool {
        self.0.as_ref() == other.0.as_ref()
    }
}
impl Eq for EventHash {}

impl PartialOrd for EventHash {
    fn partial_cmp(&self, other: &EventHash) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for EventHash {
    fn cmp(&self, other: &EventHash) -> Ordering {
        self.0.as_ref().cmp(other.0.as_ref())
    }
}

impl AsRef<[u8]> for EventHash {
    fn as_ref(&self) -> &[u8] {
        self.0.as_ref()
    }
}
