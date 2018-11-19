pub enum TransactionStatus {
    Pending,
    Failed,
    Complete,
}

pub struct Transaction {

}

pub type TransactionOrdering = u64;

pub type TransactionHash = [u8; 32];