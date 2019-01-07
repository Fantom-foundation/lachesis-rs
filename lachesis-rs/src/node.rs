use event::EventHash;
use failure::Error;
use rand::Rng;

pub trait Node {
    type D;

    fn run<R: Rng>(&self, rng: &mut R) -> Result<(), Error>;

    fn respond_message(&self) -> Result<(EventHash, Self::D), Error>;
}
