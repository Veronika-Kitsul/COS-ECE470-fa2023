use crate::types::block::{Block};
use crate::types::hash::{H256, Hashable};
use std::collections::HashMap;
use hex_literal::hex;
use crate::types::transaction::SignedTransaction;
use std::time::{SystemTime};
use crate::types::state::State;

pub struct Blockchain {
    map : HashMap<H256, Block>,
    tip : Block,
    states : HashMap<H256, State>,
}

impl Blockchain {
    /// Create a new blockchain, only containing the genesis block
    pub fn new() -> Self {
        //let mut rng = rand::thread_rng();
        //let n = rng.gen_range(0..u32::MAX);
        let n = 5;
        let d = hex!("0003ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff").into();
        
        let now = SystemTime::now();
        // let time = now.duration_since(UNIX_EPOCH).unwrap().as_millis();
        let time = 0;
        let transactions : Vec<SignedTransaction> =   Vec:: new();
        let gen = 0;
        let zeros : [u8; 32] = [0; 32];
        let p = H256::from(zeros);

        let genesis =  Block :: new(p, n, d, time, gen, transactions);
        let mut t = genesis.clone();
        let mut m : HashMap<H256, Block> = HashMap :: new();
        let mut s : HashMap<H256, State> = HashMap :: new();

        m.insert(genesis.hash(), genesis);
        Self {
            map : m,
            tip : t,
            states: s,
        }
    }

    /// Insert a block into blockchain
    pub fn insert(&mut self, block: &Block, state: State) {
        let len : u32 = self.map.get(&block.get_parent()).unwrap().get_to_genesis();
        let mut new_block = block.clone();
        new_block.set_to_genesis(len + 1);
        if new_block.get_to_genesis() > self.tip.get_to_genesis() {
            self.tip = new_block.clone();
        }
        let bhash = new_block.hash();
        self.map.insert(new_block.hash(), new_block);
        self.states.insert(bhash, state.clone());
    }

    /// Get the last block's hash of the longest chain
    pub fn tip(&self) -> H256 {
        return self.tip.hash();
    }

    /// Get the block from map corresponding to the given hash
    pub fn get_block(&self, hash:H256) -> Block {
        return self.map.get(&hash).unwrap().clone();
    }

    /// Get all blocks' hashes of the longest chain, ordered from genesis to the tip
    pub fn all_blocks_in_longest_chain(&self) -> Vec<H256> {
        let mut blocks : Vec<H256> = Vec::new();
        blocks.push(self.tip.hash());
        let mut parent = self.tip.get_parent();
        let zeros : [u8; 32] = [0; 32];
        let empty = H256::from(zeros);
        if parent.eq(&empty) {
            return blocks;
        }
        while !parent.eq(&empty) {
            blocks.insert(0, parent);
            parent = self.map.get(&parent).unwrap().get_parent();
        }
        return blocks;
    }

    pub fn contains_block(&self, hash: H256) -> bool {
        self.map.contains_key(&hash)
    }

}

// DO NOT CHANGE THIS COMMENT, IT IS FOR AUTOGRADER. BEFORE TEST


#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::block::generate_random_block;
    use crate::types::hash::Hashable;

    #[test]
    fn insert_one() {
        let mut blockchain = Blockchain::new();
        let genesis_hash = blockchain.tip();
        let block = generate_random_block(&genesis_hash);
        blockchain.insert(&block);
        assert_eq!(blockchain.tip(), block.hash());

    }

    #[test]
    fn new_blockchain() {
        let blockchain = Blockchain::new();
        assert_eq!(blockchain.map.len(), 1); // Should only contain the genesis block
    }

    #[test]
    fn insert_multiple() {
        let mut blockchain = Blockchain::new();
        let mut last_hash = blockchain.tip();

        for _ in 0..5 {
            let new_block = generate_random_block(&last_hash);
            blockchain.insert(&new_block);
            last_hash = new_block.hash();
        }

        assert_eq!(blockchain.tip(), last_hash);
    }

    #[test]
    fn longest_chain() {
        let mut blockchain = Blockchain::new();
        let genesis_hash = blockchain.tip();
        
        let block1 = generate_random_block(&genesis_hash);
        blockchain.insert(&block1);

        let block2 = generate_random_block(&block1.hash());
        blockchain.insert(&block2);

        let longest_chain = blockchain.all_blocks_in_longest_chain();

        assert_eq!(longest_chain, vec![genesis_hash, block1.hash(), block2.hash()]);
    }

    #[test]
    fn test_tip_after_insert() {
        let mut blockchain = Blockchain::new();
        let genesis_hash = blockchain.tip();
        
        let block1 = generate_random_block(&genesis_hash);
        blockchain.insert(&block1);

        let block2 = generate_random_block(&genesis_hash);
        blockchain.insert(&block2);

        assert_eq!(blockchain.tip(), block1.hash());
    }


    fn check_genesis_block() {
        let blockchain = Blockchain::new();
        let zeros: [u8; 32] = [0; 32];
        let p = H256::from(zeros);
        let genesis_block = generate_random_block(&p);
        assert_eq!(blockchain.map.get(&genesis_block.hash()).unwrap().hash(), genesis_block.hash());
    }


    #[test]
    fn check_tip_after_forks() {
        let mut blockchain = Blockchain::new();
        let genesis_hash = blockchain.tip();

        let fork1_block1 = generate_random_block(&genesis_hash);
        blockchain.insert(&fork1_block1);

        let fork1_block2 = generate_random_block(&fork1_block1.hash());
        blockchain.insert(&fork1_block2);

        let fork2_block1 = generate_random_block(&genesis_hash);
        blockchain.insert(&fork2_block1);

        assert_eq!(blockchain.tip(), fork1_block2.hash());
    }


    #[test]
    fn check_longest_chain_after_forks() {
        let mut blockchain = Blockchain::new();
        let genesis_hash = blockchain.tip();

        let fork1_block1 = generate_random_block(&genesis_hash);
        blockchain.insert(&fork1_block1);

        let fork1_block2 = generate_random_block(&fork1_block1.hash());
        blockchain.insert(&fork1_block2);

        let fork2_block1 = generate_random_block(&genesis_hash);
        blockchain.insert(&fork2_block1);

        let longest_chain = blockchain.all_blocks_in_longest_chain();
        assert_eq!(longest_chain, vec![genesis_hash, fork1_block1.hash(), fork1_block2.hash()]);
    }
}

// DO NOT CHANGE THIS COMMENT, IT IS FOR AUTOGRADER. AFTER TEST