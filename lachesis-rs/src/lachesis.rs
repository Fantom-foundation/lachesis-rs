use crate::errors::{ResourceHashgraphPoisonError, ResourceHeadPoisonError};
use crate::event::event_hash::EventHash;
use crate::event::Event;
use crate::lachesis::opera::Opera;
use crate::node::Node;
use crate::peer::{Peer, PeerId};
use failure::Error;
use rand::prelude::IteratorRandom;
use rand::Rng;
use ring::signature::Ed25519KeyPair;
use std::collections::HashMap;
use std::sync::Mutex;

pub mod opera;
pub mod parents_list;
use self::opera::OperaWire;
use self::parents_list::ParentsList;

pub struct Lachesis<P: Peer<Opera> + Clone> {
    head: Mutex<Option<EventHash>>,
    k: usize,
    network: HashMap<PeerId, P>,
    opera: Mutex<Opera>,
    pk: Ed25519KeyPair,
}

impl<P: Peer<Opera> + Clone> Lachesis<P> {
    pub fn new(k: usize, pk: Ed25519KeyPair) -> Lachesis<P> {
        let network = HashMap::new();
        let opera = Mutex::new(Opera::new());
        let head = Mutex::new(None);
        Lachesis {
            head,
            k,
            network,
            opera,
            pk,
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
        let peer_id = self.pk.public_key_bytes().to_vec();
        for p in peers {
            let (h, new_events) = p.get_sync(peer_id.clone(), Some(&opera));
            opera.sync(new_events);
            parent_hashes.push(h);
        }
        let parents = ParentsList(parent_hashes);
        let new_head = Event::new(vec![], Some(parents), peer_id.clone());
        let new_head_hash = new_head.hash()?;
        let mut head = get_from_mutex!(self.head, ResourceHeadPoisonError)?;
        *head = Some(new_head_hash.clone());
        opera.insert(new_head_hash.clone(), new_head);
        Ok(())
    }

    fn respond_message(&self, known: Option<OperaWire>) -> Result<(EventHash, OperaWire), Error> {
        let mut opera = get_from_mutex!(self.opera, ResourceHashgraphPoisonError)?;
        let head = get_from_mutex!(self.head, ResourceHeadPoisonError)?;
        let resp = match known {
            Some(remote) => {
                if remote.lamport_timestamp > opera.lamport_timestamp {
                    opera.set_lamport(remote.lamport_timestamp);
                }
                opera.diff(remote)
            }
            None => opera.wire(),
        };
        Ok((head.clone().unwrap(), resp))
    }
}
