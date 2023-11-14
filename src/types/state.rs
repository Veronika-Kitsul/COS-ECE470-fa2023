use std::collections::HashMap;
use super::{
    hash::{Hashable, H256},
    address::Address,
    transaction::SignedTransaction,
};

#[derive(Debug, Clone)]
pub struct State {
    accounts : HashMap<Address, (u32, u32)>,
}

impl State {
    pub fn new() -> Self {
        let mut map : HashMap<Address, (u32, u32)> = HashMap::new();
        Self {
            accounts : map,
        }
    }

    pub fn contains_account(&self, a: Address)-> bool{
        self.accounts.contains_key(&a)
    }

    pub fn get_nonce(&self, a: Address)->u32{
        self.accounts.get(&a).unwrap().0
    }

    pub fn get_value(&self, a: Address)->u32{
        self.accounts.get(&a).unwrap().1
    }

    pub fn add_account(&mut self, a:Address, n:u32, v:u32) {
        self.accounts.insert(a, (n, v));
    }

}

