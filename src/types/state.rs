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

pub fn create_seed_array(seed: u16) -> [u8; 32] {
    let mut arr = [0u8; 32];
    // Split the seed into two bytes and place them at the beginning of the array
    arr[0] = seed as u8; // Lower 8 bits
    arr[1] = (seed >> 8) as u8; // Upper 8 bits
    arr
}

impl State {
    pub fn new() -> Self {
        let mut map : HashMap<Address, (u32, i32)> = HashMap::new();
        
        let seed1 = create_seed_array(7000);
        let seed2 = create_seed_array(7001);
        let seed3 = create_seed_array(7002);

        // Create key pairs
        let key_pair1 = Ed25519KeyPair::from_seed_unchecked(&seed1).unwrap();
        let key_pair2 = Ed25519KeyPair::from_seed_unchecked(&seed2).unwrap();
        let key_pair3 = Ed25519KeyPair::from_seed_unchecked(&seed3).unwrap();
        
        // Get public key bytes
        let public_key_bytes1 = key_pair1.public_key().as_ref();
        let public_key_bytes2 = key_pair2.public_key().as_ref();
        let public_key_bytes3 = key_pair3.public_key().as_ref();

        // create addresses
        let address1 = Address::from_public_key_bytes(public_key_bytes1);
        let address2 = Address::from_public_key_bytes(public_key_bytes2);
        let address3 = Address::from_public_key_bytes(public_key_bytes3);

        map.insert(address1, (0, i32::MAX / 2));
        map.insert(address2, (0, i32::MAX / 2));
        map.insert(address3, (0, i32::MAX / 2));
        
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

