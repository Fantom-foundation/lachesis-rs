use crate::event::event_hash::EventHash;
use crate::event::parents::Parents;
use crate::event::Event;
use failure::Error;
use rand::Rng;
use serde::Serialize;

pub trait Node {
    type D;
    type P: Parents + Clone + Serialize;

    fn run<R: Rng>(&self, rng: &mut R) -> Result<(), Error>;

    fn respond_message(&self, known: Option<Self::D>) -> Result<(EventHash, Self::D), Error>;

    fn add_transaction(&self, msg: Vec<u8>) -> Result<(), Error>;

    fn get_ordered_events(&self) -> Result<Vec<Event<Self::P>>, Error>;
}
