use super::opera::{Opera, OperaWire};
use errors::{ResourceHashgraphPoisonError, ResourceHeadPoisonError};
use event::{Event, EventHash};
use failure::Error;
use lachesis::parents_list::ParentsList;
use node::Node;
use peer::{Peer, PeerId};
use rand::prelude::IteratorRandom;
use rand::Rng;
use std::collections::HashMap;
use std::sync::Mutex;

pub struct Lachesis<P: Peer<Opera> + Clone> {
    head: Mutex<Option<EventHash>>,
    k: usize,
    network: HashMap<PeerId, P>,
    opera: Mutex<Opera>,
}

impl<P: Peer<Opera> + Clone> Lachesis<P> {
    pub fn new(k: usize) -> Lachesis<P> {
        let network = HashMap::new();
        let opera = Mutex::new(Opera::new());
        let head = Mutex::new(None);
        Lachesis {
            head,
            k,
            network,
            opera,
        }
    }

    pub fn add_peer(&mut self, p: P) {
        self.network.insert(p.id().clone(), p);
    }

    #[inline]
    fn select_peers<R: Rng>(&self, rng: &mut R) -> Result<Vec<P>, Error> {
        Ok(self
            .network
            .values()
            .choose_multiple(rng, self.k - 1)
            .into_iter()
            .map(|p| p.clone())
            .collect())
    }
}

impl<P: Peer<Opera> + Clone> Node for Lachesis<P> {
    type D = OperaWire;
    fn run<R: Rng>(&self, rng: &mut R) -> Result<(), Error> {
        let peers = self.select_peers(rng)?;
        let mut opera = get_from_mutex!(self.opera, ResourceHashgraphPoisonError)?;
        let mut parent_hashes = vec![];
        for p in peers {
            let (h, new_events) = p.get_sync(vec![], Some(&opera));
            opera.sync(new_events);
            parent_hashes.push(h);
        }
        let parents = ParentsList(parent_hashes);
        let new_head = Event::new(vec![], Some(parents), vec![]);
        let mut head = get_from_mutex!(self.head, ResourceHeadPoisonError)?;
        *head = Some(new_head.hash()?);
        Ok(())
    }

    fn respond_message(&self) -> Result<(EventHash, OperaWire), Error> {
        let opera = get_from_mutex!(self.opera, ResourceHashgraphPoisonError)?;
        let head = get_from_mutex!(self.head, ResourceHeadPoisonError)?;
        Ok((head.clone().unwrap(), opera.wire()))
    }
}
