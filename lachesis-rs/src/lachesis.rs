use crate::errors::{
    HashgraphError, HashgraphErrorType, ResourceFramesPoisonError, ResourceHashgraphPoisonError,
    ResourceHeadPoisonError,
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
use std::collections::{HashMap, HashSet};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;

pub mod frame;
pub mod opera;
pub mod parents_list;

use self::frame::Frame;
use self::opera::OperaWire;
use self::parents_list::ParentsList;

const H: usize = 3;

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
            let (current_frame, previous_frame): (Frame, Frame) =
                self.get_frame_and_previous_frame(current_frame_id)?;
            for root in previous_frame.root_set.iter() {
                let seen_by = self.get_how_many_can_see(&current_frame, root)?;
                if seen_by > self.network.len() / 3 {
                    self.set_clotho(root)?;
                    self.set_clotho_time(root, current_frame_id)?;
                }
            }
        }
        Ok(())
    }

    fn get_how_many_can_see(
        &self,
        current_frame: &Frame,
        root: &EventHash,
    ) -> Result<usize, Error> {
        let opera = get_from_mutex!(self.opera, ResourceHashgraphPoisonError)?;
        let mut error: Option<Error> = None;

        let count = current_frame
            .root_set
            .iter()
            .map(|eh| match opera.can_see(&*eh, root) {
                Ok(seen) => Some(seen),
                Err(e) => {
                    error = Some(e);
                    None
                }
            })
            .filter(|eh| eh.is_some())
            .map(|eh| eh.unwrap())
            .count();
        if error.is_some() {
            return Err(error.unwrap());
        }
        Ok(count)
    }

    fn set_clotho(&self, root: &EventHash) -> Result<(), Error> {
        let mut opera = get_from_mutex!(self.opera, ResourceHashgraphPoisonError)?;
        opera.set_clotho(root)?;
        Ok(())
    }

    fn get_frame_and_previous_frame(
        &self,
        current_frame_id: usize,
    ) -> Result<(Frame, Frame), Error> {
        let frames = get_from_mutex!(self.frames, ResourceFramesPoisonError)?;
        let current_frame = &frames[current_frame_id];
        let previous_frame = &frames[current_frame_id - 1];
        Ok((current_frame.clone(), previous_frame.clone()))
    }

    fn set_clotho_time(&self, hash: &EventHash, current_frame_id: usize) -> Result<(), Error> {
        let mut frames = get_from_mutex!(self.frames, ResourceFramesPoisonError)?;
        let frame = &mut frames[current_frame_id - 1];
        let current_frame = self.current_frame.load(Ordering::Relaxed);
        let cloth_frame_id = frame.id();
        for d in 3..(current_frame - cloth_frame_id) {
            let previous_frame = cloth_frame_id - d;
            for root in frame.root_set.clone().iter() {
                if d == 3 {
                    self.set_clotho_time_from_event(root, frame)?;
                } else {
                    self.set_clotho_from_reslection(hash, root, previous_frame, d)?;
                }
            }
        }
        Ok(())
    }

    fn set_clotho_time_from_event(&self, root: &EventHash, frame: &mut Frame) -> Result<(), Error> {
        let opera = get_from_mutex!(self.opera, ResourceHashgraphPoisonError)?;
        let event = opera.get_event(root)?;
        frame.set_clotho_time(root.clone(), event.lamport_timestamp);
        Ok(())
    }

    fn set_clotho_from_reslection(
        &self,
        hash: &EventHash,
        root: &EventHash,
        previous_frame: usize,
        d: usize,
    ) -> Result<(), Error> {
        let mut frames = get_from_mutex!(self.frames, ResourceFramesPoisonError)?;
        let frame: &mut Frame = &mut frames[previous_frame];
        let t = self.clotho_time_reselection(frame.root_set.clone())?;
        let mut opera = get_from_mutex!(self.opera, ResourceHashgraphPoisonError)?;

        let mut error: Option<Error> = None;

        let k = frame
            .root_set
            .iter()
            .map(|h| match opera.get_event(h) {
                Ok(event) => Some(event.lamport_timestamp),
                Err(e) => {
                    error = Some(e);
                    None
                }
            })
            .filter(|t1| t1.is_some() && t == t1.unwrap())
            .count();
        if error.is_some() {
            return Err(error.unwrap());
        } else if d % H > 0 {
            if k > self.network.len() * 2 / 3 {
                opera.set_consensus_time(hash, t)?;
            }
            frame.set_clotho_time(root.clone(), t);
        } else {
            let t = frame
                .root_set
                .iter()
                .map(|h: &EventHash| -> Option<usize> {
                    match opera.get_event(h) {
                        Ok(event) => Some(event.lamport_timestamp),
                        Err(e) => {
                            error = Some(e);
                            None
                        }
                    }
                })
                .filter(|h: &Option<usize>| h.is_some())
                .map(|h: Option<usize>| h.unwrap())
                .min()
                .ok_or(Error::from(HashgraphError::new(
                    HashgraphErrorType::NoLamportTimeSet,
                )))?;
            frame.set_clotho_time(root.clone(), t);
        }
        if error.is_some() {
            return Err(error.unwrap());
        }
        Ok(())
    }

    fn clotho_time_reselection(&self, root_set: HashSet<EventHash>) -> Result<usize, Error> {
        let mut counts: HashMap<usize, usize> = HashMap::with_capacity(root_set.len());
        let opera = get_from_mutex!(self.opera, ResourceHashgraphPoisonError)?;
        for r in root_set {
            let event = opera.get_event(&r)?;
            let count = counts
                .get(&event.lamport_timestamp)
                .map(|v: &usize| v.clone())
                .unwrap_or(0);
            counts.insert(event.lamport_timestamp, count + 1);
        }
        let max_count = counts
            .values()
            .min()
            .ok_or(Error::from(HashgraphError::new(
                HashgraphErrorType::NoLamportTimeSet,
            )))?
            .clone();
        let time = counts
            .iter()
            .filter(|(_t, c)| c.clone() == &max_count)
            .map(|(t, _c)| t.clone())
            .min()
            .ok_or(Error::from(HashgraphError::new(
                HashgraphErrorType::NoLamportTimeSet,
            )))?;
        Ok(time)
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
    type P = ParentsList;
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

    fn add_transaction(&self, _msg: Vec<u8>) -> Result<(), Error> {
        Ok(())
    }

    fn get_ordered_events(&self) -> Result<Vec<Event<ParentsList>>, Error> {
        Ok(Vec::new())
    }
}
