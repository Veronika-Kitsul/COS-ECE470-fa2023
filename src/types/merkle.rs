use super::hash::{Hashable, H256};
use ring::digest;

/// A Merkle tree.
#[derive(Debug, Default)]
pub struct MerkleTree {
    tree: Vec<Vec<H256>>,
}

impl MerkleTree {
    pub fn new<T>(data: &[T]) -> Self where T: Hashable + Clone, {

        let mut t: Vec<Vec<H256>> = Vec::new();

        // if the array input is empty -- return a tree with a hash created out of
        // an array of zeros
        if data.len() == 0 {
            let mut empty : Vec<H256> = Vec::new();
            let zeros : [u8; 32] = [0; 32];
            empty.push(H256::from(zeros));
            t.push(empty);
            let mTree : MerkleTree = MerkleTree{tree: t};
            return mTree;
        }
        else if data.len() == 1 {
            // if the length of input array = 1 -- hash and let it be the tree
            let mut empty : Vec<H256> = Vec::new();
            empty.push(data[0].hash());
            t.push(empty);
            let mTree : MerkleTree = MerkleTree{tree: t};
            return mTree;
        }
        
        // hash the inputs
        let mut prev_row: Vec<H256> = data.iter().map(|item| item.hash()).collect();

        if prev_row.len() % 2 == 1
        {
            prev_row.push(prev_row[prev_row.len() - 1].clone());
        }
        t.insert(0, prev_row.clone());

        while prev_row.len() > 1 {
            
            let mut row : Vec<H256> = Vec::new();

            for i in (0..prev_row.len()).step_by(2) {
                
                // concatenating 
                let mut concat : [u8; 64] = [0; 64];
                let hash1 = prev_row[i].as_ref();
                let hash2 = prev_row[i + 1].as_ref();
                concat[..32].copy_from_slice(&hash1);
                concat[32..].copy_from_slice(&hash2);

                // hash the concatenated hashes 
                let hash = digest::digest(&digest::SHA256, &concat);
                let hash_bytes = hash.as_ref();
                let mut hash_array = [0u8; 32];
                hash_array.copy_from_slice(&hash_bytes[0..32]);
                let new_hash = H256::from(hash_array);

                // push each hash to the row
                row.push(new_hash);
            }

            // check for odd length in the row and duplicate the last one if true
            if row.len() % 2 == 1 && row.len() != 1 {
                row.push(row[row.len() - 1]);
            }

            // insert row into the tree
            t.insert(0, row.clone());
            prev_row = row;
        }

        let mTree : MerkleTree = MerkleTree{tree: t};
        mTree
    }

    pub fn root(&self) -> H256 {
        // just return the root
        self.tree[0][0]
    }

    /// Returns the Merkle Proof of data at index i
    pub fn proof(&self, index: usize) -> Vec<H256> {
        let mut sib : Vec<H256> = Vec :: new();
        let mut empty : Vec<H256> = Vec::new();
        let zeros : [u8; 32] = [0; 32];
        empty.push(H256::from(zeros));

        if self.tree[0][0].eq(&empty[0]) || self.tree.len() == 1 || index >= self.tree[self.tree.len() - 1].len() || index < 0 {
            return sib;
        }

        if index%2 == 0{
            sib.push(self.tree[self.tree.len() - 1][index + 1]);
        }
        else {
            sib.push(self.tree[self.tree.len() - 1][index - 1]);
        }

        if self.tree.len() == 2{
            return sib;
        }

        let mut position = index/2;
        for i in (1..self.tree.len() - 1).rev() {
            if position%2 == 0{
                sib.push(self.tree[i][position + 1]);
            }
            else{
                sib.push(self.tree[i][position - 1]);
            }
            position = position/2;
        }
        return sib;
    }
}

/// Verify that the datum hash with a vector of proofs will produce the Merkle root. Also need the
/// index of datum and `leaf_size`, the total number of leaves.
pub fn verify(root: &H256, datum: &H256, proof: &[H256], index: usize, leaf_size: usize) -> bool {
    if leaf_size == 0 || index >= leaf_size {
        return false;
    }
    if leaf_size == 1 {
        return root.eq(datum);
    }

    let mut next : H256;
    next = datum.clone();
    let mut position = index;

    for i in 0..proof.len() {
        let mut concat : [u8; 64] = [0; 64];
        let mut hash1: [u8; 32] = [0; 32];
        let mut hash2: [u8; 32] = [0; 32];

        if position % 2 == 0 {
            hash1.copy_from_slice(next.as_ref());
            hash2.copy_from_slice(proof[i].as_ref());
        }
        else {
            hash2.copy_from_slice(next.as_ref());
            hash1.copy_from_slice(proof[i].as_ref());
        }
        
        concat[..32].copy_from_slice(&hash1);
        concat[32..].copy_from_slice(&hash2);

        let hash = digest::digest(&digest::SHA256, &concat);
        let hash_bytes = hash.as_ref();
        let mut hash_array = [0u8; 32];
        hash_array.copy_from_slice(&hash_bytes[0..32]);
        next = H256::from(hash_array);
        position = position / 2;
    }

    return root.eq(&next);
}

// DO NOT CHANGE THIS COMMENT, IT IS FOR AUTOGRADER. BEFORE TEST

#[cfg(test)]
mod tests {
    use crate::types::hash::H256;
    use super::*;

    macro_rules! gen_merkle_tree_data {
        () => {{
            vec![
                (hex!("0a0b0c0d0e0f0e0d0a0b0c0d0e0f0e0d0a0b0c0d0e0f0e0d0a0b0c0d0e0f0e0d")).into(),
                (hex!("0101010101010101010101010101010101010101010101010101010101010202")).into(),
            ]
        }};
    }

    #[test]
    fn merkle_root() {
        let input_data: Vec<H256> = gen_merkle_tree_data!();
        let merkle_tree = MerkleTree::new(&input_data);
        let root = merkle_tree.root();
        assert_eq!(
            root,
            (hex!("6b787718210e0b3b608814e04e61fde06d0df794319a12162f287412df3ec920")).into()
        );
        // "b69566be6e1720872f73651d1851a0eae0060a132cf0f64a0ffaea248de6cba0" is the hash of
        // "0a0b0c0d0e0f0e0d0a0b0c0d0e0f0e0d0a0b0c0d0e0f0e0d0a0b0c0d0e0f0e0d"
        // "965b093a75a75895a351786dd7a188515173f6928a8af8c9baa4dcff268a4f0f" is the hash of
        // "0101010101010101010101010101010101010101010101010101010101010202"
        // "6b787718210e0b3b608814e04e61fde06d0df794319a12162f287412df3ec920" is the hash of
        // the concatenation of these two hashes "b69..." and "965..."
        // notice that the order of these two matters
    }

    #[test]
    fn merkle_proof() {
        let input_data: Vec<H256> = gen_merkle_tree_data!();
        let merkle_tree = MerkleTree::new(&input_data);
        let proof = merkle_tree.proof(0);
        assert_eq!(proof,
                   vec![hex!("965b093a75a75895a351786dd7a188515173f6928a8af8c9baa4dcff268a4f0f").into()]
        );
        // "965b093a75a75895a351786dd7a188515173f6928a8af8c9baa4dcff268a4f0f" is the hash of
        // "0101010101010101010101010101010101010101010101010101010101010202"
    }

    #[test]
    fn merkle_verifying() {
        let input_data: Vec<H256> = gen_merkle_tree_data!();
        let merkle_tree = MerkleTree::new(&input_data);
        let proof = merkle_tree.proof(0);
        assert!(verify(&merkle_tree.root(), &input_data[0].hash(), &proof, 0, input_data.len()));
    }

    #[test]
    fn verify_empty_tree() {
        let root = H256::from([0u8; 32]);  // Root of empty tree is zeros
        let datum = H256::from([0u8; 32]); 
        let proof = vec![];
        let index = 0;
        let leaf_size = 0;
        assert!(!verify(&root, &datum, &proof, index, leaf_size));
    }

    #[test]
    fn verify_single_leaf() {
        let root = H256::from([1u8; 32]);  // Assuming this is the hashed value of the single leaf
        let datum = H256::from([1u8; 32]); // Same as root for a single leaf tree
        let proof = vec![];
        let index = 0;
        let leaf_size = 1;
        assert!(verify(&root, &datum, &proof, index, leaf_size));
    }

    #[test]
    fn verify_with_invalid_index() {
        let root = H256::from([5u8; 32]);  // Hypothetical root
        let datum = H256::from([1u8; 32]); // Hypothetical leaf
        let proof = vec![H256::from([2u8; 32])];  // Hypothetical sibling hash
        let index = 5;
        let leaf_size = 2;
        assert!(!verify(&root, &datum, &proof, index, leaf_size));
    }

    #[test]
    fn verify_with_invalid_proof() {
        let root = H256::from([5u8; 32]);  // Hypothetical root
        let datum = H256::from([1u8; 32]); // Hypothetical leaf
        let proof = vec![H256::from([99u8; 32]), H256::from([100u8; 32])];  // Incorrect proof
        let index = 0;
        let leaf_size = 4;
        assert!(!verify(&root, &datum, &proof, index, leaf_size));
    }


    fn hash_u8_to_h256(item: u8) -> H256 {
        let hash = digest::digest(&digest::SHA256, &[item]);
        let hash_bytes = hash.as_ref();
        let mut hash_array = [0u8; 32];
        hash_array.copy_from_slice(&hash_bytes[0..32]);
        H256::from(hash_array)  // Assuming H256 can be initialized this way
    }

    // tests from OH 
    macro_rules! gen_merkle_tree_data_8 {
        () => {{
            vec![
                (hex!("1111111111111111111111111111111111111111111111111111111111111111")).into(),
                (hex!("2222222222222222222222222222222222222222222222222222222222222222")).into(),
                (hex!("3333333333333333333333333333333333333333333333333333333333333333")).into(),
                (hex!("4444444444444444444444444444444444444444444444444444444444444444")).into(),
                (hex!("5555555555555555555555555555555555555555555555555555555555555555")).into(),
                (hex!("6666666666666666666666666666666666666666666666666666666666666666")).into(),
                (hex!("7777777777777777777777777777777777777777777777777777777777777777")).into(),
                (hex!("8888888888888888888888888888888888888888888888888888888888888888")).into(),
            ]
        }};
    }

    #[test]
    fn merkle_root_8() {
        let input_data: Vec<H256> = gen_merkle_tree_data_8!();
        let merkle_tree = MerkleTree::new(&input_data);
        let root = merkle_tree.root();
        assert_eq!(
            root,
            (hex!("f6f30766a7f45acc7e2d29b1ea939a0b1f37e7aee16563e0560c6d68a33422f6")).into()
        );
    }

    #[test]
    fn merkle_proof_8() {
        let input_data: Vec<H256> = gen_merkle_tree_data_8!();
        let merkle_tree = MerkleTree::new(&input_data);
        let proof = merkle_tree.proof(6);
        assert_eq!(proof,
                   vec![
                    (hex!("e8b72e0b71c772f8398f58104fb04c243c67ec46014cb06c22329c49b03d4c86")).into(),
                    (hex!("32d87e2145ddc3ec123fc116d28a26b904d77f0aa0ac98b6dd5e1679217cfea0")).into(),
                    (hex!("106b7b96a5197fda2e53fb6ec68892d0af89fa35a64fd2ad70c462098668cb07")).into(),
                   ]
        );
    }

    #[test]
    fn merkle_verifying_8() {
        let input_data: Vec<H256> = gen_merkle_tree_data_8!();
        let merkle_tree = MerkleTree::new(&input_data);
        let proof = merkle_tree.proof(6);
        assert!(verify(&merkle_tree.root(), &input_data[6].hash(), &proof, 6, input_data.len()));
    }
}

// DO NOT CHANGE THIS COMMENT, IT IS FOR AUTOGRADER. AFTER TEST