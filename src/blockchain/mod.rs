use crate::types::block::Block;
use crate::types::hash::H256;
use std::collections::HashMap;
use std::ptr;
use rand::Rng;
use std::time::Instant;

pub struct Blockchain {
    map : HashMap,
    tip : Block,
}

impl Blockchain {
    /// Create a new blockchain, only containing the genesis block
    pub fn new() -> Self {
        let p : *H256 = ptr::null();
        let genesis : Block :: generate_random_block(p);
        let mut m = HashMap :: new();
        m.insert(genesis.hash(), genesis);
        Self {
            map : m,
            tip : genesis,
        }
    }

    /// Insert a block into blockchain
    pub fn insert(&mut self, block: &Block) {
        let len : u32 = self.map.get(*block.get_parent().hash()).get_to_genesis()
        block.set_to_genesis(len + 1);
        if block.get_to_genesis() > tip.get_to_genesis() {
            tip = block;
        }
        self.map.insert(block.hash(), block);
    }

    /// Get the last block's hash of the longest chain
    pub fn tip(&self) -> H256 {
        return self.tip.hash();
    }

    /// Get all blocks' hashes of the longest chain, ordered from genesis to the tip
    pub fn all_blocks_in_longest_chain(&self) -> Vec<H256> {
        let mut blocks : Vec<H256> :: new();
        blocks.push(self.tip.hash());
        let mut parent = self.tip.get_parent();
        if parent != ptr::null() {
            return blocks;
        }
        while (parent != ptr::null()) {
            blocks.insert(*parent.hash());
            parent = parent.get_parent();
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