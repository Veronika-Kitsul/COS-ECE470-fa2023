use crate::types::block::{Block};
use crate::types::block;
use crate::types::hash::{H256, Hashable};
use std::collections::HashMap;
use std::ptr;

pub struct Blockchain {
    map : HashMap<H256, Block>,
    tip : Block,
}

impl Blockchain {
    /// Create a new blockchain, only containing the genesis block
    pub fn new() -> Self {
        let zeros : [u8; 32] = [0; 32];
        let p = H256::from(zeros);
        let genesis = block :: generate_random_block(&p);
        let mut t = genesis.clone();
        let mut m : HashMap<H256, Block> = HashMap :: new();
        m.insert(genesis.hash(), genesis);
        Self {
            map : m,
            tip : t,
        }
    }

    /// Insert a block into blockchain
    pub fn insert(&mut self, block: &Block) {
        let len : u32 = self.map.get(&block.get_parent()).unwrap().get_to_genesis();
        let mut new_block = block.clone();
        new_block.set_to_genesis(len + 1);
        if new_block.get_to_genesis() > self.tip.get_to_genesis() {
            self.tip = new_block.clone();
        }
        self.map.insert(new_block.hash(), new_block);
    }

    /// Get the last block's hash of the longest chain
    pub fn tip(&self) -> H256 {
        return self.tip.hash();
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
}

// DO NOT CHANGE THIS COMMENT, IT IS FOR AUTOGRADER. AFTER TEST