use failure::Backtrace;
use std::fmt;
use std::sync::PoisonError;

#[derive(Debug)]
pub(crate) enum NodeErrorType {
    EmptyNetwork,
    NoHead,
}

impl fmt::Display for NodeErrorType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let msg = match self {
            NodeErrorType::EmptyNetwork => "The node network it's empty",
            NodeErrorType::NoHead => "The node has no head",
        };
        write!(f, "{}", msg)
    }
}

#[derive(Debug, Fail)]
#[fail(display = "Node failed with error: {}\nTraceback: {}", error_type, backtrace)]
pub(crate) struct NodeError {
    backtrace: Backtrace,
    error_type: NodeErrorType,
}

impl NodeError {
    pub(crate) fn new(error_type: NodeErrorType) -> NodeError {
        NodeError {
            backtrace: Backtrace::new(),
            error_type,
        }
    }
}

#[derive(Debug)]
pub(crate) enum EventErrorType {
    UnsignedEvent,
    RoundNotSet,
    NoSelfParent,
    NoParents,
    NoSignature,
    NoTimestamp,
}

impl fmt::Display for EventErrorType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let msg = match self {
            EventErrorType::UnsignedEvent => "The event it's unsigned",
            EventErrorType::RoundNotSet => "The event round isn't set",
            EventErrorType::NoSelfParent => "The event self parent isn't set",
            EventErrorType::NoParents => "The event parents aren't set",
            EventErrorType::NoSignature => "The event signature isn't set",
            EventErrorType::NoTimestamp => "The event timestamp isn't set",
        };
        write!(f, "{}", msg)
    }
}

#[derive(Debug, Fail)]
#[fail(display = "Event failed with error: {}\nTraceback: {}", error_type, backtrace)]
pub(crate) struct EventError {
    backtrace: Backtrace,
    error_type: EventErrorType,
}

impl EventError {
    pub(crate) fn new(error_type: EventErrorType) -> EventError {
        EventError {
            backtrace: Backtrace::new(),
            error_type,
        }
    }
}

#[derive(Debug)]
pub(crate) enum HashgraphErrorType {
    EventNotFound,
}

impl fmt::Display for HashgraphErrorType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let msg = match self {
            HashgraphErrorType::EventNotFound => "Event not found in hashgraph",
        };
        write!(f, "{}", msg)
    }
}

#[derive(Debug, Fail)]
#[fail(display = "Hashgraph failed with error: {}\nTraceback: {}", error_type, backtrace)]
pub(crate) struct HashgraphError {
    backtrace: Backtrace,
    error_type: HashgraphErrorType,
}

impl HashgraphError {
    pub(crate) fn new(error_type: HashgraphErrorType) -> HashgraphError {
        HashgraphError {
            backtrace: Backtrace::new(),
            error_type,
        }
    }
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
