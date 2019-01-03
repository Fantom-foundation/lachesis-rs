use errors::ResourceHashgraphPoisonError;
use event::EventHash;
use failure::Error;
use node::Node;
use peer::{Peer, PeerId};
use rand::Rng;
use rand::prelude::IteratorRandom;
use std::collections::HashMap;
use std::sync::Mutex;
use super::opera::{Opera, OperaWire};

pub struct Lachesis<P: Peer<Opera> + Clone> {
    head: Option<EventHash>,
    k: usize,
    network: HashMap<PeerId, P>,
    opera: Mutex<Opera>,
}

impl<P: Peer<Opera> + Clone> Lachesis<P> {
    pub fn new(k: usize) -> Lachesis<P> {
        let network = HashMap::new();
        let opera = Mutex::new(Opera::new());
        Lachesis {
            head: None,
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
        Ok(self.network
            .values()
            .choose_multiple(rng, self.k-1)
            .into_iter()
            .map(|p| p.clone())
            .collect()
        )
    }
}

impl<P: Peer<Opera> + Clone> Node for Lachesis<P> {
    type D = OperaWire;
    fn run<R: Rng>(&self, rng: &mut R) -> Result<(), Error> {
        let peers = self.select_peers(rng)?;
        let mut opera = get_from_mutex!(self.opera, ResourceHashgraphPoisonError)?;
        for p in peers {
            let (_h, new_events) = p.get_sync(vec![], Some(&opera));
            opera.sync(new_events);
        }
        Ok(())
    }

    fn respond_message(&self) -> Result<(EventHash, OperaWire), Error> {
        let opera = get_from_mutex!(self.opera, ResourceHashgraphPoisonError)?;
        Ok((self.head.clone().unwrap(), opera.wire()))
    }
}
