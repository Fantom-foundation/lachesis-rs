use errors::HashgraphError;
use event::{Event, EventHash, Parents};
use failure::Error;
use peer::PeerId;
use std::collections::{BTreeMap, HashMap};
use std::iter::repeat_with;

#[derive(Clone)]
pub struct Hashgraph(BTreeMap<EventHash, Event>);

impl Hashgraph {
    pub fn new() -> Hashgraph {
        Hashgraph(BTreeMap::new())
    }

    pub fn get_mut(&mut self, id: &EventHash) -> Result<&mut Event, Error> {
        self.0.get_mut(id).ok_or(Error::from(HashgraphError::EventNotFound))
    }

    pub fn get(&self, id: &EventHash) -> Result<&Event, Error> {
        self.0.get(id).ok_or(Error::from(HashgraphError::EventNotFound))
    }

    pub fn insert(&mut self, hash: EventHash, event: Event) {
        self.0.insert(hash, event);
    }

    pub fn extract(&mut self, id: &EventHash) -> Result<Event, Error> {
        self.0.remove(id).ok_or(Error::from(HashgraphError::EventNotFound))
    }

    pub fn ancestors<'a>(&'a self, id: &'a EventHash) -> Vec<&'a EventHash> {
        let mut other_ancestors = self.other_ancestors(id);
        let self_ancestors = self.self_ancestors(id);
        other_ancestors.extend(self_ancestors.into_iter());
        other_ancestors
    }

    pub fn other_ancestors<'a>(&'a self, id: &'a EventHash) -> Vec<&'a EventHash> {
        let mut prev = Some(id);
        repeat_with(|| {
            if let Some(previous) = prev {
                let send = Some(previous);
                let event = self.get(previous).unwrap(); // TODO: Properly send this error
                prev = match event.parents() {
                    Some(Parents(_, other_parent)) => Some(other_parent),
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

    pub fn self_ancestors<'a>(&'a self, id: &'a EventHash) -> Vec<&'a EventHash> {
        let mut prev = Some(id);
        repeat_with(|| {
            if let Some(previous) = prev {
                let send = Some(previous);
                let event = self.get(previous).unwrap(); // TODO: Properly send this error
                prev = match event.parents() {
                    Some(Parents(self_parent, _)) => Some(self_parent),
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
    pub fn higher(&self, a: &EventHash, b: &EventHash) -> bool {
        let a_self_ancestors = self.self_ancestors(a);
        let b_ancesotrs = self.self_ancestors(b);
        if a_self_ancestors.contains(&b) {
            return true
        }
        if b_ancesotrs.contains(&a) {
            return false
        }
        a_self_ancestors.len() > b_ancesotrs.len()
    }

    #[inline]
    pub fn events_parents_can_see(&self, hash: &EventHash) -> Result<HashMap<PeerId, EventHash>, Error> {
        match self.get(hash)?.parents() {
            Some(Parents(self_parent, other_parent)) => {
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
            },
            None => Ok(HashMap::new()),
        }
    }

    pub fn difference(&self, g: &Hashgraph) -> Vec<EventHash> {
        self.0
            .keys()
            .filter(|e| !g.0.contains_key(e))
            .map(|e| (*e).clone())
            .collect()
    }

    pub fn is_valid_event(&self, event: &Event) -> Result<bool, Error> {
        match event.parents() {
            Some(Parents(self_parent, other_parent)) => {
                Ok(self.0.contains_key(self_parent) &&
                    self.0.contains_key(other_parent) &&
                    self.0[self_parent].creator() == event.creator() &&
                    self.0[other_parent].creator() != event.creator())
            },
            None => Ok(true),
        }
    }
}

#[cfg(test)]
mod tests {
    use event::{Event, Parents};
    use super::Hashgraph;

    #[test]
    fn it_should_succeed_on_event_with_no_parents() {
        let mut hashgraph = Hashgraph::new();
        let event = Event::new(vec![], None, Vec::new());
        let hash = event.hash().unwrap();
        hashgraph.insert(hash.clone(), event.clone());
        assert!(hashgraph.is_valid_event(&event).unwrap());
    }

    #[test]
    fn it_should_succeed_on_event_with_correct_parents() {
        let mut hashgraph = Hashgraph::new();
        let n1 = vec![42];
        let n2 = vec![43];
        let self_parent = Event::new(vec![], None, n1.clone());
        let other_parent = Event::new(vec![], None, n2);
        let sphash = self_parent.hash().unwrap();
        let ophash = other_parent.hash().unwrap();
        let event = Event::new(vec![], Some(Parents(sphash, ophash)), n1);
        let hash = event.hash().unwrap();
        hashgraph.insert(ophash.clone(), other_parent);
        hashgraph.insert(sphash.clone(), self_parent);
        hashgraph.insert(hash.clone(), event.clone());
        assert!(hashgraph.is_valid_event(&event).unwrap());
    }

    #[test]
    fn it_should_fail_if_self_parent_creator_differs() {
        let mut hashgraph = Hashgraph::new();
        let n1 = vec![42];
        let n2 = vec![43];
        let n3 = vec![44];
        let self_parent = Event::new(vec![], None, n1);
        let other_parent = Event::new(vec![], None, n2);
        let sphash = self_parent.hash().unwrap();
        let ophash = other_parent.hash().unwrap();
        let event = Event::new(vec![], Some(Parents(sphash, ophash)), n3);
        let hash = event.hash().unwrap();
        hashgraph.insert(ophash.clone(), other_parent);
        hashgraph.insert(sphash.clone(), self_parent);
        hashgraph.insert(hash.clone(), event.clone());
        assert!(!hashgraph.is_valid_event(&event).unwrap());
    }

    #[test]
    fn it_should_fail_if_other_parent_its_sent_by_same_node() {
        let mut hashgraph = Hashgraph::new();
        let n1 = vec![42];
        let n2 = vec![43];
        let self_parent = Event::new(vec![], None, n1);
        let other_parent = Event::new(vec![], None, n2.clone());
        let sphash = self_parent.hash().unwrap();
        let ophash = other_parent.hash().unwrap();
        let event = Event::new(vec![], Some(Parents(sphash, ophash)), n2.clone());
        let hash = event.hash().unwrap();
        hashgraph.insert(ophash.clone(), other_parent);
        hashgraph.insert(sphash.clone(), self_parent);
        hashgraph.insert(hash.clone(), event.clone());
        assert!(!hashgraph.is_valid_event(&event).unwrap());
    }

    #[test]
    fn it_should_fail_if_self_parent_isnt_in_the_graph() {
        let mut hashgraph = Hashgraph::new();
        let n1 = vec![42];
        let n2 = vec![43];
        let self_parent = Event::new(vec![], None, n1);
        let other_parent = Event::new(vec![], None, n2.clone());
        let sphash = self_parent.hash().unwrap();
        let ophash = other_parent.hash().unwrap();
        let event = Event::new(vec![], Some(Parents(sphash, ophash)), n2.clone());
        let hash = event.hash().unwrap();
        hashgraph.insert(ophash.clone(), other_parent);
        hashgraph.insert(hash.clone(), event.clone());
        assert!(!hashgraph.is_valid_event(&event).unwrap());
    }

    #[test]
    fn it_should_fail_if_other_parent_isnt_in_the_graph() {
        let mut hashgraph = Hashgraph::new();
        let n1 = vec![42];
        let n2 = vec![43];
        let self_parent = Event::new(vec![], None, n1);
        let other_parent = Event::new(vec![], None, n2.clone());
        let sphash = self_parent.hash().unwrap();
        let ophash = other_parent.hash().unwrap();
        let event = Event::new(vec![], Some(Parents(sphash, ophash)), n2.clone());
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
        let mut hg1 = Hashgraph::new();
        let mut hg2 = Hashgraph::new();
        hg1.insert(hash1.clone(), event1);
        hg1.insert(hash2.clone(), event2);
        hg2.insert(hash3.clone(), event3);
        let mut expected = vec![hash1.clone(), hash2.clone()];
        expected.sort();
        let mut actual = hg1.difference(&hg2);
        actual.sort();
        assert_eq!(expected, actual)
    }
}
