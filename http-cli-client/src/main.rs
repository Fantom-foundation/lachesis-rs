extern crate fal;

use self::fal::client::Client;
use self::fal::transaction::{Transaction, TransactionStatus, TransactionHash, TransactionOrdering};

struct HttpCliClient {

}

impl Client for HttpCliClient {

    fn submit_transaction(tx_hash: TransactionHash, tx: Transaction) -> TransactionStatus {
        return TransactionStatus::Failed;
    }

    fn check_transaction_status(tx_hash: TransactionHash) -> TransactionStatus {
        return TransactionStatus::Failed;
    }
}

fn main() {
    println!("Hello, world!");
}
