use crate::transaction::{Transaction, TransactionHash, TransactionOrdering};

pub trait Node {
    fn submit_transaction(tx: Transaction);
    fn get_transaction_by_hash(tx_hash: TransactionHash) -> (TransactionOrdering, Transaction);
    fn get_transaction_by_order(tx_order: TransactionOrdering) -> (TransactionHash, Transaction);
}

