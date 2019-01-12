use crate::event::event_hash::EventHash;
use std::collections::HashSet;

pub struct Frame {
    id: usize,
    root_set: HashSet<EventHash>
}


impl Frame {
    pub fn new(id: usize) -> Frame {
        Frame {
            id,
            root_set: HashSet::new(),
        }
    }

    pub fn id(&self) -> usize {
        self.id
    }

    pub fn add(&mut self, hash: EventHash) {
        self.root_set.insert(hash);
    }
}