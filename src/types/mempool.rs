use super::{
    hash::{Hashable, H256},
    transaction::SignedTransaction,
};
use std::collections::HashMap;

#[derive(Debug, Default, Clone)]
pub struct Mempool {
    transactions : HashMap<H256, SignedTransaction>,
}

impl Mempool {
    pub fn new() -> Self {
        let mut m : HashMap<H256, SignedTransaction> = HashMap::new();
        Self {
            transactions : m,
        }
    }

    // check if transaction is in mempool
    pub fn contains_transaction(&self, hash: H256) -> bool{
        self.transations.contains(hash)
    }

    // remove transaction from the mempool and return it
    pub fn rm_transaction(&self, hash: H256) -> SignedTransaction{
        self.transactions.remove(&hash).unwrap()
    }

    // get a transaction
    pub fn get_transaction(&self, hash: H256) -> SignedTransaction {
        self.transactions.get(&hash).unwrap().clone()
    }

    // add transaction to the mempool
    pub fn add_transaction(&self, transaction : SignedTransaction) {
        self.transactions.insert(transaction.hash(), transaction);
    }

    pub fn get_max(&self, max : u8) -> Vec<SignedTransaction> {
        let entries = &max.min(self.transactions.len());
        let values: Vec<SignedTransaction> = self.transactions.into_iter().take(entries).map(|(_, value)| value).collect();
        return values;
    }
}
