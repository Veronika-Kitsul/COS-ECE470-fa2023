use serde::{Serialize,Deserialize};
use ring::signature;
use ring::signature::{Ed25519KeyPair, Signature};
use rand::Rng;
use crate::types::address::Address;
use crate::types::hash::{Hashable, H256};
use crate::types::block;

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct Transaction {
    Receiver: Address,
    Value: i32,
    nonce: u32
}

#[derive(Serialize, Deserialize, Debug, Default, Clone, Hashable)]
pub struct SignedTransaction {
    signer_pk : [u8; 32],
    transaction : Transaction,
    signature : Vec<u8>,
}

/// Create digital signature of a transaction
pub fn sign(t: &Transaction, key: &Ed25519KeyPair) -> Signature {
    let serialized_t_str = serde_json::to_string(&t).unwrap();
    let serialized_t = serialized_t_str.as_bytes();
    let signature = key.sign(&serialized_t);
    signature
}

/// Verify digital signature of a transaction, using public key instead of secret key
pub fn verify(t: &Transaction, public_key: &[u8], signature: &[u8]) -> bool {
    let serialized_t_str = serde_json::to_string(&t).unwrap();
    let serialized_t = serialized_t_str.as_bytes();
    let public_key = ring::signature::UnparsedPublicKey::new(&ring::signature::ED25519, public_key);
    public_key.verify(serialized_t, signature).is_ok()
}
/*
#[cfg(any(test, test_utilities))]
pub fn generate_random_transaction() -> Transaction {
    let mut rng = rand::thread_rng();
    let value = rng.gen_range(0..i32::MAX);
    let sender: [u8; 32] = rng.gen();
    let receiver: [u8; 32] = rng.gen();
    
    Transaction {
        Sender: Address::from_public_key_bytes(&sender), 
        Receiver: Address::from_public_key_bytes(&receiver), 
        Value: value
    }
}

// DO NOT CHANGE THIS COMMENT, IT IS FOR AUTOGRADER. BEFORE TEST

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::key_pair;
    use ring::signature::KeyPair;


    #[test]
    fn sign_verify() {
        let t = generate_random_transaction();
        let key = key_pair::random();
        let signature = sign(&t, &key);
        assert!(verify(&t, key.public_key().as_ref(), signature.as_ref()));
    }
    #[test]
    fn sign_verify_two() {
        let t = generate_random_transaction();
        let key = key_pair::random();
        let signature = sign(&t, &key);
        let key_2 = key_pair::random();
        let t_2 = generate_random_transaction();
        assert!(!verify(&t_2, key.public_key().as_ref(), signature.as_ref()));
        assert!(!verify(&t, key_2.public_key().as_ref(), signature.as_ref()));
    }
}
*/
// DO NOT CHANGE THIS COMMENT, IT IS FOR AUTOGRADER. AFTER TEST