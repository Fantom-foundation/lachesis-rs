use crate::transaction::{Transaction, TransactionHash, TransactionStatus};
use crate::network::{Transport, Message};

pub trait Client<T: Transaction, U: Transport<W>, W: Message> {
    fn submit_transaction(tx_hash: TransactionHash, tx: T) -> TransactionStatus;
    fn check_transaction_status(tx_hash: TransactionHash) -> TransactionStatus;
}
