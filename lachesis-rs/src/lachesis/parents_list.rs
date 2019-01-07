use errors::ParentsError;
use event::{EventHash, Parents};
use failure::Error;

#[derive(Clone, Serialize)]
pub struct ParentsList(pub Vec<EventHash>);

impl Parents for ParentsList {
    fn self_parent(&self) -> Result<EventHash, Error> {
        Ok(self
            .0
            .first()
            .ok_or(Error::from(ParentsError::EmptyParents))?
            .clone())
    }
}
