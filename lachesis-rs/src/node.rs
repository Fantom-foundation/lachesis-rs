use errors::*;
use event::{Event, EventHash, EventSignature, Parents};
use failure::Error;
use hashgraph::Hashgraph;
use peer::{Peer, PeerId};
use rand::Rng;
use rand::prelude::IteratorRandom;
use ring::signature;
use round::Round;
use std::cell::RefCell;
use std::collections::{BTreeSet, HashMap, HashSet};
use std::iter::FromIterator;
use std::rc::Rc;
use std::sync::Mutex;
use std::sync::mpsc::Receiver;
use std::time::{SystemTime, UNIX_EPOCH};

macro_rules! get_from_mutex {
    ($resource: expr, $error: ident) => {
        $resource.lock().map_err(|e| $error::from(e))
    }
}

const C: usize = 6;

pub enum PeerMessage {
    Sync(PeerId),
}

#[inline]
fn get_current_timestamp() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).expect("Time went back").as_secs()
}

#[inline]
fn assign_root_round(event: &mut Event) -> Result<usize, Error> {
    event.set_round(0);
    Ok(0)
}

#[inline]
fn get_round_pairs(r: &Round) -> Vec<(usize, EventHash)> {
    r.witnesses().iter().map(|w| (r.id, w.clone())).collect()
}

struct NodeInternalState {
    consensus: BTreeSet<usize>,
    pending_events: HashSet<EventHash>,
    rounds: Vec<Round>,
    super_majority: usize,
    votes: HashMap<(EventHash, EventHash), bool>,
}

pub struct Node<P: Peer> {
    hashgraph: Mutex<Rc<RefCell<Hashgraph>>>,
    head: Mutex<Option<EventHash>>,
    network: Mutex<HashMap<PeerId, Rc<Box<P>>>>,
    // TODO: Plain keys in memory? Not great. See https://stackoverflow.com/a/1263421 for possible
    // alternatives
    pk: signature::Ed25519KeyPair,
    sync_channel: Receiver<PeerMessage>,
    state: Mutex<NodeInternalState>,
}

impl<P: Peer> Node<P> {
    pub fn new(
        pk: signature::Ed25519KeyPair,
        sync_channel: Receiver<PeerMessage>,
        hashgraph: Rc<RefCell<Hashgraph>>
    ) -> Result<Self, Error> {
        let state = Mutex::new(NodeInternalState {
            consensus: BTreeSet::new(),
            pending_events: HashSet::new(),
            rounds: Vec::new(),
            super_majority: 0,
            votes: HashMap::new(),
        });
        let node = Node {
            hashgraph: Mutex::new(hashgraph),
            head: Mutex::new(None),
            network: Mutex::new(HashMap::new()),
            pk,
            state,
            sync_channel,
        };
        node.create_new_head(None)?;
        Ok(node)
    }

    #[inline]
    pub fn add_node(&self, peer: Rc<Box<P>>) -> Result<(), Error> {
        let super_majority = {
            let mut network = get_from_mutex!(self.network, ResourceNetworkPoisonError)?;
            network.insert(peer.id().clone(), peer);
            network.len() * 2 /3
        };
        self.set_super_majority(super_majority)?;
        Ok(())
    }

    pub fn sync(&self, remote_head: EventHash, remote_hg: Rc<RefCell<Hashgraph>>)
        -> Result<Vec<EventHash>, Error> {
        let res = self.merge_hashgraph(remote_hg.clone())?;

        self.maybe_change_head(remote_head, remote_hg.clone())?;
        Ok(res)
    }

    pub fn divide_rounds(&self, events: Vec<EventHash>) -> Result<(), Error> {
        for eh in events.into_iter() {
            let round = self.assign_round(&eh)?;

            self.maybe_add_new_round(round)?;

            self.set_event_can_see_self(&eh)?;

            self.maybe_add_witness_to_round(round, &eh)?;
        }
        Ok(())
    }

    pub fn decide_fame(&self) -> Result<BTreeSet<usize>, Error> {
        let mut famous_events = HashMap::new();
        let mut rounds_done = BTreeSet::new();
        let mutex_guard = get_from_mutex!(self.hashgraph, ResourceHashgraphPoisonError)?;
        let super_majority = self.get_super_majority()?;
        for (round, veh) in self.get_voters()?.into_iter() {
            let witnesses = self.get_round_witnesses(round, &veh)?;
            for (ur, eh) in self.get_undetermined_events(round)? {
                if round - ur == 1 {
                    self.vote(veh.clone(), eh.clone(), witnesses.contains(&eh))?;
                } else  {
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
                            let new_vote =
                                    mutex_guard.borrow().get(&veh)?.signature()?.as_ref()[0] != 0;
                            self.vote(veh.clone(), eh, new_vote)?;
                        }
                    }
                }
            }
        }

        for (e, vote) in famous_events.into_iter() {
            let mut hashgraph = mutex_guard.borrow_mut();
            let ev = hashgraph.get_mut(&e)?;
            ev.famous(vote);
        }

        let new_consensus: BTreeSet<usize> = BTreeSet::from_iter(
            rounds_done.into_iter().filter(|r| self.are_all_witnesses_famous(*r).unwrap())
        );

        let mut state = get_from_mutex!(self.state, ResourceNodeInternalStatePoisonError)?;
        state.consensus = BTreeSet::from_iter(state.consensus.union(&new_consensus).map(|r| r.clone()));

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

    pub fn run<R: Rng>(&self, rng: &mut R) -> Result<(), Error> {
        let (head, hg) = {
            let peer = self.select_peer(rng)?;
            peer.get_sync(self.pk.public_key_bytes().to_vec())
        };
        let new_events = self.sync(head, hg)?;
        self.divide_rounds(new_events)?;
        let new_consensus = self.decide_fame()?;
        self.find_order(new_consensus)?;
        Ok(())
    }

    pub fn start_responding_messages(&self) -> Result<(), Error> {
        match self.sync_channel.recv()? {
            PeerMessage::Sync(peer_id) => {
                let network = get_from_mutex!(self.network, ResourceNetworkPoisonError)?;
                let head = self.get_head()?;
                let hashgraph = self.get_hashgraph()?;
                let wire = hashgraph.borrow().wire();
                let peer = network.get(&peer_id).ok_or(Error::from(NodeError::PeerNotFound))?;
                peer.send_sync((head, wire));
            },
        };
        Ok(())
    }

    pub fn get_hashgraph(&self) -> Result<Rc<RefCell<Hashgraph>>, Error> {
        let hashgraph = get_from_mutex!(self.hashgraph, ResourceHashgraphPoisonError)?;
        Ok(hashgraph.clone())
    }

    pub fn get_head(&self) -> Result<EventHash, Error> {
        get_from_mutex!(self.head, ResourceHeadPoisonError)?
            .clone()
            .map(|v| v.clone())
            .ok_or(Error::from(NodeError::NoHead))
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
    fn maybe_add_witness_to_round(&self, round: usize, eh: &EventHash) -> Result<(), Error>{
        let mutex_guard = get_from_mutex!(self.hashgraph, ResourceHashgraphPoisonError)?;
        let hashgraph = mutex_guard.borrow();
        let event = hashgraph.get(&eh)?;
        if round == 0 || round > hashgraph.get(&event.self_parent()?)?.round()? {
            let creator = event.creator().clone();
            self.add_witness_to_round(round, creator, eh)?;
        }
        Ok(())
    }

    #[inline]
    fn add_witness_to_round(&self, round: usize, creator: PeerId, eh: &EventHash) -> Result<(), Error> {
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
    fn is_round_received(&self, unique_famous_witnesses: &HashSet<EventHash>, eh: &EventHash)
        -> Result<bool, Error> {
        let mutex_guard = get_from_mutex!(self.hashgraph, ResourceHashgraphPoisonError)?;
        Ok(unique_famous_witnesses.iter()
            .all(|ufwh| mutex_guard.borrow().ancestors(ufwh).contains(&eh)))
    }

    #[inline]
    fn set_received_information(
        &self,
        hash: &EventHash,
        round: usize,
        unique_famous_witnesses: &HashSet<EventHash>
    ) -> Result<(), Error> {
        let timestamp_deciders = self.get_timestamp_deciders(hash, unique_famous_witnesses)?;
        let mutex_guard = get_from_mutex!(self.hashgraph, ResourceHashgraphPoisonError)?;
        let times = timestamp_deciders.into_iter()
            .map(|eh| mutex_guard.borrow().get(&eh).unwrap().timestamp().unwrap())
            .collect::<Vec<u64>>();
        let times_sum: u64 = times.iter().sum();
        let new_time = times_sum / times.len() as u64;
        let mut hashgraph = mutex_guard.borrow_mut();
        let event = hashgraph.get_mut(hash)?;
        event.set_timestamp(new_time);
        event.set_round_received(round);
        Ok(())
    }

    #[inline]
    fn get_timestamp_deciders(
        &self,
        hash: &EventHash,
        unique_famous_witnesses: &HashSet<EventHash>
    ) -> Result<HashSet<EventHash>, Error> {
        let mut result = HashSet::new();
        let mutex_guard = get_from_mutex!(self.hashgraph, ResourceHashgraphPoisonError)?;
        let hashgraph = mutex_guard.borrow();
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
        let mutex_guard = get_from_mutex!(self.hashgraph, ResourceHashgraphPoisonError)?;
        let hashgraph = mutex_guard.borrow();
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
        let mutex_guard = get_from_mutex!(self.hashgraph, ResourceHashgraphPoisonError)?;
        let hashgraph = mutex_guard.borrow();
        let state = get_from_mutex!(self.state, ResourceNodeInternalStatePoisonError)?;
        Ok(HashSet::from_iter(
            state.rounds[round].witnesses().into_iter()
                .filter(|eh| hashgraph.get(eh).unwrap().is_famous())
        ))
    }

    #[inline]
    fn are_all_witnesses_famous(&self, round: usize) -> Result<bool, Error> {
        let mutex_guard = get_from_mutex!(self.hashgraph, ResourceHashgraphPoisonError)?;
        let hashgraph = mutex_guard.borrow();
        let state = get_from_mutex!(self.state, ResourceNodeInternalStatePoisonError)?;
        Ok(state.rounds[round].witnesses().iter()
            .map(|eh| hashgraph.get(eh).unwrap())
            .all(|e| e.is_famous()))
    }

    #[inline]
    fn get_vote(&self, witnesses: &HashSet<EventHash>, eh: &EventHash) -> Result<(bool, usize), Error> {
        let total = self.get_votes_for_event(witnesses, eh)?;
        if total > witnesses.len()/2 {
            Ok((true, total))
        } else {
            Ok((false, witnesses.len()-total))
        }
    }

    #[inline]
    fn get_votes_for_event(
        &self,
        witnesses: &HashSet<EventHash>,
        eh: &EventHash
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
        let mutex_guard = get_from_mutex!(self.hashgraph, ResourceHashgraphPoisonError)?;
        let hashgraph = mutex_guard.borrow();
        let state = get_from_mutex!(self.state, ResourceNodeInternalStatePoisonError)?;
        Ok((next_consensus..round)
            .filter(|r| !state.consensus.contains(r))
            .map(|r| get_round_pairs(&state.rounds[r]).into_iter())
            .flatten()
            .filter(|(_,h)| hashgraph.get(&h).unwrap().is_undefined())
            .collect::<Vec<(usize, EventHash)>>())
    }

    #[inline]
    fn get_round_witnesses(
        &self,
        round: usize,
        hash: &EventHash
    ) -> Result<HashSet<EventHash>, Error> {
        let mut hits: HashMap<PeerId, usize> = HashMap::new();
        let mutex_guard = get_from_mutex!(self.hashgraph, ResourceHashgraphPoisonError)?;
        let hashgraph = mutex_guard.borrow();
        let event = hashgraph.get(hash)?;
        let prev_round = round - 1;
        for (creator, event_hash) in event.can_see().iter() {
            let possible_witness = hashgraph.get(event_hash)?;
            if possible_witness.round()? == prev_round {
                for (_creator, _event_hash) in possible_witness.can_see().iter() {
                    let r = hashgraph.get(_event_hash)?.round()?;
                    if r == prev_round {
                        let new_val = hits.get(creator).map(|v| *v+1).unwrap_or(1);
                        hits.insert(creator.clone(), new_val);
                    }
                }
            }
        }
        let super_majority = self.get_super_majority()?;
        let state = get_from_mutex!(self.state, ResourceNodeInternalStatePoisonError)?;
        let r = &state.rounds[prev_round];
        let map_iter = hits.into_iter()
            .filter(|(_,v)| *v > super_majority)
            .map(|(c, _)| r.witnesses_map()[&c].clone());
        Ok(HashSet::from_iter(map_iter))
    }

    #[inline]
    fn get_voters(&self) -> Result<Vec<(usize, EventHash)>, Error> {
        let next_consensus = self.get_next_consensus()?;
        let state = get_from_mutex!(self.state, ResourceNodeInternalStatePoisonError)?;
        Ok(state.rounds[next_consensus..state.rounds.len()].iter()
            .flat_map(|r| get_round_pairs(r))
            .collect())
    }

    #[inline]
    fn get_next_consensus(&self) -> Result<usize, Error> {
        let state = get_from_mutex!(self.state, ResourceNodeInternalStatePoisonError)?;
        Ok(state.consensus.iter()
            .last()
            .map(|v| *v + 1)
            .unwrap_or(0))
    }

    #[inline]
    fn set_event_can_see_self(&self, hash: &EventHash) -> Result<(), Error> {
        let mutex_guard = get_from_mutex!(self.hashgraph, ResourceHashgraphPoisonError)?;
        let mut hashgraph = mutex_guard.borrow_mut();
        let event = hashgraph.get_mut(&hash)?;
        let creator = event.creator().clone();
        event.add_can_see(creator, hash.clone());
        Ok(())
    }

    #[inline]
    fn assign_round(&self, hash: &EventHash) -> Result<usize, Error> {
        let is_root = {
            let mutex_guard = get_from_mutex!(self.hashgraph, ResourceHashgraphPoisonError)?;
            let hashgraph = mutex_guard.borrow();
            hashgraph.get(hash)?.is_root()
        };
        if is_root {
            let mutex_guard = get_from_mutex!(self.hashgraph, ResourceHashgraphPoisonError)?;
            let mut hashgraph = mutex_guard.borrow_mut();
            assign_root_round(hashgraph.get_mut(&hash)?)
        } else {
            self.assign_non_root_round(hash)
        }
    }

    #[inline]
    fn assign_non_root_round(&self, hash: &EventHash) -> Result<usize, Error> {
        let events_parents_can_see =  {
            let mutex_guard = get_from_mutex!(self.hashgraph, ResourceHashgraphPoisonError)?;
            let hashgraph = mutex_guard.borrow();
            hashgraph.events_parents_can_see(hash)?
        };
        let mut r = self.get_parents_round(hash)?;
        let hits = self.get_hits_per_events(r, &events_parents_can_see)?;
        let sm = self.get_super_majority()?;
        let votes = hits
            .values()
            .map(|v| v.clone())
            .filter(|v| *v > sm);
        if votes.sum::<usize>() > sm {
            r += 1;
        }
        self.set_events_parents_can_see(hash, events_parents_can_see)?;
        Ok(r)
    }

    #[inline]
    fn get_hits_per_events(
        &self, r: usize, events_parents_can_see: &HashMap<PeerId, EventHash>
    ) -> Result<HashMap<PeerId, usize>, Error> {
        let mutex_guard = get_from_mutex!(self.hashgraph, ResourceHashgraphPoisonError)?;
        let hashgraph = mutex_guard.borrow();
        let mut hits: HashMap<PeerId, usize> = HashMap::new();
        for (_, h) in events_parents_can_see.iter() {
            let event = hashgraph.get(h)?;
            if event.round()? == r {
                for (_c, _h) in event.can_see().iter() {
                    let seen_event = hashgraph.get(_h)?;
                    if seen_event.round()? == r {
                        let prev = hits.get(_c).map(|v| v.clone()).unwrap_or(0);
                        hits.insert(_c.clone(), prev+1);
                    }
                }
            }
        }
        Ok(hits)
    }

    #[inline]
    fn get_parents_round(&self, hash: &EventHash) -> Result<usize, Error> {
        let mutex_guard = get_from_mutex!(self.hashgraph, ResourceHashgraphPoisonError)?;
        let hashgraph = mutex_guard.borrow();
        let event = hashgraph.get(hash)?;
        let parents = event.parents().clone().ok_or(Error::from(EventError::NoParents))?;
        parents.max_round((*mutex_guard).clone())
    }

    #[inline]
    fn set_events_parents_can_see(
        &self,
        hash: &EventHash,
        events_parents_can_see: HashMap<Vec<u8>, EventHash>
    ) -> Result<(), Error> {
        let mutex_guard = get_from_mutex!(self.hashgraph, ResourceHashgraphPoisonError)?;
        let mut hashgraph = mutex_guard.borrow_mut();
        let event = hashgraph.get_mut(hash)?;
        event.set_can_see(events_parents_can_see);
        Ok(())
    }

    #[inline]
    fn merge_hashgraph(&self, remote_hg: Rc<RefCell<Hashgraph>>) -> Result<Vec<EventHash>, Error> {
        let diff = {
            let hashgraph = get_from_mutex!(self.hashgraph, ResourceHashgraphPoisonError)?;
            remote_hg.borrow().difference(hashgraph.clone())
        };
        for eh in diff.clone().into_iter() {
            let is_valid_event = {
                let rhg = remote_hg.borrow();
                let event = rhg.get(&eh)?;
                self.is_valid_event(&eh, event)
            }?;
            if is_valid_event {
                let mut rhg = remote_hg.borrow_mut();
                self.add_event(rhg.get(&eh)?.clone())?;
            }
        }
        Ok(diff)
    }

    #[inline]
    fn maybe_change_head(&self, remote_head: EventHash, remote_hg: Rc<RefCell<Hashgraph>>) -> Result<(), Error> {
        let remote_head_event = remote_hg.borrow().get(&remote_head).unwrap().clone();

        if self.is_valid_event(&remote_head, &remote_head_event)? {
            let current_head = self.get_head()?;
            let parents = Parents(current_head, remote_head);
            self.create_new_head(Some(parents))?;
        }
        Ok(())
    }

    #[inline]
    fn is_valid_event(&self, event_hash: &EventHash, event: &Event) -> Result<bool, Error> {
        event
            .is_valid(event_hash)
            .and_then(|b| {
                if !b {
                    Ok(false)
                } else {
                    let mutex_guard = get_from_mutex!(self.hashgraph, ResourceHashgraphPoisonError)?;
                    let hashgraph = mutex_guard.borrow();
                    hashgraph.is_valid_event(event)
                }
            })
    }

    #[inline]
    fn select_peer<R: Rng>(&self, rng: &mut R) -> Result<Rc<Box<P>>, Error> {
        self.network.lock().unwrap()
            .values()
            .choose(rng)
            .ok_or(Error::from(NodeError::EmptyNetwork))
            .map(|p| p.clone())
    }

    fn create_new_head(&self, parents: Option<Parents>) -> Result<(), Error> {
        let mut event = Event::new(
            Vec::new(),
            parents,
            self.pk.public_key_bytes().to_vec()
        );
        if event.is_root() {
            event.set_timestamp(get_current_timestamp())
        }
        let hash = event.hash()?;
        let signature = self.pk.sign(hash.as_ref());
        event.sign(EventSignature(signature.as_ref().to_vec()));
        self.add_event(event)?;
        let mut current_head = get_from_mutex!(self.head, ResourceHeadPoisonError)?;
        *current_head = Some(hash);
        Ok(())
    }

    #[inline]
    fn add_event(&self, e: Event) -> Result<(), Error> {
        let hash = e.hash()?;
        self.add_pending_event(hash.clone())?;
        let mutex_guard = get_from_mutex!(self.hashgraph, ResourceHashgraphPoisonError)?;
        let mut hashgraph = mutex_guard.borrow_mut();
        Ok(hashgraph.insert(hash, e))
    }

    #[inline]
    fn add_pending_event(&self, e: EventHash) -> Result<(), Error> {
        let mut state = get_from_mutex!(self.state, ResourceNodeInternalStatePoisonError)?;
        state.pending_events.insert(e);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use event::{Event, EventHash, EventSignature, Parents};
    use hashgraph::*;
    use peer::{Peer, PeerId};
    use ring::{rand, signature};
    use ring::digest::{digest, SHA256};
    use std::cell::RefCell;
    use std::collections::HashSet;
    use std::iter::FromIterator;
    use std::rc::Rc;
    use std::sync::mpsc::channel;
    use super::Node;

    fn create_node() -> Node<DummyPeer> {
        let rng = rand::SystemRandom::new();
        let pkcs8_bytes = signature::Ed25519KeyPair::generate_pkcs8(&rng).unwrap();
        let kp = signature::Ed25519KeyPair::from_pkcs8(untrusted::Input::from(&pkcs8_bytes)).unwrap();
        let (_, receiver) = channel();
        let hashgraph = BTreeHashgraph::new();
        Node::new(kp, receiver, Rc::new(RefCell::new(hashgraph))).unwrap()
    }

    fn create_useless_peer(id: PeerId) -> Rc<Box<DummyPeer>> {
        let digest = digest(&SHA256, b"42");
        let event = EventHash(digest.as_ref().to_vec());
        Rc::new(Box::new(DummyPeer {
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

    impl Peer for DummyPeer {
        fn get_sync(&self, _pk: PeerId) -> (EventHash, Rc<RefCell<Hashgraph>>) {
            (self.head.clone(), Rc::new(RefCell::new(self.hashgraph.clone())))
        }
        fn send_sync(&self, _msg: (EventHash, HashgraphWire)) {}
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
        assert_eq!(state.pending_events, HashSet::from_iter(vec![head, hash.clone()].into_iter()));
        let mutex_guard = node.hashgraph.lock().unwrap();
        let hashgraph = mutex_guard.borrow();
        assert!(hashgraph.contains_key(&hash));
        assert_eq!(hashgraph.get(&hash).unwrap(), &event);
    }

    #[test]
    fn it_should_create_a_new_head() {
        let node = create_node();
        let prev_head = node.head.lock().unwrap().clone().unwrap().clone();
        node.create_new_head(Some(Parents(prev_head.clone(), prev_head.clone()))).unwrap();
        let head = node.head.lock().unwrap().clone().unwrap().clone();
        assert_ne!(head, prev_head);
        let mutex_guard = node.hashgraph.lock().unwrap();
        let hashgraph = mutex_guard.borrow();
        let head_event = hashgraph.get(&head).unwrap();
        assert!(head_event.is_valid(&head).unwrap());
        assert_eq!(head_event.parents(), &Some(Parents(prev_head.clone(), prev_head.clone())));
    }

    #[test]
    fn root_event_should_be_valid_in_node() {
        let node = create_node();
        let head = node.head.lock().unwrap().clone().unwrap().clone();
        let event = {
            let mutex_guard = node.hashgraph.lock().unwrap();
            let hashgraph = mutex_guard.borrow();
            hashgraph.get(&head).unwrap().clone()
        };
        assert!(node.is_valid_event(&head, &event).unwrap());
    }

    #[test]
    fn invalid_event_should_be_invalid_in_node() {
        let node = create_node();
        let head = node.head.lock().unwrap().clone().unwrap().clone();
        let event = {
            let mutex_guard = node.hashgraph.lock().unwrap();
            let hashgraph = mutex_guard.borrow();
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
        let mut event = Event::new(vec![], Some(Parents(head.clone(), head.clone())), node.pk.public_key_bytes().to_vec());
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
        node.maybe_change_head(remote_head.clone(), remote_hashgraph).unwrap();
        let new_head = node.head.lock().unwrap().clone().unwrap().clone();
        let mutex_guard = node.hashgraph.lock().unwrap();
        let hashgraph = mutex_guard.borrow();
        let head_event = hashgraph.get(&new_head).unwrap();
        assert_eq!(head_event.parents(), &Some(Parents(head.clone(), remote_head.clone())));
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
        node.maybe_change_head(real_hash.clone(), remote_hashgraph).unwrap();
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
        let mutex_guard = node.hashgraph.lock().unwrap();
        let hashgraph = mutex_guard.borrow();
        assert!(hashgraph.contains_key(&head));
        assert!(hashgraph.contains_key(&remote_head));
    }
}
