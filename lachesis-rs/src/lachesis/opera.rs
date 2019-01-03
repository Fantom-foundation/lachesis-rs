use event::{Event, EventHash, ParentsPair};
use std::collections::HashMap;

pub struct Opera {
    graph: HashMap<EventHash, Event<ParentsPair>>,
}

impl Opera {
    pub fn new() -> Opera {
        let graph = HashMap::new();
        Opera {
            graph,
        }
    }

    pub fn sync(&mut self, other: Opera) {
        for (eh, ev) in other.graph {
            self.graph.insert(eh, ev);
        }
    }

    pub fn wire(&self) -> OperaWire {
        OperaWire {}
    }
}

pub struct OperaWire {}