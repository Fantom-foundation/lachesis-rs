use super::parents_list::ParentsList;
use crate::errors::{HashgraphError, HashgraphErrorType};
use crate::event::event_hash::EventHash;
use crate::event::Event;
use failure::Error;
use std::collections::{BTreeMap, HashMap, HashSet};
use std::iter::FromIterator;

#[derive(Clone)]
pub struct OperaEvent {
    event: Event<ParentsList>,
    lamport_timestamp: usize,
    flag_table: HashSet<EventHash>,
}

pub struct Opera {
    graph: HashMap<EventHash, OperaEvent>,
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

    pub fn insert(&mut self, hash: EventHash, event: Event<ParentsList>) -> Result<(), Error> {
        self.lamport_timestamp += 1;
        let flag_table = match event.parents() {
            None => HashSet::with_capacity(0),
            Some(ps) => self.parent_list_to_flag_table(ps)?,
        };
        self.graph.insert(
            hash,
            OperaEvent {
                event,
                flag_table,
                lamport_timestamp: self.lamport_timestamp,
            }
        );
        Ok(())
    }

    fn parent_list_to_flag_table(&mut self, ps: &ParentsList) -> Result<HashSet<EventHash>, Error> {
        let mut ft = HashSet::new();
        for p in ps.0.iter() {
            let event = self
                .graph
                .get(p)
                .ok_or(Error::from(HashgraphError::new(
                    HashgraphErrorType::EventNotFound
                )))?
                .clone();
            ft.insert(p.clone());
            ft = ft.union(&event.flag_table).map(|e| e.clone()).collect();
        }
        Ok(ft)
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
    graph: BTreeMap<EventHash, OperaEvent>,
    pub lamport_timestamp: usize,
}
