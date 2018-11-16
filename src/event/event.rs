use bincode::serialize;
use errors::EventError;
use event::{EventHash, EventSignature};
use failure::Error;
use hashgraph::Hashgraph;
use peer::PeerId;
use ring::digest::{digest, SHA256};
use std::cmp::max;
use std::collections::HashMap;

#[derive(Clone, Eq, PartialEq, Serialize)]
pub struct Parents(pub EventHash, pub EventHash);

impl Parents {
    pub fn max_round(&self, hg: &Hashgraph) -> Result<usize, Error> {
        let other_round = hg.get(&self.1)?.round()?;
        let self_round = hg.get(&self.0)?.round()?;
        Ok(max(other_round, self_round))
    }
}

#[derive(Clone, Serialize)]
pub struct Event {
    #[serde(skip)]
    can_see: HashMap<PeerId, EventHash>,
    #[serde(skip)]
    famous: Option<bool>,
    payload: Vec<Vec<u8>>,
    parents: Option<Parents>,
    timestamp: Option<u64>,
    creator: PeerId,
    signature: Option<EventSignature>,
    #[serde(skip)]
    round: Option<usize>,
    #[serde(skip)]
    round_received: Option<usize>,
}

impl Event {
    pub fn new(
        payload: Vec<Vec<u8>>,
        parents: Option<Parents>,
        creator: PeerId,
    ) -> Event {
        Event {
            can_see: HashMap::new(),
            creator,
            famous: None,
            payload,
            parents,
            round: None,
            round_received: None,
            signature: None,
            timestamp: None,
        }
    }

    #[inline]
    pub fn set_timestamp(&mut self, timestamp: u64) {
        self.timestamp = Some(timestamp);
    }

    #[inline]
    pub fn timestamp(&self) -> Result<u64, Error> {
        self.timestamp.clone().ok_or(Error::from(EventError::NoTimestamp))
    }

    #[inline]
    pub fn set_round_received(&mut self, round_received: usize) {
        self.round_received = Some(round_received);
    }

    #[inline]
    pub fn is_self_parent(&self, hash: &EventHash) -> bool {
        match self.parents {
            Some(Parents(self_parent, _)) => self_parent == *hash,
            None => false,
        }
    }

    #[inline]
    pub fn signature(&self) -> Result<EventSignature, Error> {
        self.signature.clone().ok_or(Error::from(EventError::NoSignature))
    }

    #[inline]
    pub fn famous(&mut self, famous: bool) {
        self.famous = Some(famous)
    }

    #[inline]
    pub fn is_famous(&self) -> bool {
        self.famous.unwrap_or(false)
    }

    #[inline]
    pub fn is_undefined(&self) -> bool {
        self.famous.is_none()
    }

    #[inline]
    pub fn can_see(&self) -> &HashMap<PeerId, EventHash> {
        &self.can_see
    }

    #[inline]
    pub fn set_can_see(&mut self, can_see: HashMap<PeerId, EventHash>) {
        self.can_see = can_see;
    }

    #[inline]
    pub fn round(&self) -> Result<usize, Error> {
        self.round.ok_or(Error::from(EventError::RoundNotSet))
    }

    #[inline]
    pub fn add_can_see(&mut self, peer: PeerId, hash: EventHash) {
        self.can_see.insert(peer, hash);
    }

    #[inline]
    pub fn is_root(&self) -> bool {
        self.parents.is_some()
    }

    #[inline]
    pub fn self_parent(&self) -> Result<EventHash, Error> {
        self.parents.clone().map(|p| p.0).ok_or(Error::from(EventError::NoSelfParent))
    }

    #[inline]
    pub fn parents(&self) -> &Option<Parents> {
        &self.parents
    }

    #[inline]
    pub fn creator(&self) -> &PeerId {
        &self.creator
    }

    pub fn sign(&mut self, signature: EventSignature) {
        self.signature = Some(signature);
    }

    #[inline]
    pub fn set_round(&mut self, round: usize) {
        self.round = Some(round);
    }

    pub fn hash(&self) -> Result<EventHash, Error> {
        let value = (
            self.payload.clone(),
            self.parents.clone(),
            self.timestamp.clone(),
            self.creator.clone()
        );
        let bytes = serialize(&value)?;
        Ok(EventHash(digest(&SHA256, bytes.as_ref())))
    }
    
    // TODO: Implement
    pub fn is_valid(&self, hash: &EventHash) -> Result<bool, Error> {
        self.signature.clone()
            .map(|s| s.verify(&self, &self.creator))
            .unwrap_or(Err(Error::from(EventError::UnsignedEvent)))?;
        Ok(hash.as_ref() == self.hash()?.as_ref())
    }
}
