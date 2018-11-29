use failure::Backtrace;
use std::sync::PoisonError;

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

#[derive(Debug, Fail)]
#[fail(display = "Hashgraph Mutex was poisoned")]
pub struct ResourceHashgraphPoisonError {
    backtrace: Backtrace
}

impl ResourceHashgraphPoisonError {
    pub fn new() -> ResourceHashgraphPoisonError {
        ResourceHashgraphPoisonError {
            backtrace: Backtrace::new()
        }
    }
}

//for op-?, "auto" type conversion
impl<T> From<PoisonError<T>> for ResourceHashgraphPoisonError {
    fn from(_: PoisonError<T>) -> Self {
        ResourceHashgraphPoisonError::new()
    }
}

#[derive(Debug, Fail)]
#[fail(display = "Head Mutex was poisoned")]
pub struct ResourceHeadPoisonError {
    backtrace: Backtrace
}

impl ResourceHeadPoisonError {
    pub fn new() -> ResourceHeadPoisonError {
        ResourceHeadPoisonError {
            backtrace: Backtrace::new()
        }
    }
}

//for op-?, "auto" type conversion
impl<T> From<PoisonError<T>> for ResourceHeadPoisonError {
    fn from(_: PoisonError<T>) -> Self {
        ResourceHeadPoisonError::new()
    }
}

#[derive(Debug, Fail)]
#[fail(display = "Network Mutex was poisoned")]
pub struct ResourceNetworkPoisonError {
    backtrace: Backtrace
}

impl ResourceNetworkPoisonError {
    pub fn new() -> ResourceNetworkPoisonError {
        ResourceNetworkPoisonError {
            backtrace: Backtrace::new()
        }
    }
}

//for op-?, "auto" type conversion
impl<T> From<PoisonError<T>> for ResourceNetworkPoisonError {
    fn from(_: PoisonError<T>) -> Self {
        ResourceNetworkPoisonError::new()
    }
}

#[derive(Debug, Fail)]
#[fail(display = "Node internal state Mutex was poisoned")]
pub struct ResourceNodeInternalStatePoisonError {
    backtrace: Backtrace
}

impl ResourceNodeInternalStatePoisonError {
    pub fn new() -> ResourceNodeInternalStatePoisonError {
        ResourceNodeInternalStatePoisonError {
            backtrace: Backtrace::new()
        }
    }
}

//for op-?, "auto" type conversion
impl<T> From<PoisonError<T>> for ResourceNodeInternalStatePoisonError {
    fn from(_: PoisonError<T>) -> Self {
        ResourceNodeInternalStatePoisonError::new()
    }
}
