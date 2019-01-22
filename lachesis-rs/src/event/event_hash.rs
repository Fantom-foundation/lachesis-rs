#[derive(Clone, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub struct EventHash(pub [u8; 32]);

impl EventHash {
    pub fn new(digest: &[u8]) -> EventHash {
        let mut a: [u8; 32] = [0; 32];
        a.copy_from_slice(&digest[0..32]);
        EventHash(a)
    }
}

impl AsRef<[u8]> for EventHash {
    fn as_ref(&self) -> &[u8] {
        self.0.as_ref()
    }
}
