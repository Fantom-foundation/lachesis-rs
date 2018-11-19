use crate::transaction::{Transaction, TransactionHash, TransactionStatus};

pub trait Client {
//    fn connect_to_node(addr: NodeAddr) -> NodeConn;
    fn submit_transaction(tx_hash: TransactionHash, tx: Transaction) -> TransactionStatus;
    fn check_transaction_status(tx_hash: TransactionHash) -> TransactionStatus;

}
