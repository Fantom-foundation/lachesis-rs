use crate::errors::{
    ResourceFramesPoisonError, ResourceHashgraphPoisonError, ResourceHeadPoisonError,
};
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
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;

pub mod frame;
pub mod opera;
pub mod parents_list;

use self::frame::Frame;
use self::opera::OperaWire;
use self::parents_list::ParentsList;

pub struct Lachesis<P: Peer<Opera> + Clone> {
    current_frame: AtomicUsize,
    frames: Mutex<Vec<Frame>>,
    head: Mutex<Option<EventHash>>,
    k: usize,
    network: HashMap<PeerId, P>,
    opera: Mutex<Opera>,
    pk: Ed25519KeyPair,
}

impl<P: Peer<Opera> + Clone> Lachesis<P> {
    pub fn new(k: usize, pk: Ed25519KeyPair) -> Lachesis<P> {
        let frame = Frame::new(0);
        let current_frame = AtomicUsize::new(frame.id());
        let frames = Mutex::new(vec![frame]);
        let network = HashMap::new();
        let opera = Mutex::new(Opera::new());
        let head = Mutex::new(None);
        Lachesis {
            current_frame,
            frames,
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

    fn sync<R: Rng>(&self, rng: &mut R) -> Result<(), Error> {
        let peers = self.select_peers(rng)?;
        let mut opera = get_from_mutex!(self.opera, ResourceHashgraphPoisonError)?;
        let mut parent_hashes = vec![];
        let peer_id = self.pk.public_key_bytes().to_vec();
        for p in peers {
            let (h, new_events) = p.get_sync(peer_id.clone(), Some(&opera))?;
            opera.sync(new_events);
            parent_hashes.push(h);
        }
        let parents = ParentsList(parent_hashes);
        let new_head = Event::new(vec![], Some(parents), peer_id.clone());
        let new_head_hash = new_head.hash()?;
        let mut head = get_from_mutex!(self.head, ResourceHeadPoisonError)?;
        *head = Some(new_head_hash.clone());
        opera.insert(
            new_head_hash.clone(),
            new_head,
            self.current_frame.load(Ordering::Relaxed),
        )?;
        Ok(())
    }

    fn root_selection(&self) -> Result<(), Error> {
        let new_frame = self.assign_new_roots()?;
        self.maybe_create_new_frame(new_frame)?;
        Ok(())
    }

    fn clotho_selection(&self) -> Result<(), Error> {
        let current_frame_id = self.current_frame.load(Ordering::Relaxed);
        if current_frame_id > 0 {
            let mut opera = get_from_mutex!(self.opera, ResourceHashgraphPoisonError)?;
            let frames = get_from_mutex!(self.frames, ResourceFramesPoisonError)?;
            let current_frame: &Frame = &frames[current_frame_id];
            let previous_frame: &Frame = &frames[current_frame_id - 1];
            for root in previous_frame.root_set.iter() {
                let seen_by = current_frame
                    .root_set
                    .iter()
                    .map(|eh| match opera.can_see(&*eh, root) {
                        Ok(c) => Some(c),
                        Err(e) => {
                            debug!(target: "swirlds", "{}", e);
                            return None;
                        }
                    })
                    .filter(|eh| eh.is_some())
                    .count();
                if seen_by > self.network.len() / 3 {
                    opera.set_clotho(root)?;
                }
            }
        }
        Ok(())
    }

    fn assign_new_roots(&self) -> Result<Vec<EventHash>, Error> {
        let mut opera = get_from_mutex!(self.opera, ResourceHashgraphPoisonError)?;
        let mut new_root = vec![];
        let mut new_frame = vec![];
        for e in opera.unfamous_events().clone() {
            let is_root =
                e.flag_table.is_empty() || e.flag_table.len() > 2 / 3 * self.network.len();
            if is_root {
                let hash = e.event.hash()?;
                new_root.push(hash.clone());
                if !e.flag_table.is_empty() {
                    new_frame.push(hash);
                }
            }
        }
        for h in new_root {
            opera.set_root(&h)?;
        }
        Ok(new_frame)
    }

    fn maybe_create_new_frame(&self, new_frame: Vec<EventHash>) -> Result<(), Error> {
        let mut opera = get_from_mutex!(self.opera, ResourceHashgraphPoisonError)?;
        if !new_frame.is_empty() {
            let mut new_current_frame = Frame::new(self.current_frame.load(Ordering::Relaxed) + 1);
            let new_current_frame_id = new_current_frame.id();
            self.current_frame
                .store(new_current_frame_id, Ordering::Relaxed);
            for h in new_frame {
                opera.change_frame(&h, new_current_frame_id)?;
                new_current_frame.add(h);
            }
            let mut frames = get_from_mutex!(self.frames, ResourceFramesPoisonError)?;
            frames.push(new_current_frame);
        }
        Ok(())
    }
}

impl<P: Peer<Opera> + Clone> Node for Lachesis<P> {
    type D = OperaWire;
    fn run<R: Rng>(&self, rng: &mut R) -> Result<(), Error> {
        self.sync(rng)?;
        self.root_selection()?;
        self.clotho_selection()?;
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
        match head.clone() {
            Some(cloned_head) => Ok((cloned_head, resp)),
            None => Err(format_err!("head.clone() returned None")),
        }
    }
}
