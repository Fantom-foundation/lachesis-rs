use event::{Event, EventHash};
use lachesis::parents_list::ParentsList;
use std::collections::{BTreeMap, HashMap};
use std::iter::FromIterator;

pub struct Opera {
    graph: HashMap<EventHash, Event<ParentsList>>,
}

impl Opera {
    pub fn new() -> Opera {
        let graph = HashMap::new();
        Opera { graph }
    }

    pub fn sync(&mut self, other: Opera) {
        for (eh, ev) in other.graph {
            self.graph.insert(eh, ev);
        }
    }

    pub fn wire(&self) -> OperaWire {
        OperaWire {
            graph: BTreeMap::from_iter(self.graph.clone().into_iter()),
        }
    }

    pub fn insert(&mut self, hash: EventHash, event: Event<ParentsList>) {
        self.graph.insert(hash, event);
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
        }
    }
}

pub struct OperaWire {
    graph: BTreeMap<EventHash, Event<ParentsList>>,
}
