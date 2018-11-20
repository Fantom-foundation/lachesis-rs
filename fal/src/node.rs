use crate::transaction::{Transaction, TransactionHash, AbsoluteOrdering};

pub trait Node<T: Transaction>{
    fn get_transaction_by_hash(tx_hash: TransactionHash) -> T;
    fn get_transaction_by_order(tx_order: AbsoluteOrdering) -> T;
}

