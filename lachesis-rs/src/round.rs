use crate::event::event_hash::EventHash;
use crate::peer::PeerId;
use std::collections::HashMap;

pub struct Round {
    pub id: usize,
    witnesses: HashMap<PeerId, EventHash>,
}

impl Round {
    pub fn new(id: usize) -> Round {
        Round {
            id,
            witnesses: HashMap::new(),
        }
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

#[cfg(test)]
mod tests {
    use super::Round;
    use crate::event::event_hash::EventHash;
    use ring::digest::{digest, SHA256};

    #[test]
    fn it_should_correctly_get_all_witnesses() {
        let mut round = Round::new(0);
        let digest1 = digest(&SHA256, b"42");
        let event1 = EventHash(digest1.as_ref().to_vec());
        let digest2 = digest(&SHA256, b"fish");
        let event2 = EventHash(digest2.as_ref().to_vec());
        round.add_witness(vec![1], event1.clone());
        round.add_witness(vec![0], event2.clone());
        let mut expected = vec![event1, event2];
        expected.sort();
        let mut actual = round.witnesses();
        actual.sort();
        assert_eq!(round.id, 0);
        assert_eq!(expected, actual);
    }
}
