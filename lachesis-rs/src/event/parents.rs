use crate::event::event_hash::EventHash;
use crate::hashgraph::Hashgraph;
use failure::Error;
use std::cmp::max;

pub trait Parents {
    fn self_parent(&self) -> Result<EventHash, Error>;
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ParentsPair(pub EventHash, pub EventHash);

impl ParentsPair {
    pub fn max_round<H: Hashgraph>(&self, hg: H) -> Result<usize, Error> {
        let other_round = hg.get(&self.1)?.round()?;
        let self_round = hg.get(&self.0)?.round()?;
        Ok(max(other_round, self_round))
    }
}

impl Parents for ParentsPair {
    fn self_parent(&self) -> Result<EventHash, Error> {
        Ok(self.0.clone())
    }
}
