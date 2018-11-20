use crate::transaction::{Transaction, TransactionHash, TransactionStatus};
use crate::transport::{Transport, Message, TransportError};

pub trait Client<T: Transaction, U: Transport<W, X>, W: Message, X: TransportError> {
    fn submit_transaction(tx_hash: TransactionHash, tx: T) -> TransactionStatus;
    fn check_transaction_status(tx_hash: TransactionHash) -> TransactionStatus;
}
