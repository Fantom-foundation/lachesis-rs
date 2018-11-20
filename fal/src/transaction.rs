use std::cmp::Ordering;

pub enum TransactionStatus {
    Pending,
    Failed,
    Complete,
}

pub trait Transaction {
    fn get_absolute_ordering() -> AbsoluteOrdering;
}

pub type AbsoluteOrdering = u64;

pub type TransactionHash = [u8; 32];
