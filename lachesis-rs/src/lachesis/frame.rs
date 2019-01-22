use crate::event::event_hash::EventHash;
use std::collections::{HashMap, HashSet};

#[derive(Clone)]
pub struct Frame {
    clotho_times: HashMap<EventHash, usize>,
    id: usize,
    pub root_set: HashSet<EventHash>,
}

impl Frame {
    pub fn new(id: usize) -> Frame {
        Frame {
            id,
            clotho_times: HashMap::new(),
            root_set: HashSet::new(),
        }
    }

    pub fn id(&self) -> usize {
        self.id
    }

    pub fn add(&mut self, hash: EventHash) {
        self.root_set.insert(hash);
    }

    pub fn set_clotho_time(&mut self, hash: EventHash, time: usize) {
        self.clotho_times.insert(hash, time);
    }
}
