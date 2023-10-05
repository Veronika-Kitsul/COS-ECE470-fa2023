use serde::{Serialize, Deserialize};
use crate::types::hash::{H256, Hashable};
use std::time::{SystemTime, UNIX_EPOCH};
use crate::types::transaction::SignedTransaction;
use super::merkle:: MerkleTree;
use ring::digest;
use rand::Rng;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Block {
    header : Header,
    content : Content,
    to_genesis : u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Header {
    parent : H256,
    nonce : u32,
    difficulty: H256,
    timestamp: u128,
    merkle_root : H256,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Content {
    transactions : Vec<SignedTransaction>,
}

impl Hashable for Header {
    fn hash(&self) -> H256 {
        let serialized_h_str = serde_json::to_string(&self).unwrap();
        let serialized_h = serialized_h_str.as_bytes();
        let digest_hash = digest::digest(&digest::SHA256, &serialized_h);
        let hash_bytes = digest_hash.as_ref();
        let mut hash_array = [0u8; 32];
        hash_array.copy_from_slice(&hash_bytes[0..32]);
        return H256::from(hash_array);
    }
}

impl Hashable for Block {
    fn hash(&self) -> H256 {
        return self.header.hash();
    }
}

impl Hashable for SignedTransaction {
    fn hash(&self) -> H256 {
        let serialized_h_str = serde_json::to_string(&self).unwrap();
        let serialized_h = serialized_h_str.as_bytes();
        let digest_hash = digest::digest(&digest::SHA256, &serialized_h);
        let hash_bytes = digest_hash.as_ref();
        let mut hash_array = [0u8; 32];
        hash_array.copy_from_slice(&hash_bytes[0..32]);
        return H256::from(hash_array);
    }
}

impl Header {
    pub fn new(p: H256, n: u32, d: H256, t: u128, r: H256)-> Header {
        Header{
            parent : p,
            nonce : n,
            difficulty : d,
            timestamp : t,
            merkle_root : r,
        }
    }
    pub fn parent(&self) -> H256 {
        return self.parent;
    }
    pub fn nonce(&self) -> u32{
        return self.nonce;
    }
    pub fn difficulty(&self) -> H256 {
        return self.difficulty;
    }
    pub fn timestamp(&self) -> u128 {
        return self.timestamp;
    }
}

impl Content {
    pub fn new(t: Vec<SignedTransaction>) -> Content {
        Content {
            transactions : t.clone(),
        }
    }


    pub fn root(&self) -> H256 {
        let mut hash_transactions : Vec<H256> =   Vec:: new();
        for t in &self.transactions {
            hash_transactions.push(t.hash());
        }
        let mut tree = MerkleTree :: new(&hash_transactions);
        return tree.root();
    }

}

impl Block {

    pub fn new(p: H256, n: u32, d: H256, t: u128, gen : u32, transactions: Vec<SignedTransaction>) -> Block {
        let c = Content :: new(transactions);

        Block{
            header :  Header :: new(p, n, d, t, c.root()),
            content : c,
            to_genesis : gen,
        }
    }

    pub fn get_parent(&self) -> H256 {
        return self.header.parent();
    }

    pub fn get_difficulty(&self) -> H256 {
        return self.header.difficulty();
    }

    pub fn get_to_genesis(&self) -> u32 {
        return self.to_genesis;
    }
    pub fn set_to_genesis(&mut self, gen : u32) {
        self.to_genesis = gen;
    }

    pub fn get_root(&self) -> H256 {
        return self.content.root();
    }
}

#[cfg(any(test, test_utilities))]
pub fn generate_random_block(parent: &H256) -> Block {
    let mut rng = rand::thread_rng();
    let n = rng.gen_range(0..u32::MAX);

    let bytes: [u8; 32] = [0xFF; 32];
    let d = H256::from(bytes);
    
    let t = 0;
    let zeros : [u8; 32] = [0; 32];
    let r = H256::from(zeros);
    let transactions : Vec<SignedTransaction> =   Vec:: new();
    let gen = 0;

    Block {
        header : Header :: new(*parent, n, d, t, r),
        content : Content :: new(transactions),
        to_genesis : gen,
    }
}