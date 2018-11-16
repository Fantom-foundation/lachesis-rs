#[derive(Debug, Fail)]
pub(crate) enum NodeError {
    #[fail(display = "The node network it's empty")]
    EmptyNetwork,
    #[fail(display = "The node has no head")]
    NoHead,
}

#[derive(Debug, Fail)]
pub(crate) enum EventError {
    #[fail(display = "The event it's unsigned")]
    UnsignedEvent,
    #[fail(display = "The event round isn't set")]
    RoundNotSet,
    #[fail(display = "The event self parent isn't set")]
    NoSelfParent,
    #[fail(display = "The event parents aren't set")]
    NoParents,
    #[fail(display = "The event signature isn't set")]
    NoSignature,
    #[fail(display = "The event timestamp isn't set")]
    NoTimestamp,
}

#[derive(Debug, Fail)]
pub(crate) enum HashgraphError {
    #[fail(display = "Event not found in hashgraph")]
    EventNotFound,
}
