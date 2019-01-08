use crate::errors::{HashgraphError, HashgraphErrorType};
use crate::event::event_hash::EventHash;
use crate::event::parents::ParentsPair;
use crate::event::Event;
use crate::peer::PeerId;
use failure::Error;
use std::collections::{BTreeMap, HashMap};
use std::iter::repeat_with;

#[derive(Deserialize, Serialize)]
pub struct HashgraphWire(BTreeMap<EventHash, Event<ParentsPair>>);

pub trait Hashgraph: Send + Sync {
    fn get_mut(&mut self, id: &EventHash) -> Result<&mut Event<ParentsPair>, Error>;
    fn get(&self, id: &EventHash) -> Result<&Event<ParentsPair>, Error>;
    fn insert(&mut self, hash: EventHash, event: Event<ParentsPair>);
    fn ancestors<'a>(&'a self, id: &'a EventHash) -> Vec<&'a EventHash>;
    fn other_ancestors<'a>(&'a self, id: &'a EventHash) -> Vec<&'a EventHash>;
    fn self_ancestors<'a>(&'a self, id: &'a EventHash) -> Vec<&'a EventHash>;
    fn higher(&self, a: &EventHash, b: &EventHash) -> bool;
    fn events_parents_can_see(&self, hash: &EventHash)
        -> Result<HashMap<PeerId, EventHash>, Error>;
    fn difference<H: Hashgraph>(&self, g: H) -> Vec<EventHash>;
    fn is_valid_event(&self, event: &Event<ParentsPair>) -> Result<bool, Error>;
    fn contains_key(&self, id: &EventHash) -> bool;
    fn wire(&self) -> HashgraphWire;
    fn find_roots(&self) -> Vec<EventHash>;
    fn find_self_child(&self, eh: &EventHash) -> Option<EventHash>;
}

#[derive(Clone, Debug)]
pub struct BTreeHashgraph(BTreeMap<EventHash, Event<ParentsPair>>);

impl BTreeHashgraph {
    pub fn new() -> BTreeHashgraph {
        BTreeHashgraph(BTreeMap::new())
    }
}

impl From<HashgraphWire> for BTreeHashgraph {
    fn from(v: HashgraphWire) -> Self {
        BTreeHashgraph(v.0)
    }
}

impl Hashgraph for BTreeHashgraph {
    fn get_mut(&mut self, id: &EventHash) -> Result<&mut Event<ParentsPair>, Error> {
        self.0.get_mut(id).ok_or(Error::from(HashgraphError::new(
            HashgraphErrorType::EventNotFound,
        )))
    }

    fn get(&self, id: &EventHash) -> Result<&Event<ParentsPair>, Error> {
        self.0.get(id).ok_or(Error::from(HashgraphError::new(
            HashgraphErrorType::EventNotFound,
        )))
    }

    fn insert(&mut self, hash: EventHash, event: Event<ParentsPair>) {
        self.0.insert(hash, event);
    }

    fn ancestors<'a>(&'a self, id: &'a EventHash) -> Vec<&'a EventHash> {
        let mut other_ancestors = self.other_ancestors(id);
        let self_ancestors = self.self_ancestors(id);
        other_ancestors.retain(|h| *h != id);
        other_ancestors.extend(self_ancestors.into_iter());
        other_ancestors
    }

    fn other_ancestors<'a>(&'a self, id: &'a EventHash) -> Vec<&'a EventHash> {
        let mut prev = Some(id);
        repeat_with(|| {
            if let Some(previous) = prev {
                let send = Some(previous);
                let event = self.get(previous).unwrap(); // TODO: Properly send this error
                prev = match event.parents() {
                    Some(ParentsPair(_, other_parent)) => Some(other_parent),
                    None => None,
                };
                send
            } else {
                None
            }
        })
        .take_while(|e| e.is_some())
        .map(|v| v.unwrap()) // This is safe because of the take_while
        .collect()
    }

    fn self_ancestors<'a>(&'a self, id: &'a EventHash) -> Vec<&'a EventHash> {
        let mut prev = Some(id);
        repeat_with(|| {
            if let Some(previous) = prev {
                let send = Some(previous);
                let event = self.get(previous).unwrap(); // TODO: Properly send this error
                prev = match event.parents() {
                    Some(ParentsPair(self_parent, _)) => Some(self_parent),
                    None => None,
                };
                send
            } else {
                None
            }
        })
        .take_while(|e| e.is_some())
        .map(|v| v.unwrap()) // This is safe because of the take_while
        .collect()
    }

    #[inline]
    fn higher(&self, a: &EventHash, b: &EventHash) -> bool {
        let a_self_ancestors = self.self_ancestors(a);
        if a_self_ancestors.contains(&b) {
            return true;
        }
        let b_self_ancestors = self.self_ancestors(b);
        if b_self_ancestors.contains(&a) {
            return false;
        }
        a_self_ancestors.len() > b_self_ancestors.len()
    }

    #[inline]
    fn events_parents_can_see(
        &self,
        hash: &EventHash,
    ) -> Result<HashMap<PeerId, EventHash>, Error> {
        match self.get(hash)?.parents() {
            Some(ParentsPair(self_parent, other_parent)) => {
                let self_parent_event = self.get(self_parent)?;
                let other_parent_event = self.get(other_parent)?;
                let mut result = HashMap::new();
                for (k, v) in self_parent_event.can_see().into_iter() {
                    result.insert(k.clone(), v.clone());
                }
                for (k, other) in other_parent_event.can_see().into_iter() {
                    if result.contains_key(k) {
                        let value = (&result[k]).clone();
                        if self.higher(other, &value) {
                            result.insert(k.clone(), other.clone());
                        }
                    } else {
                        result.insert(k.clone(), other.clone());
                    }
                }
                Ok(result)
            }
            None => Ok(HashMap::new()),
        }
    }

    fn difference<H: Hashgraph>(&self, g: H) -> Vec<EventHash> {
        self.0
            .keys()
            .filter(|e| !g.contains_key(e))
            .map(|e| (*e).clone())
            .collect()
    }

    fn is_valid_event(&self, event: &Event<ParentsPair>) -> Result<bool, Error> {
        match event.parents() {
            Some(ParentsPair(self_parent, other_parent)) => Ok(self.0.contains_key(self_parent)
                && self.0.contains_key(other_parent)
                && self.0[self_parent].creator() == event.creator()
                && self.0[other_parent].creator() != event.creator()),
            None => Ok(true),
        }
    }

    fn contains_key(&self, id: &EventHash) -> bool {
        self.0.contains_key(id)
    }

    fn wire(&self) -> HashgraphWire {
        HashgraphWire(self.0.clone())
    }

    fn find_roots(&self) -> Vec<EventHash> {
        self.0
            .values()
            .filter(|e| e.is_root())
            .map(|e| e.hash().unwrap())
            .collect()
    }

    fn find_self_child(&self, eh: &EventHash) -> Option<EventHash> {
        self.0
            .values()
            .find(|e| {
                let e = *e;
                match e.parents() {
                    Some(ParentsPair(sp, _)) => sp == eh,
                    None => false,
                }
            })
            .map(|e| e.hash().unwrap())
    }
}

#[cfg(test)]
mod tests {
    use super::{BTreeHashgraph, Hashgraph};
    use crate::event::{event_hash::EventHash, parents::ParentsPair, Event};
    use std::collections::HashMap;

    #[test]
    fn it_should_succeed_on_event_with_no_parents() {
        let mut hashgraph = BTreeHashgraph::new();
        let event = Event::new(vec![], None, Vec::new());
        let hash = event.hash().unwrap();
        hashgraph.insert(hash.clone(), event.clone());
        assert!(hashgraph.is_valid_event(&event).unwrap());
    }

    #[test]
    fn it_should_succeed_on_event_with_correct_parents() {
        let mut hashgraph = BTreeHashgraph::new();
        let n1 = vec![42];
        let n2 = vec![43];
        let self_parent = Event::new(vec![], None, n1.clone());
        let other_parent = Event::new(vec![], None, n2);
        let sphash = self_parent.hash().unwrap();
        let ophash = other_parent.hash().unwrap();
        let event = Event::new(
            vec![],
            Some(ParentsPair(sphash.clone(), ophash.clone())),
            n1,
        );
        let hash = event.hash().unwrap();
        hashgraph.insert(ophash.clone(), other_parent);
        hashgraph.insert(sphash.clone(), self_parent);
        hashgraph.insert(hash.clone(), event.clone());
        assert!(hashgraph.is_valid_event(&event).unwrap());
    }

    #[test]
    fn it_should_fail_if_self_parent_creator_differs() {
        let mut hashgraph = BTreeHashgraph::new();
        let n1 = vec![42];
        let n2 = vec![43];
        let n3 = vec![44];
        let self_parent = Event::new(vec![], None, n1);
        let other_parent = Event::new(vec![], None, n2);
        let sphash = self_parent.hash().unwrap();
        let ophash = other_parent.hash().unwrap();
        let event = Event::new(
            vec![],
            Some(ParentsPair(sphash.clone(), ophash.clone())),
            n3,
        );
        let hash = event.hash().unwrap();
        hashgraph.insert(ophash.clone(), other_parent);
        hashgraph.insert(sphash.clone(), self_parent);
        hashgraph.insert(hash.clone(), event.clone());
        assert!(!hashgraph.is_valid_event(&event).unwrap());
    }

    #[test]
    fn it_should_fail_if_other_parent_its_sent_by_same_node() {
        let mut hashgraph = BTreeHashgraph::new();
        let n1 = vec![42];
        let n2 = vec![43];
        let self_parent = Event::new(vec![], None, n1);
        let other_parent = Event::new(vec![], None, n2.clone());
        let sphash = self_parent.hash().unwrap();
        let ophash = other_parent.hash().unwrap();
        let event = Event::new(
            vec![],
            Some(ParentsPair(sphash.clone(), ophash.clone())),
            n2.clone(),
        );
        let hash = event.hash().unwrap();
        hashgraph.insert(ophash.clone(), other_parent);
        hashgraph.insert(sphash.clone(), self_parent);
        hashgraph.insert(hash.clone(), event.clone());
        assert!(!hashgraph.is_valid_event(&event).unwrap());
    }

    #[test]
    fn it_should_fail_if_self_parent_isnt_in_the_graph() {
        let mut hashgraph = BTreeHashgraph::new();
        let n1 = vec![42];
        let n2 = vec![43];
        let self_parent: Event<ParentsPair> = Event::new(vec![], None, n1);
        let other_parent = Event::new(vec![], None, n2.clone());
        let sphash = self_parent.hash().unwrap();
        let ophash = other_parent.hash().unwrap();
        let event = Event::new(
            vec![],
            Some(ParentsPair(sphash.clone(), ophash.clone())),
            n2.clone(),
        );
        let hash = event.hash().unwrap();
        hashgraph.insert(ophash.clone(), other_parent);
        hashgraph.insert(hash.clone(), event.clone());
        assert!(!hashgraph.is_valid_event(&event).unwrap());
    }

    #[test]
    fn it_should_fail_if_other_parent_isnt_in_the_graph() {
        let mut hashgraph = BTreeHashgraph::new();
        let n1 = vec![42];
        let n2 = vec![43];
        let self_parent = Event::new(vec![], None, n1);
        let other_parent: Event<ParentsPair> = Event::new(vec![], None, n2.clone());
        let sphash = self_parent.hash().unwrap();
        let ophash = other_parent.hash().unwrap();
        let event = Event::new(
            vec![],
            Some(ParentsPair(sphash.clone(), ophash.clone())),
            n2.clone(),
        );
        let hash = event.hash().unwrap();
        hashgraph.insert(sphash.clone(), self_parent);
        hashgraph.insert(hash.clone(), event.clone());
        assert!(!hashgraph.is_valid_event(&event).unwrap());
    }

    #[test]
    fn it_should_calculate_the_difference_of_two_hashgraphs() {
        let event1 = Event::new(vec![b"42".to_vec()], None, Vec::new());
        let hash1 = event1.hash().unwrap();
        let event2 = Event::new(vec![b"fish".to_vec()], None, Vec::new());
        let hash2 = event2.hash().unwrap();
        let event3 = Event::new(vec![b"ford prefect".to_vec()], None, Vec::new());
        let hash3 = event3.hash().unwrap();
        let mut hg1 = BTreeHashgraph::new();
        let mut hg2 = BTreeHashgraph::new();
        hg1.insert(hash1.clone(), event1);
        hg1.insert(hash2.clone(), event2);
        hg2.insert(hash3.clone(), event3);
        let mut expected = vec![hash1.clone(), hash2.clone()];
        expected.sort();
        let mut actual = hg1.difference(hg2);
        actual.sort();
        assert_eq!(expected, actual)
    }

    #[test]
    fn it_should_return_self_ancestors() {
        let event1 = Event::new(vec![b"42".to_vec()], None, Vec::new());
        let hash1 = event1.hash().unwrap();
        let event2 = Event::new(vec![b"fish".to_vec()], None, vec![1]);
        let hash2 = event2.hash().unwrap();
        let event3 = Event::new(
            vec![b"ford prefect".to_vec()],
            Some(ParentsPair(hash1.clone(), hash2.clone())),
            Vec::new(),
        );
        let hash3 = event3.hash().unwrap();
        let event4 = Event::new(vec![b"42".to_vec()], None, vec![1]);
        let hash4 = event4.hash().unwrap();
        let event5 = Event::new(
            vec![b"ford prefect".to_vec()],
            Some(ParentsPair(hash3.clone(), hash4.clone())),
            Vec::new(),
        );
        let hash5 = event5.hash().unwrap();
        let event6 = Event::new(vec![b"42".to_vec()], None, vec![2]);
        let hash6 = event6.hash().unwrap();
        let event7 = Event::new(
            vec![b"ford prefect".to_vec()],
            Some(ParentsPair(hash5.clone(), hash6.clone())),
            Vec::new(),
        );
        let hash7 = event7.hash().unwrap();
        let mut hashgraph = BTreeHashgraph::new();
        hashgraph.insert(hash1.clone(), event1.clone());
        hashgraph.insert(hash2.clone(), event2.clone());
        hashgraph.insert(hash3.clone(), event3.clone());
        hashgraph.insert(hash4.clone(), event4.clone());
        hashgraph.insert(hash5.clone(), event5.clone());
        hashgraph.insert(hash6.clone(), event6.clone());
        hashgraph.insert(hash7.clone(), event7.clone());
        let mut expected = vec![&hash1, &hash3, &hash5, &hash7];
        expected.sort();
        let mut actual = hashgraph.self_ancestors(&hash7);
        actual.sort();
        assert_eq!(expected, actual);
    }

    #[test]
    fn it_should_return_other_ancestors() {
        let event1 = Event::new(vec![b"42".to_vec()], None, Vec::new());
        let hash1 = event1.hash().unwrap();
        let event2 = Event::new(vec![b"fish".to_vec()], None, vec![1]);
        let hash2 = event2.hash().unwrap();
        let event3 = Event::new(
            vec![b"ford prefect".to_vec()],
            Some(ParentsPair(hash2.clone(), hash1.clone())),
            Vec::new(),
        );
        let hash3 = event3.hash().unwrap();
        let event4 = Event::new(vec![b"42".to_vec()], None, vec![1]);
        let hash4 = event4.hash().unwrap();
        let event5 = Event::new(
            vec![b"ford prefect".to_vec()],
            Some(ParentsPair(hash4.clone(), hash3.clone())),
            Vec::new(),
        );
        let hash5 = event5.hash().unwrap();
        let event6 = Event::new(vec![b"42".to_vec()], None, vec![2]);
        let hash6 = event6.hash().unwrap();
        let event7 = Event::new(
            vec![b"ford prefect".to_vec()],
            Some(ParentsPair(hash6.clone(), hash5.clone())),
            Vec::new(),
        );
        let hash7 = event7.hash().unwrap();
        let mut hashgraph = BTreeHashgraph::new();
        hashgraph.insert(hash1.clone(), event1.clone());
        hashgraph.insert(hash2.clone(), event2.clone());
        hashgraph.insert(hash3.clone(), event3.clone());
        hashgraph.insert(hash4.clone(), event4.clone());
        hashgraph.insert(hash5.clone(), event5.clone());
        hashgraph.insert(hash6.clone(), event6.clone());
        hashgraph.insert(hash7.clone(), event7.clone());
        let mut expected = vec![&hash1, &hash3, &hash5, &hash7];
        expected.sort();
        let mut actual = hashgraph.other_ancestors(&hash7);
        actual.sort();
        assert_eq!(expected, actual);
    }

    #[test]
    fn it_should_return_ancestors() {
        let event1 = Event::new(vec![b"42".to_vec()], None, Vec::new());
        let hash1 = event1.hash().unwrap();
        let event2 = Event::new(vec![b"fish".to_vec()], None, vec![1]);
        let hash2 = event2.hash().unwrap();
        let event3 = Event::new(
            vec![b"ford prefect".to_vec()],
            Some(ParentsPair(hash2.clone(), hash1.clone())),
            Vec::new(),
        );
        let hash3 = event3.hash().unwrap();
        let event4 = Event::new(vec![b"42".to_vec()], None, vec![1]);
        let hash4 = event4.hash().unwrap();
        let event5 = Event::new(
            vec![b"ford prefect".to_vec()],
            Some(ParentsPair(hash4.clone(), hash3.clone())),
            Vec::new(),
        );
        let hash5 = event5.hash().unwrap();
        let event6 = Event::new(vec![b"42".to_vec()], None, vec![2]);
        let hash6 = event6.hash().unwrap();
        let event7 = Event::new(
            vec![b"ford prefect".to_vec()],
            Some(ParentsPair(hash6.clone(), hash5.clone())),
            Vec::new(),
        );
        let hash7 = event7.hash().unwrap();
        let mut hashgraph = BTreeHashgraph::new();
        hashgraph.insert(hash1.clone(), event1.clone());
        hashgraph.insert(hash2.clone(), event2.clone());
        hashgraph.insert(hash3.clone(), event3.clone());
        hashgraph.insert(hash4.clone(), event4.clone());
        hashgraph.insert(hash5.clone(), event5.clone());
        hashgraph.insert(hash6.clone(), event6.clone());
        hashgraph.insert(hash7.clone(), event7.clone());
        let mut expected = vec![&hash1, &hash3, &hash5, &hash6, &hash7];
        expected.sort();
        let mut actual = hashgraph.ancestors(&hash7);
        actual.sort();
        assert_eq!(expected, actual);
    }

    #[test]
    fn it_should_not_be_higher_if_its_ancestor() {
        let event1 = Event::new(vec![b"42".to_vec()], None, Vec::new());
        let hash1 = event1.hash().unwrap();
        let event2 = Event::new(vec![b"fish".to_vec()], None, vec![1]);
        let hash2 = event2.hash().unwrap();
        let event3 = Event::new(
            vec![b"ford prefect".to_vec()],
            Some(ParentsPair(hash2.clone(), hash1.clone())),
            Vec::new(),
        );
        let hash3 = event3.hash().unwrap();
        let event4 = Event::new(vec![b"42".to_vec()], None, vec![1]);
        let hash4 = event4.hash().unwrap();
        let event5 = Event::new(
            vec![b"ford prefect".to_vec()],
            Some(ParentsPair(hash4.clone(), hash3.clone())),
            Vec::new(),
        );
        let hash5 = event5.hash().unwrap();
        let event6 = Event::new(vec![b"42".to_vec()], None, vec![2]);
        let hash6 = event6.hash().unwrap();
        let event7 = Event::new(
            vec![b"ford prefect".to_vec()],
            Some(ParentsPair(hash6.clone(), hash5.clone())),
            Vec::new(),
        );
        let hash7 = event7.hash().unwrap();
        let mut hashgraph = BTreeHashgraph::new();
        hashgraph.insert(hash1.clone(), event1.clone());
        hashgraph.insert(hash2.clone(), event2.clone());
        hashgraph.insert(hash3.clone(), event3.clone());
        hashgraph.insert(hash4.clone(), event4.clone());
        hashgraph.insert(hash5.clone(), event5.clone());
        hashgraph.insert(hash6.clone(), event6.clone());
        hashgraph.insert(hash7.clone(), event7.clone());
        assert!(!hashgraph.higher(&hash6, &hash7));
    }

    #[test]
    fn it_should_be_higher_if_its_child() {
        let event1 = Event::new(vec![b"42".to_vec()], None, Vec::new());
        let hash1 = event1.hash().unwrap();
        let event2 = Event::new(vec![b"fish".to_vec()], None, vec![1]);
        let hash2 = event2.hash().unwrap();
        let event3 = Event::new(
            vec![b"ford prefect".to_vec()],
            Some(ParentsPair(hash2.clone(), hash1.clone())),
            Vec::new(),
        );
        let hash3 = event3.hash().unwrap();
        let event4 = Event::new(vec![b"42".to_vec()], None, vec![1]);
        let hash4 = event4.hash().unwrap();
        let event5 = Event::new(
            vec![b"ford prefect".to_vec()],
            Some(ParentsPair(hash4.clone(), hash3.clone())),
            Vec::new(),
        );
        let hash5 = event5.hash().unwrap();
        let event6 = Event::new(vec![b"42".to_vec()], None, vec![2]);
        let hash6 = event6.hash().unwrap();
        let event7 = Event::new(
            vec![b"ford prefect".to_vec()],
            Some(ParentsPair(hash6.clone(), hash5.clone())),
            Vec::new(),
        );
        let hash7 = event7.hash().unwrap();
        let mut hashgraph = BTreeHashgraph::new();
        hashgraph.insert(hash1.clone(), event1.clone());
        hashgraph.insert(hash2.clone(), event2.clone());
        hashgraph.insert(hash3.clone(), event3.clone());
        hashgraph.insert(hash4.clone(), event4.clone());
        hashgraph.insert(hash5.clone(), event5.clone());
        hashgraph.insert(hash6.clone(), event6.clone());
        hashgraph.insert(hash7.clone(), event7.clone());
        assert!(hashgraph.higher(&hash7, &hash6));
    }

    #[test]
    fn it_should_return_expected_events_that_parents_can_see() {
        let event1 = Event::new(vec![b"42".to_vec()], None, Vec::new());
        let hash1 = event1.hash().unwrap();
        let event2 = Event::new(vec![b"fish".to_vec()], None, vec![1]);
        let hash2 = event2.hash().unwrap();
        let event3 = Event::new(
            vec![b"ford prefect".to_vec()],
            Some(ParentsPair(hash2.clone(), hash1.clone())),
            Vec::new(),
        );
        let hash3 = event3.hash().unwrap();
        let event4 = Event::new(vec![b"42".to_vec()], None, vec![1]);
        let hash4 = event4.hash().unwrap();
        let event5 = Event::new(
            vec![b"ford prefect".to_vec()],
            Some(ParentsPair(hash4.clone(), hash3.clone())),
            Vec::new(),
        );
        let hash5 = event5.hash().unwrap();
        let event6 = Event::new(vec![b"42".to_vec()], None, vec![2]);
        let hash6 = event6.hash().unwrap();
        let event7 = Event::new(
            vec![b"ford prefect".to_vec()],
            Some(ParentsPair(hash6.clone(), hash5.clone())),
            Vec::new(),
        );
        let hash7 = event7.hash().unwrap();
        let mut hashgraph = BTreeHashgraph::new();
        hashgraph.insert(hash1.clone(), event1.clone());
        hashgraph.insert(hash2.clone(), event2.clone());
        hashgraph.insert(hash3.clone(), event3.clone());
        hashgraph.insert(hash4.clone(), event4.clone());
        hashgraph.insert(hash5.clone(), event5.clone());
        hashgraph.insert(hash6.clone(), event6.clone());
        hashgraph.insert(hash7.clone(), event7.clone());
        assert!(hashgraph.higher(&hash5, &hash6));
    }

    #[test]
    fn it_should_be_higher_if_has_more_ancestors() {
        let event1 = Event::new(vec![b"42".to_vec()], None, Vec::new());
        let hash1 = event1.hash().unwrap();
        let event2 = Event::new(vec![b"fish".to_vec()], None, vec![1]);
        let hash2 = event2.hash().unwrap();
        let event3 = Event::new(
            vec![b"ford prefect".to_vec()],
            Some(ParentsPair(hash2.clone(), hash1.clone())),
            Vec::new(),
        );
        let hash3 = event3.hash().unwrap();
        let event4 = Event::new(vec![b"42".to_vec()], None, vec![1]);
        let hash4 = event4.hash().unwrap();
        let mut event5 = Event::new(
            vec![b"ford prefect".to_vec()],
            Some(ParentsPair(hash4.clone(), hash3.clone())),
            Vec::new(),
        );
        event5.add_can_see(vec![2], hash3.clone());
        event5.add_can_see(vec![1], hash4.clone());
        let hash5 = event5.hash().unwrap();
        let mut event6 = Event::new(vec![b"42".to_vec()], None, vec![2]);
        event6.add_can_see(vec![2], hash4.clone());
        let hash6 = event6.hash().unwrap();
        let event7 = Event::new(
            vec![b"ford prefect".to_vec()],
            Some(ParentsPair(hash6.clone(), hash5.clone())),
            Vec::new(),
        );
        let hash7 = event7.hash().unwrap();
        let mut hashgraph = BTreeHashgraph::new();
        hashgraph.insert(hash1.clone(), event1.clone());
        hashgraph.insert(hash2.clone(), event2.clone());
        hashgraph.insert(hash3.clone(), event3.clone());
        hashgraph.insert(hash4.clone(), event4.clone());
        hashgraph.insert(hash5.clone(), event5.clone());
        hashgraph.insert(hash6.clone(), event6.clone());
        hashgraph.insert(hash7.clone(), event7.clone());
        let actual = hashgraph.events_parents_can_see(&hash7).unwrap();
        let expected: HashMap<Vec<u8>, EventHash> =
            [(vec![2], hash3.clone()), (vec![1], hash4.clone())]
                .iter()
                .cloned()
                .collect();
        assert_eq!(expected, actual);
    }
}
