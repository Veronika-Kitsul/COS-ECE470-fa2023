use super::{
    hash::{Hashable, H256},
    transaction::SignedTransaction,
};
use std::collections::HashMap;

#[derive(Debug, Default, Clone)]
pub struct Mempool {
    pub transactions : HashMap<H256, SignedTransaction>,
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
        self.transactions.contains_key(&hash)
    }

    // remove transaction from the mempool and return it
    pub fn rm_transaction(&mut self, hash: H256) {
        if self.transactions.contains_key(&hash) {
            print!("actually removing the transactions\n");
            self.transactions.remove(&hash);
        }
    }

    // get a transaction
    pub fn get_transaction(&self, hash: H256) -> SignedTransaction {
        self.transactions.get(&hash).unwrap().clone()
    }

    // add transaction to the mempool
    pub fn add_transaction(&mut self, transaction : SignedTransaction) {
        self.transactions.insert(transaction.hash(), transaction);
    }

    pub fn get_max(&self, max: usize, min: usize) -> Vec<SignedTransaction> {
        if min > self.transactions.len() {
            let empty : Vec<SignedTransaction> = Vec::new();
            return empty;
        }

        let entries = (max).min(min);
        let values: Vec<SignedTransaction> = self.transactions.iter().take(entries).map(|(_, value)| value.clone()).collect();
        values
    }    
}
