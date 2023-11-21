use std::collections::HashMap;
use rand::Rng;
use ring::signature::{Ed25519KeyPair, KeyPair};
use super::{
    address::Address,
};

#[derive(Debug, Clone)]
pub struct State {
    pub accounts : HashMap<Address, (u32, i32)>,
}

impl State {
    pub fn new() -> Self {
        let mut map : HashMap<Address, (u32, i32)> = HashMap::new();

        // Generate a random seed
        let mut rng = rand::thread_rng();
        let random_seed: [u8; 32] = rng.gen();

        // Create a key pair
        let key_pair = Ed25519KeyPair::from_seed_unchecked(&random_seed).unwrap();

        // Get public key bytes and create an address
        let public_key_bytes = key_pair.public_key().as_ref();
        let address = Address::from_public_key_bytes(public_key_bytes);

        map.insert(address, (0, i32::MAX));
        
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

    pub fn get_value(&self, a: Address)->i32{
        self.accounts.get(&a).unwrap().1
    }

    pub fn add_account(&mut self, a:Address, n:u32, v:i32) {
        self.accounts.insert(a, (n, v));
    }
}

