use crate::event::event_hash::EventHash;
use failure::Error;
use rand::Rng;

pub trait Node {
    type D;

    fn run<R: Rng>(&self, rng: &mut R) -> Result<(), Error>;

    fn respond_message(&self, known: Option<Self::D>) -> Result<(EventHash, Self::D), Error>;
}
