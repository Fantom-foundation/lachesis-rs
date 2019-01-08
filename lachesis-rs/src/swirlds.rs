use crate::errors::*;
use crate::event::{
    event_hash::EventHash, event_signature::EventSignature, parents::ParentsPair, Event,
};
use crate::hashgraph::{Hashgraph, HashgraphWire};
use crate::node::Node;
use crate::peer::{Peer, PeerId};
use crate::printable_hash::PrintableHash;
use crate::round::Round;
use failure::Error;
use rand::prelude::IteratorRandom;
use rand::Rng;
use ring::signature;
use std::cmp::Ordering;
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::fmt;
use std::iter::FromIterator;
use std::marker::PhantomData;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

const C: usize = 6;

#[inline]
fn get_current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went back")
        .as_secs()
}

#[inline]
fn assign_round(event: &mut Event<ParentsPair>, round: usize) -> Result<usize, Error> {
    event.set_round(round);
    Ok(round)
}

#[inline]
fn get_round_pairs(r: &Round) -> Vec<(usize, EventHash)> {
    r.witnesses().iter().map(|w| (r.id, w.clone())).collect()
}

struct NodeInternalState<P: Peer<H>, H: Hashgraph> {
    consensus: BTreeSet<usize>,
    network: HashMap<PeerId, Arc<Box<P>>>,
    pending_events: HashSet<EventHash>,
    rounds: Vec<Round>,
    super_majority: usize,
    votes: HashMap<(EventHash, EventHash), bool>,
    _phantom: PhantomData<H>,
}

impl<P: Peer<H>, H: Hashgraph + Clone + fmt::Debug> fmt::Debug for Swirlds<P, H> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fn print_arrows(f: &mut fmt::Formatter, n_nodes: usize) -> fmt::Result {
            for _ in 0..3 {
                write!(f, "        ")?;
                for _ in 0..n_nodes {
                    write!(f, "    |     ")?;
                }
                writeln!(f, "")?;
            }
            Ok(())
        }
        fn update_last_events<H: Hashgraph>(
            events: &mut BTreeMap<PeerId, Option<EventHash>>,
            h: &H,
        ) {
            for (k, v) in events.clone() {
                if let Some(v) = v {
                    let self_child = h.find_self_child(&v);
                    events.insert(k.clone(), self_child);
                }
            }
        }
        fn print_hashes(
            f: &mut fmt::Formatter,
            events: &mut BTreeMap<PeerId, Option<EventHash>>,
        ) -> fmt::Result {
            write!(f, "        ")?;
            for peer in events.keys() {
                if let Some(Some(ev)) = events.get(peer) {
                    write!(f, "{}  ", ev.printable_hash())?;
                } else {
                    write!(f, "          ")?;
                }
            }
            writeln!(f, "")?;
            Ok(())
        }
        fn print_other_parents<H: Hashgraph>(
            f: &mut fmt::Formatter,
            events: &mut BTreeMap<PeerId, Option<EventHash>>,
            h: &H,
        ) -> fmt::Result {
            write!(f, "        ")?;
            for peer in events.keys() {
                if let Some(Some(ev)) = events.get(peer) {
                    let ev = h.get(ev).unwrap();
                    if let Some(ParentsPair(_, other_parent)) = ev.parents() {
                        write!(f, "{}  ", other_parent.printable_hash())?;
                    } else {
                        write!(f, "          ")?;
                    }
                } else {
                    write!(f, "          ")?;
                }
            }
            writeln!(f, "")?;
            Ok(())
        }
        fn print_rounds<H: Hashgraph>(
            f: &mut fmt::Formatter,
            events: &mut BTreeMap<PeerId, Option<EventHash>>,
            h: &H,
        ) -> fmt::Result {
            write!(f, "        ")?;
            for peer in events.keys() {
                if let Some(Some(ev)) = events.get(peer) {
                    let ev = h.get(ev).unwrap();
                    if let Some(round) = ev.maybe_round() {
                        let r_string = format!("{}", round);
                        let spaces = (0..(10 - r_string.len())).map(|_| " ").collect::<String>();
                        write!(f, "{}{}", round, spaces)?;
                    } else {
                        write!(f, "          ")?;
                    }
                } else {
                    write!(f, "          ")?;
                }
            }
            writeln!(f, "")?;
            Ok(())
        }
        fn num_of_some_in_map(map: &BTreeMap<PeerId, Option<EventHash>>) -> usize {
            let vs: Vec<&Option<EventHash>> = map.values().filter(|v| v.is_some()).collect();
            vs.len()
        }
        let state = get_from_mutex!(self.state, ResourceNodeInternalStatePoisonError).unwrap();
        let head = get_from_mutex!(self.head, ResourceHeadPoisonError)
            .unwrap()
            .clone();
        let network: &HashMap<PeerId, Arc<Box<P>>> = &state.network;
        let hashgraph = get_from_mutex!(self.hashgraph, ResourceHashgraphPoisonError).unwrap();
        writeln!(f, "Node ID: {:?}", self.get_id().printable_hash())?;
        writeln!(f, "Head: {:?}", head.map(|h| h.printable_hash()))?;
        let roots: Vec<EventHash> = hashgraph.find_roots();
        let mut last_event_per_peer: BTreeMap<PeerId, Option<EventHash>> = BTreeMap::new();
        for peer in network.keys() {
            for root in roots.iter() {
                let e: &Event<ParentsPair> = hashgraph.get(root).unwrap();
                if e.creator() == peer {
                    last_event_per_peer.insert(peer.clone(), Some(root.clone()));
                }
            }
            if !last_event_per_peer.contains_key(peer) {
                last_event_per_peer.insert(peer.clone(), None);
            }
        }
        for root in roots.iter() {
            let e: &Event<ParentsPair> = hashgraph.get(root).unwrap();
            if e.creator() == &self.get_id() {
                last_event_per_peer.insert(self.get_id().clone(), Some(root.clone()));
            }
        }
        if !last_event_per_peer.contains_key(&self.get_id()) {
            last_event_per_peer.insert(self.get_id().clone(), None);
        }
        write!(f, "Peers:  ")?;
        for peer in last_event_per_peer.keys() {
            write!(f, "{}  ", peer.printable_hash())?;
        }
        writeln!(f, "")?;
        write!(f, "        ")?;
        for root in last_event_per_peer.values() {
            if let Some(root) = root {
                write!(f, "{}  ", root.printable_hash())?;
            } else {
                write!(f, "          ")?;
            }
        }
        writeln!(f, "")?;
        let h = (*hashgraph).clone();
        print_rounds(f, &mut last_event_per_peer, &h)?;
        update_last_events(&mut last_event_per_peer, &h);
        while num_of_some_in_map(&last_event_per_peer) > 0 {
            print_arrows(f, network.len() + 1)?;
            print_hashes(f, &mut last_event_per_peer)?;
            print_other_parents(f, &mut last_event_per_peer, &h)?;
            print_rounds(f, &mut last_event_per_peer, &h)?;
            update_last_events(&mut last_event_per_peer, &h);
        }
        writeln!(f, "")?;
        writeln!(f, "")
    }
}

pub struct Swirlds<P: Peer<H>, H: Hashgraph + Clone + fmt::Debug> {
    hashgraph: Mutex<H>,
    head: Mutex<Option<EventHash>>,
    // TODO: Plain keys in memory? Not great. See https://stackoverflow.com/a/1263421 for possible
    // alternatives
    pk: signature::Ed25519KeyPair,
    state: Mutex<NodeInternalState<P, H>>,
}

impl<P: Peer<H>, H: Hashgraph + Clone + fmt::Debug> Swirlds<P, H> {
    pub fn new(pk: signature::Ed25519KeyPair, hashgraph: H) -> Result<Self, Error> {
        let state = Mutex::new(NodeInternalState {
            consensus: BTreeSet::new(),
            network: HashMap::new(),
            pending_events: HashSet::new(),
            rounds: Vec::new(),
            super_majority: 0,
            votes: HashMap::new(),
            _phantom: PhantomData,
        });
        let node = Swirlds {
            hashgraph: Mutex::new(hashgraph),
            head: Mutex::new(None),
            pk,
            state,
        };
        node.create_new_head(None, Some(0))?;
        Ok(node)
    }

    #[inline]
    pub fn add_node(&self, peer: Arc<Box<P>>) -> Result<(), Error> {
        let super_majority = {
            let mut state = get_from_mutex!(self.state, ResourceNodeInternalStatePoisonError)?;
            state.network.insert(peer.id().clone(), peer);
            state.network.len() * 2 / 3
        };
        self.set_super_majority(super_majority)?;
        Ok(())
    }

    pub fn sync(&self, remote_head: EventHash, remote_hg: H) -> Result<Vec<EventHash>, Error> {
        info!(
            "[Node {:?}] Syncing with head {:?}",
            self.get_id().printable_hash(),
            remote_head.printable_hash()
        );
        debug!("{:?}", self);
        let mut res = self.merge_hashgraph(remote_hg.clone())?;
        info!(
            "[Node {:?}] Merging {:?}",
            self.get_id().printable_hash(),
            res.iter()
                .map(|v| v.printable_hash())
                .collect::<Vec<String>>()
        );
        debug!("{:?}", self);

        if res.len() > 0 {
            let new_head = self.maybe_change_head(remote_head, remote_hg.clone())?;
            res.extend(new_head.into_iter());
        }
        Ok(res)
    }

    pub fn divide_rounds(&self, events: Vec<EventHash>) -> Result<(), Error> {
        for eh in events.into_iter() {
            let round = self.assign_round(&eh)?;
            info!(
                "[Node {:?}] Round {} assigned to {:?}",
                self.get_id().printable_hash(),
                round,
                eh.printable_hash()
            );
            debug!("{:?}", self);

            self.maybe_add_new_round(round)?;

            self.set_event_can_see_self(&eh)?;

            self.maybe_add_witness_to_round(round, &eh)?;
        }
        Ok(())
    }

    pub fn decide_fame(&self) -> Result<BTreeSet<usize>, Error> {
        let mut famous_events = HashMap::new();
        let mut rounds_done = BTreeSet::new();
        let super_majority = self.get_super_majority()?;
        for (round, veh) in self.get_voters()?.into_iter() {
            let witnesses = self.get_round_witnesses(round, &veh)?;
            for (ur, eh) in self.get_undetermined_events(round)? {
                if round - ur == 1 {
                    self.vote(veh.clone(), eh.clone(), witnesses.contains(&eh))?;
                } else {
                    let (vote, stake) = self.get_vote(&witnesses, &eh)?;
                    if (round - ur) % C != 1 {
                        if stake > super_majority {
                            famous_events.insert(eh, vote);
                            rounds_done.insert(ur);
                        } else {
                            self.vote(veh.clone(), eh, vote)?;
                        }
                    } else {
                        if stake > super_majority {
                            self.vote(veh.clone(), eh, vote)?;
                        } else {
                            let hashgraph =
                                get_from_mutex!(self.hashgraph, ResourceHashgraphPoisonError)?;
                            let new_vote = hashgraph.get(&veh)?.signature()?.as_ref()[0] != 0;
                            self.vote(veh.clone(), eh, new_vote)?;
                        }
                    }
                }
            }
        }

        self.update_famous_events(famous_events)?;

        let new_consensus: BTreeSet<usize> = BTreeSet::from_iter(
            rounds_done
                .into_iter()
                .filter(|r| self.are_all_witnesses_famous(*r).unwrap()),
        );
        info!(
            "[Node {:?}] New consensus rounds: {:?}",
            self.get_id().printable_hash(),
            new_consensus
        );
        debug!("{:?}", self);

        let mut state = get_from_mutex!(self.state, ResourceNodeInternalStatePoisonError)?;
        state.consensus =
            BTreeSet::from_iter(state.consensus.union(&new_consensus).map(|r| r.clone()));

        Ok(new_consensus)
    }

    pub fn find_order(&self, new_consensus: BTreeSet<usize>) -> Result<(), Error> {
        let mut state = get_from_mutex!(self.state, ResourceNodeInternalStatePoisonError)?;
        for round in new_consensus {
            let unique_famous_witnesses = self.get_unique_famous_witnesses(round)?;
            for eh in state.pending_events.clone() {
                let is_round_received = self.is_round_received(&unique_famous_witnesses, &eh)?;
                if is_round_received {
                    self.set_received_information(&eh, round, &unique_famous_witnesses)?;
                    state.pending_events.remove(&eh);
                }
            }
        }
        Ok(())
    }

    pub fn get_id(&self) -> PeerId {
        self.pk.public_key_bytes().to_vec()
    }

    pub fn get_hashgraph(&self) -> Result<H, Error> {
        let hashgraph = get_from_mutex!(self.hashgraph, ResourceHashgraphPoisonError)?;
        Ok(hashgraph.clone())
    }

    pub fn get_head(&self) -> Result<EventHash, Error> {
        get_from_mutex!(self.head, ResourceHeadPoisonError)?
            .clone()
            .map(|v| v.clone())
            .ok_or(Error::from(NodeError::new(NodeErrorType::NoHead)))
    }

    pub fn get_peer(&self, id: &PeerId) -> Result<Arc<Box<P>>, Error> {
        let state = get_from_mutex!(self.state, ResourceNodeInternalStatePoisonError)?;
        state
            .network
            .get(id)
            .map(|v| v.clone())
            .ok_or(Error::from(NodeError::new(NodeErrorType::PeerNotFound(
                id.clone(),
            ))))
    }

    #[inline]
    fn update_famous_events(&self, famous_events: HashMap<EventHash, bool>) -> Result<(), Error> {
        let mut hashgraph = get_from_mutex!(self.hashgraph, ResourceHashgraphPoisonError)?;
        for (e, vote) in famous_events.into_iter() {
            let ev = hashgraph.get_mut(&e)?;
            ev.famous(vote);
        }
        Ok(())
    }

    #[inline]
    pub fn get_stats(&self) -> Result<(usize, usize), Error> {
        let state = get_from_mutex!(self.state, ResourceNodeInternalStatePoisonError)?;
        Ok((state.rounds.len(), state.pending_events.len()))
    }

    #[inline]
    fn set_super_majority(&self, sm: usize) -> Result<(), Error> {
        let mut state = get_from_mutex!(self.state, ResourceNodeInternalStatePoisonError)?;
        state.super_majority = sm;
        Ok(())
    }

    #[inline]
    fn get_super_majority(&self) -> Result<usize, Error> {
        let state = get_from_mutex!(self.state, ResourceNodeInternalStatePoisonError)?;
        Ok(state.super_majority)
    }

    #[inline]
    fn vote(&self, veh: EventHash, eh: EventHash, vote: bool) -> Result<(), Error> {
        let mut state = get_from_mutex!(self.state, ResourceNodeInternalStatePoisonError)?;
        state.votes.insert((veh, eh), vote);
        Ok(())
    }

    #[inline]
    fn maybe_add_witness_to_round(&self, round: usize, eh: &EventHash) -> Result<(), Error> {
        let hashgraph = get_from_mutex!(self.hashgraph, ResourceHashgraphPoisonError)?;
        let event = hashgraph.get(&eh)?;
        if round == 0 || round > hashgraph.get(&event.self_parent()?)?.round()? {
            let creator = event.creator().clone();
            self.add_witness_to_round(round, creator, eh)?;
        }
        Ok(())
    }

    #[inline]
    fn add_witness_to_round(
        &self,
        round: usize,
        creator: PeerId,
        eh: &EventHash,
    ) -> Result<(), Error> {
        let mut state = get_from_mutex!(self.state, ResourceNodeInternalStatePoisonError)?;
        state.rounds[round].add_witness(creator, eh.clone());
        Ok(())
    }

    #[inline]
    fn maybe_add_new_round(&self, round: usize) -> Result<(), Error> {
        let mut state = get_from_mutex!(self.state, ResourceNodeInternalStatePoisonError)?;
        if state.rounds.len() == round {
            state.rounds.push(Round::new(round));
        }
        Ok(())
    }

    #[inline]
    fn is_round_received(
        &self,
        unique_famous_witnesses: &HashSet<EventHash>,
        eh: &EventHash,
    ) -> Result<bool, Error> {
        let hashgraph = get_from_mutex!(self.hashgraph, ResourceHashgraphPoisonError)?;
        Ok(unique_famous_witnesses
            .iter()
            .all(|ufwh| hashgraph.ancestors(ufwh).contains(&eh)))
    }

    #[inline]
    fn set_received_information(
        &self,
        hash: &EventHash,
        round: usize,
        unique_famous_witnesses: &HashSet<EventHash>,
    ) -> Result<(), Error> {
        let timestamp_deciders = self.get_timestamp_deciders(hash, unique_famous_witnesses)?;
        let mut hashgraph = get_from_mutex!(self.hashgraph, ResourceHashgraphPoisonError)?;
        let times = timestamp_deciders
            .into_iter()
            .map(|eh| hashgraph.get(&eh).unwrap().timestamp().unwrap())
            .collect::<Vec<u64>>();
        let times_sum: u64 = times.iter().sum();
        let new_time = times_sum / times.len() as u64;
        let event = hashgraph.get_mut(hash)?;
        event.set_timestamp(new_time);
        event.set_round_received(round);
        Ok(())
    }

    #[inline]
    fn get_timestamp_deciders(
        &self,
        hash: &EventHash,
        unique_famous_witnesses: &HashSet<EventHash>,
    ) -> Result<HashSet<EventHash>, Error> {
        let mut result = HashSet::new();
        let hashgraph = get_from_mutex!(self.hashgraph, ResourceHashgraphPoisonError)?;
        for unique_famous_witness in unique_famous_witnesses {
            let self_ancestors = hashgraph.self_ancestors(unique_famous_witness).into_iter();
            for self_ancestor in self_ancestors {
                let ancestors = hashgraph.ancestors(self_ancestor);
                let event = hashgraph.get(self_ancestor)?;
                if ancestors.contains(&hash) && !event.is_self_parent(hash) {
                    result.insert(self_ancestor.clone());
                }
            }
        }
        Ok(result)
    }

    #[inline]
    fn get_unique_famous_witnesses(&self, round: usize) -> Result<HashSet<EventHash>, Error> {
        let mut famous_witnesses = self.get_famous_witnesses(round)?;
        let hashgraph = get_from_mutex!(self.hashgraph, ResourceHashgraphPoisonError)?;
        for w in famous_witnesses.clone() {
            for w1 in famous_witnesses.clone() {
                if w != w1 {
                    let e = hashgraph.get(&w)?;
                    let e1 = hashgraph.get(&w1)?;
                    if e.parents() == e1.parents() {
                        famous_witnesses.remove(&w);
                    }
                }
            }
        }
        Ok(famous_witnesses)
    }

    #[inline]
    fn get_famous_witnesses(&self, round: usize) -> Result<HashSet<EventHash>, Error> {
        let hashgraph = get_from_mutex!(self.hashgraph, ResourceHashgraphPoisonError)?;
        let state = get_from_mutex!(self.state, ResourceNodeInternalStatePoisonError)?;
        Ok(HashSet::from_iter(
            state.rounds[round]
                .witnesses()
                .into_iter()
                .filter(|eh| hashgraph.get(eh).unwrap().is_famous()),
        ))
    }

    #[inline]
    fn are_all_witnesses_famous(&self, round: usize) -> Result<bool, Error> {
        let hashgraph = get_from_mutex!(self.hashgraph, ResourceHashgraphPoisonError)?;
        let state = get_from_mutex!(self.state, ResourceNodeInternalStatePoisonError)?;
        Ok(state.rounds[round]
            .witnesses()
            .iter()
            .map(|eh| hashgraph.get(eh).unwrap())
            .all(|e| e.is_famous()))
    }

    #[inline]
    fn get_vote(
        &self,
        witnesses: &HashSet<EventHash>,
        eh: &EventHash,
    ) -> Result<(bool, usize), Error> {
        let total = self.get_votes_for_event(witnesses, eh)?;
        if total > witnesses.len() / 2 {
            Ok((true, total))
        } else {
            Ok((false, witnesses.len() - total))
        }
    }

    #[inline]
    fn get_votes_for_event(
        &self,
        witnesses: &HashSet<EventHash>,
        eh: &EventHash,
    ) -> Result<usize, Error> {
        let state = get_from_mutex!(self.state, ResourceNodeInternalStatePoisonError)?;
        let mut total = 0;
        for w in witnesses {
            if state.votes[&(w.clone(), eh.clone())] {
                total += 1;
            }
        }
        Ok(total)
    }

    #[inline]
    fn get_undetermined_events(&self, round: usize) -> Result<Vec<(usize, EventHash)>, Error> {
        let next_consensus = self.get_next_consensus()?;
        let hashgraph = get_from_mutex!(self.hashgraph, ResourceHashgraphPoisonError)?;
        let state = get_from_mutex!(self.state, ResourceNodeInternalStatePoisonError)?;
        Ok((next_consensus..round)
            .filter(|r| !state.consensus.contains(r))
            .map(|r| get_round_pairs(&state.rounds[r]).into_iter())
            .flatten()
            .filter(|(_, h)| hashgraph.get(&h).unwrap().is_undefined())
            .collect::<Vec<(usize, EventHash)>>())
    }

    #[inline]
    fn get_round_witnesses(
        &self,
        round: usize,
        hash: &EventHash,
    ) -> Result<HashSet<EventHash>, Error> {
        if round == 0 {
            Ok(HashSet::new())
        } else {
            let hits = self.get_round_hits(round, hash)?;
            let prev_round = round - 1;
            let super_majority = self.get_super_majority()?;
            let state = get_from_mutex!(self.state, ResourceNodeInternalStatePoisonError)?;
            let r = &state.rounds[prev_round];
            let map_iter = hits
                .into_iter()
                .filter(|(_, v)| *v > super_majority)
                .map(|(c, _)| r.witnesses_map()[&c].clone());
            Ok(HashSet::from_iter(map_iter))
        }
    }

    #[inline]
    fn get_round_hits(
        &self,
        round: usize,
        hash: &EventHash,
    ) -> Result<HashMap<PeerId, usize>, Error> {
        if round == 0 {
            Ok(HashMap::new())
        } else {
            let mut hits: HashMap<PeerId, usize> = HashMap::new();
            let hashgraph = get_from_mutex!(self.hashgraph, ResourceHashgraphPoisonError)?;
            let event = hashgraph.get(hash)?;
            let prev_round = round - 1;
            for (creator, event_hash) in event.can_see().iter() {
                let possible_witness = hashgraph.get(event_hash)?;
                if possible_witness.round()? == prev_round {
                    for (_creator, _event_hash) in possible_witness.can_see().iter() {
                        let r = hashgraph.get(_event_hash)?.round()?;
                        if r == prev_round {
                            let new_val = hits.get(creator).map(|v| *v + 1).unwrap_or(1);
                            hits.insert(creator.clone(), new_val);
                        }
                    }
                }
            }
            Ok(hits)
        }
    }

    #[inline]
    fn get_voters(&self) -> Result<Vec<(usize, EventHash)>, Error> {
        let next_consensus = self.get_next_consensus()?;
        let state = get_from_mutex!(self.state, ResourceNodeInternalStatePoisonError)?;
        let max_r = state.rounds.iter().map(|r| r.id).max().unwrap_or(0);
        Ok(state.rounds[next_consensus..max_r]
            .iter()
            .flat_map(|r| get_round_pairs(r))
            .collect())
    }

    #[inline]
    fn get_next_consensus(&self) -> Result<usize, Error> {
        let state = get_from_mutex!(self.state, ResourceNodeInternalStatePoisonError)?;
        Ok(state.consensus.iter().last().map(|v| *v + 1).unwrap_or(0))
    }

    #[inline]
    fn set_event_can_see_self(&self, hash: &EventHash) -> Result<(), Error> {
        let mut hashgraph = get_from_mutex!(self.hashgraph, ResourceHashgraphPoisonError)?;
        let event = hashgraph.get_mut(&hash)?;
        let creator = event.creator().clone();
        event.add_can_see(creator, hash.clone());
        Ok(())
    }

    #[inline]
    fn assign_round(&self, hash: &EventHash) -> Result<usize, Error> {
        let is_root = {
            let hashgraph = get_from_mutex!(self.hashgraph, ResourceHashgraphPoisonError)?;
            hashgraph.get(hash)?.is_root()
        };
        if is_root {
            let mut hashgraph = get_from_mutex!(self.hashgraph, ResourceHashgraphPoisonError)?;
            assign_round(hashgraph.get_mut(&hash)?, 0)
        } else {
            self.assign_non_root_round(hash)
        }
    }

    #[inline]
    fn assign_non_root_round(&self, hash: &EventHash) -> Result<usize, Error> {
        let events_parents_can_see = {
            let hashgraph = get_from_mutex!(self.hashgraph, ResourceHashgraphPoisonError)?;
            hashgraph.events_parents_can_see(hash)?
        };
        let mut r = self.get_parents_round(hash)?;
        let hits = self.get_hits_per_events(r, &events_parents_can_see)?;
        let sm = self.get_super_majority()?;
        let votes = hits.values().map(|v| v.clone()).filter(|v| *v > sm);
        if votes.sum::<usize>() > sm {
            r += 1;
        }
        self.set_events_parents_can_see(hash, events_parents_can_see)?;
        let mut hashgraph = get_from_mutex!(self.hashgraph, ResourceHashgraphPoisonError)?;
        assign_round(hashgraph.get_mut(&hash)?, r)
    }

    #[inline]
    fn get_hits_per_events(
        &self,
        r: usize,
        events_parents_can_see: &HashMap<PeerId, EventHash>,
    ) -> Result<HashMap<PeerId, usize>, Error> {
        let hashgraph = get_from_mutex!(self.hashgraph, ResourceHashgraphPoisonError)?;
        let mut hits: HashMap<PeerId, usize> = HashMap::new();
        for (_, h) in events_parents_can_see.iter() {
            let event = hashgraph.get(h)?;
            if event.round()? == r {
                for (_c, _h) in event.can_see().iter() {
                    let seen_event = hashgraph.get(_h)?;
                    if seen_event.round()? == r {
                        let prev = hits.get(_c).map(|v| v.clone()).unwrap_or(0);
                        hits.insert(_c.clone(), prev + 1);
                    }
                }
            }
        }
        Ok(hits)
    }

    #[inline]
    fn get_parents_round(&self, hash: &EventHash) -> Result<usize, Error> {
        let hashgraph = get_from_mutex!(self.hashgraph, ResourceHashgraphPoisonError)?;
        let event = hashgraph.get(hash)?;
        let parents = event.parents().clone().ok_or(Error::from(EventError::new(
            EventErrorType::NoParents { hash: hash.clone() },
        )))?;
        parents.max_round(hashgraph.clone())
    }

    #[inline]
    fn set_events_parents_can_see(
        &self,
        hash: &EventHash,
        events_parents_can_see: HashMap<Vec<u8>, EventHash>,
    ) -> Result<(), Error> {
        let mut hashgraph = get_from_mutex!(self.hashgraph, ResourceHashgraphPoisonError)?;
        let event = hashgraph.get_mut(hash)?;
        event.set_can_see(events_parents_can_see);
        Ok(())
    }

    #[inline]
    fn merge_hashgraph(&self, remote_hg: H) -> Result<Vec<EventHash>, Error> {
        let mut diff = {
            let hashgraph = get_from_mutex!(self.hashgraph, ResourceHashgraphPoisonError)?;
            remote_hg.difference(hashgraph.clone())
        };
        diff.sort_by(|h1, h2| {
            let h1_higher = remote_hg.higher(h1, h2);
            let h2_higher = remote_hg.higher(h2, h1);
            if h1_higher {
                Ordering::Greater
            } else if h2_higher {
                Ordering::Less
            } else {
                Ordering::Equal
            }
        });
        let mut res = Vec::with_capacity(diff.len());
        for eh in diff.clone().into_iter() {
            let is_valid_event = {
                let event = remote_hg.get(&eh)?;
                self.is_valid_event(&eh, event)
            }?;
            if is_valid_event {
                self.add_event(remote_hg.get(&eh)?.clone())?;
                res.push(eh);
            } else {
                warn!(
                    "[Node {:?}] Error {:?} isn't valid",
                    self.get_id().printable_hash(),
                    eh.printable_hash()
                );
            }
        }
        Ok(res)
    }

    #[inline]
    fn maybe_change_head(
        &self,
        remote_head: EventHash,
        remote_hg: H,
    ) -> Result<Option<EventHash>, Error> {
        let remote_head_event = remote_hg.get(&remote_head).unwrap().clone();

        if self.is_valid_event(&remote_head, &remote_head_event)? {
            let current_head = self.get_head()?;
            let parents = ParentsPair(current_head, remote_head);
            Ok(Some(self.create_new_head(Some(parents), None)?))
        } else {
            Ok(None)
        }
    }

    #[inline]
    fn is_valid_event(
        &self,
        event_hash: &EventHash,
        event: &Event<ParentsPair>,
    ) -> Result<bool, Error> {
        event.is_valid(event_hash).and_then(|b| {
            if !b {
                Ok(false)
            } else {
                let hashgraph = get_from_mutex!(self.hashgraph, ResourceHashgraphPoisonError)?;
                hashgraph.is_valid_event(event)
            }
        })
    }

    #[inline]
    fn select_peer<R: Rng>(&self, rng: &mut R) -> Result<Arc<Box<P>>, Error> {
        let state = get_from_mutex!(self.state, ResourceNodeInternalStatePoisonError)?;
        state
            .network
            .values()
            .choose(rng)
            .ok_or(Error::from(NodeError::new(NodeErrorType::EmptyNetwork)))
            .map(|p| p.clone())
    }

    fn create_new_head(
        &self,
        parents: Option<ParentsPair>,
        round: Option<usize>,
    ) -> Result<EventHash, Error> {
        let mut event = Event::new(Vec::new(), parents, self.pk.public_key_bytes().to_vec());
        if event.is_root() {
            event.set_timestamp(get_current_timestamp())
        }
        round.iter().for_each(|r| event.set_round(r.clone()));
        let hash = event.hash()?;
        let signature = self.pk.sign(hash.as_ref());
        event.sign(EventSignature(signature.as_ref().to_vec()));
        self.add_event(event)?;
        let mut current_head = get_from_mutex!(self.head, ResourceHeadPoisonError)?;
        *current_head = Some(hash.clone());
        Ok(hash.clone())
    }

    #[inline]
    fn add_event(&self, e: Event<ParentsPair>) -> Result<(), Error> {
        let hash = e.hash()?;
        self.add_pending_event(hash.clone())?;
        let mut hashgraph = get_from_mutex!(self.hashgraph, ResourceHashgraphPoisonError)?;
        Ok(hashgraph.insert(hash, e))
    }

    #[inline]
    fn add_pending_event(&self, e: EventHash) -> Result<(), Error> {
        let mut state = get_from_mutex!(self.state, ResourceNodeInternalStatePoisonError)?;
        state.pending_events.insert(e);
        Ok(())
    }
}

impl<P: Peer<H>, H: Hashgraph + Clone + fmt::Debug> Node for Swirlds<P, H> {
    type D = HashgraphWire;
    fn run<R: Rng>(&self, rng: &mut R) -> Result<(), Error> {
        let (head, hg) = {
            let peer = self.select_peer(rng)?;
            peer.get_sync(self.pk.public_key_bytes().to_vec(), None)
        };
        let new_events = self.sync(head, hg)?;
        self.divide_rounds(new_events)?;
        let new_consensus = self.decide_fame()?;
        self.find_order(new_consensus)?;
        Ok(())
    }

    fn respond_message(
        &self,
        _k: Option<HashgraphWire>,
    ) -> Result<(EventHash, HashgraphWire), Error> {
        let head = self.get_head()?;
        let hashgraph = self.get_hashgraph()?;
        let wire = hashgraph.wire();
        Ok((head, wire))
    }
}

#[cfg(test)]
mod tests {
    use super::Swirlds;
    use crate::event::{
        event_hash::EventHash, event_signature::EventSignature, parents::ParentsPair, Event,
    };
    use crate::hashgraph::*;
    use crate::peer::{Peer, PeerId};
    use ring::digest::{digest, SHA256};
    use ring::{rand, signature};
    use std::collections::HashSet;
    use std::iter::FromIterator;
    use std::sync::Arc;

    fn create_node() -> Swirlds<DummyPeer, BTreeHashgraph> {
        let rng = rand::SystemRandom::new();
        let pkcs8_bytes = signature::Ed25519KeyPair::generate_pkcs8(&rng).unwrap();
        let kp =
            signature::Ed25519KeyPair::from_pkcs8(untrusted::Input::from(&pkcs8_bytes)).unwrap();
        let hashgraph = BTreeHashgraph::new();
        Swirlds::new(kp, hashgraph).unwrap()
    }

    fn create_useless_peer(id: PeerId) -> Arc<Box<DummyPeer>> {
        let digest = digest(&SHA256, b"42");
        let event = EventHash(digest.as_ref().to_vec());
        Arc::new(Box::new(DummyPeer {
            hashgraph: BTreeHashgraph::new(),
            head: event,
            id,
        }))
    }

    #[derive(Clone)]
    struct DummyPeer {
        hashgraph: BTreeHashgraph,
        head: EventHash,
        id: PeerId,
    }

    impl Peer<BTreeHashgraph> for DummyPeer {
        fn get_sync(
            &self,
            _pk: PeerId,
            _h: Option<&BTreeHashgraph>,
        ) -> (EventHash, BTreeHashgraph) {
            (self.head.clone(), self.hashgraph.clone())
        }
        fn id(&self) -> &PeerId {
            &self.id
        }
    }

    #[test]
    fn it_should_calculate_super_majority_correctly() {
        let node = create_node();
        let peer1 = create_useless_peer(vec![1]);
        let peer2 = create_useless_peer(vec![2]);
        let peer3 = create_useless_peer(vec![3]);
        let peer4 = create_useless_peer(vec![4]);
        assert_eq!(node.get_super_majority().unwrap(), 0);
        node.add_node(peer1).unwrap();
        assert_eq!(node.get_super_majority().unwrap(), 0);
        node.add_node(peer2).unwrap();
        assert_eq!(node.get_super_majority().unwrap(), 1);
        node.add_node(peer3).unwrap();
        assert_eq!(node.get_super_majority().unwrap(), 2);
        node.add_node(peer4).unwrap();
        assert_eq!(node.get_super_majority().unwrap(), 2);
    }

    #[test]
    fn it_should_add_event_correctly() {
        let event = Event::new(vec![], None, vec![2]);
        let hash = event.hash().unwrap();
        let node = create_node();
        let head = node.head.lock().unwrap().clone().unwrap().clone();
        node.add_event(event.clone()).unwrap();
        let state = node.state.lock().unwrap();
        assert_eq!(
            state.pending_events,
            HashSet::from_iter(vec![head, hash.clone()].into_iter())
        );
        let hashgraph = node.hashgraph.lock().unwrap();
        assert!(hashgraph.contains_key(&hash));
        assert_eq!(hashgraph.get(&hash).unwrap(), &event);
    }

    #[test]
    fn it_should_create_a_new_head() {
        let node = create_node();
        let prev_head = node.head.lock().unwrap().clone().unwrap().clone();
        node.create_new_head(
            Some(ParentsPair(prev_head.clone(), prev_head.clone())),
            None,
        )
        .unwrap();
        let head = node.head.lock().unwrap().clone().unwrap().clone();
        assert_ne!(head, prev_head);
        let hashgraph = node.hashgraph.lock().unwrap();
        let head_event = hashgraph.get(&head).unwrap();
        assert!(head_event.is_valid(&head).unwrap());
        assert_eq!(
            head_event.parents(),
            &Some(ParentsPair(prev_head.clone(), prev_head.clone()))
        );
    }

    #[test]
    fn root_event_should_be_valid_in_node() {
        let node = create_node();
        let head = node.head.lock().unwrap().clone().unwrap().clone();
        let event = {
            let hashgraph = node.hashgraph.lock().unwrap();
            hashgraph.get(&head).unwrap().clone()
        };
        assert!(node.is_valid_event(&head, &event).unwrap());
    }

    #[test]
    fn invalid_event_should_be_invalid_in_node() {
        let node = create_node();
        let head = node.head.lock().unwrap().clone().unwrap().clone();
        let event = {
            let hashgraph = node.hashgraph.lock().unwrap();
            hashgraph.get(&head).unwrap().clone()
        };
        use ring::digest::{digest, SHA256};
        let real_hash = EventHash(digest(&SHA256, &vec![1]).as_ref().to_vec());
        assert!(!node.is_valid_event(&real_hash, &event).unwrap());
    }

    #[test]
    fn event_with_invalid_history_should_be_invalid_in_node() {
        let node = create_node();
        let head = node.head.lock().unwrap().clone().unwrap().clone();
        let mut event = Event::new(
            vec![],
            Some(ParentsPair(head.clone(), head.clone())),
            node.pk.public_key_bytes().to_vec(),
        );
        let hash = event.hash().unwrap();
        let signature = node.pk.sign(hash.as_ref()).as_ref().to_vec();
        event.sign(EventSignature(signature));
        node.add_event(event.clone()).unwrap();
        assert!(!node.is_valid_event(&hash, &event).unwrap());
    }

    #[test]
    fn it_should_create_a_head_with_head_and_remote_head_parents() {
        let node = create_node();
        let remote_node = create_node();
        let head = node.head.lock().unwrap().clone().unwrap().clone();
        let remote_head = remote_node.head.lock().unwrap().clone().unwrap().clone();
        let remote_hashgraph = {
            let mutex_guard = remote_node.hashgraph.lock().unwrap();
            (*mutex_guard).clone()
        };
        node.maybe_change_head(remote_head.clone(), remote_hashgraph)
            .unwrap();
        let new_head = node.head.lock().unwrap().clone().unwrap().clone();
        let hashgraph = node.hashgraph.lock().unwrap();
        let head_event = hashgraph.get(&new_head).unwrap();
        assert_eq!(
            head_event.parents(),
            &Some(ParentsPair(head.clone(), remote_head.clone()))
        );
    }

    #[test]
    #[should_panic(expected = "EventNotFound")]
    fn it_shouldnt_create_a_head() {
        let node = create_node();
        let remote_node = create_node();
        let remote_hashgraph = {
            let mutex_guard = remote_node.hashgraph.lock().unwrap();
            (*mutex_guard).clone()
        };
        use ring::digest::{digest, SHA256};
        let real_hash = EventHash(digest(&SHA256, &vec![1]).as_ref().to_vec());
        node.maybe_change_head(real_hash.clone(), remote_hashgraph)
            .unwrap();
    }

    #[test]
    fn it_should_merge_the_hashgraph() {
        let node = create_node();
        let remote_node = create_node();
        let head = node.head.lock().unwrap().clone().unwrap().clone();
        let remote_head = remote_node.head.lock().unwrap().clone().unwrap().clone();
        let remote_hashgraph = {
            let mutex_guard = remote_node.hashgraph.lock().unwrap();
            (*mutex_guard).clone()
        };
        node.merge_hashgraph(remote_hashgraph).unwrap();
        let hashgraph = node.hashgraph.lock().unwrap();
        assert!(hashgraph.contains_key(&head));
        assert!(hashgraph.contains_key(&remote_head));
    }
}
