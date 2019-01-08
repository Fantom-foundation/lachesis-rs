use crate::event::event_hash::EventHash;
use crate::failure::Backtrace;
use crate::peer::PeerId;
use crate::printable_hash::PrintableHash;
use std::fmt;
use std::sync::PoisonError;

#[derive(Debug, Fail)]
pub enum ParentsError {
    #[fail(display = "Parents are empty")]
    EmptyParents,
}

#[derive(Debug)]
pub(crate) enum NodeErrorType {
    PeerNotFound(PeerId),
    EmptyNetwork,
    NoHead,
}

impl fmt::Display for NodeErrorType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let msg = match self {
            NodeErrorType::EmptyNetwork => String::from("The node network it's empty"),
            NodeErrorType::NoHead => String::from("The node has no head"),
            NodeErrorType::PeerNotFound(p) => format!("Peer {} not found", p.printable_hash()),
        };
        write!(f, "{}", msg)
    }
}

#[derive(Debug, Fail)]
#[fail(
    display = "Node failed with error: {}\nTraceback: {}",
    error_type, backtrace
)]
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
    UnsignedEvent { hash: EventHash },
    RoundNotSet { hash: EventHash },
    NoSelfParent { hash: EventHash },
    NoParents { hash: EventHash },
    NoSignature { hash: EventHash },
    NoTimestamp { hash: EventHash },
}

impl fmt::Display for EventErrorType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let msg = match self {
            EventErrorType::UnsignedEvent { hash } => {
                format!("The event {} it's unsigned", hash.printable_hash())
            }
            EventErrorType::RoundNotSet { hash } => {
                format!("The event {} round isn't set", hash.printable_hash())
            }
            EventErrorType::NoSelfParent { hash } => {
                format!("The event {} self parent isn't set", hash.printable_hash())
            }
            EventErrorType::NoParents { hash } => {
                format!("The event {} parents aren't set", hash.printable_hash())
            }
            EventErrorType::NoSignature { hash } => {
                format!("The event {} signature isn't set", hash.printable_hash())
            }
            EventErrorType::NoTimestamp { hash } => {
                format!("The event {} timestamp isn't set", hash.printable_hash())
            }
        };
        write!(f, "{}", msg)
    }
}

#[derive(Debug, Fail)]
#[fail(
    display = "Event failed with error: {}\nTraceback: {}",
    error_type, backtrace
)]
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
#[fail(
    display = "Hashgraph failed with error: {}\nTraceback: {}",
    error_type, backtrace
)]
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
    backtrace: Backtrace,
}

impl ResourceHashgraphPoisonError {
    pub fn new() -> ResourceHashgraphPoisonError {
        ResourceHashgraphPoisonError {
            backtrace: Backtrace::new(),
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
    backtrace: Backtrace,
}

impl ResourceHeadPoisonError {
    pub fn new() -> ResourceHeadPoisonError {
        ResourceHeadPoisonError {
            backtrace: Backtrace::new(),
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
    backtrace: Backtrace,
}

impl ResourceNetworkPoisonError {
    pub fn new() -> ResourceNetworkPoisonError {
        ResourceNetworkPoisonError {
            backtrace: Backtrace::new(),
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
    backtrace: Backtrace,
}

impl ResourceNodeInternalStatePoisonError {
    pub fn new() -> ResourceNodeInternalStatePoisonError {
        ResourceNodeInternalStatePoisonError {
            backtrace: Backtrace::new(),
        }
    }
}

//for op-?, "auto" type conversion
impl<T> From<PoisonError<T>> for ResourceNodeInternalStatePoisonError {
    fn from(_: PoisonError<T>) -> Self {
        ResourceNodeInternalStatePoisonError::new()
    }
}
