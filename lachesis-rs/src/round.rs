use event::EventHash;
use peer::PeerId;
use std::collections::HashMap;

pub struct Round {
    pub id: usize,
    witnesses: HashMap<PeerId, EventHash>
}

impl Round {
    pub fn new(id: usize) -> Round {
        Round { id, witnesses: HashMap::new() }
    }

    pub fn add_witness(&mut self, peer: PeerId, event: EventHash) {
        self.witnesses.insert(peer, event);
    }

    pub fn witnesses(&self) -> Vec<EventHash> {
        self.witnesses.values().map(|h| h.clone()).collect()
    }

    pub fn witnesses_map(&self) -> &HashMap<PeerId, EventHash> {
        &self.witnesses
    }
}