use super::parents_list::ParentsList;
use crate::event::{event_hash::EventHash, Event};
use std::collections::{BTreeMap, HashMap};
use std::iter::FromIterator;

pub struct Opera {
    graph: HashMap<EventHash, (usize, Event<ParentsList>)>,
    pub lamport_timestamp: usize,
}

impl Opera {
    pub fn new() -> Opera {
        let graph = HashMap::new();
        Opera {
            graph,
            lamport_timestamp: 0,
        }
    }

    pub fn sync(&mut self, other: Opera) {
        for (eh, ev) in other.graph {
            self.graph.insert(eh, ev);
        }
        if self.lamport_timestamp < other.lamport_timestamp {
            self.lamport_timestamp = other.lamport_timestamp;
        }
    }

    pub fn wire(&self) -> OperaWire {
        OperaWire {
            graph: BTreeMap::from_iter(self.graph.clone().into_iter()),
            lamport_timestamp: self.lamport_timestamp,
        }
    }

    pub fn insert(&mut self, hash: EventHash, event: Event<ParentsList>) {
        self.lamport_timestamp += 1;
        self.graph.insert(hash, (self.lamport_timestamp, event));
    }

    pub fn set_lamport(&mut self, lamport_timestamp: usize) {
        self.lamport_timestamp = lamport_timestamp;
    }

    pub fn diff(&self, wire: OperaWire) -> OperaWire {
        let local_keys: Vec<&EventHash> = self.graph.keys().collect();
        let remote_keys: Vec<&EventHash> = wire.graph.keys().collect();
        let diff_keys = local_keys
            .into_iter()
            .filter(|k| !remote_keys.contains(k))
            .map(|k| (k.clone(), self.graph.get(k).unwrap().clone()))
            .collect();
        OperaWire {
            graph: diff_keys,
            lamport_timestamp: self.lamport_timestamp,
        }
    }
}

pub struct OperaWire {
    graph: BTreeMap<EventHash, (usize, Event<ParentsList>)>,
    pub lamport_timestamp: usize,
}
