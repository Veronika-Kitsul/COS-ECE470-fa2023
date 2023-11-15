use log::info;
use std::time;
use std::thread;
use ring::signature::{Ed25519KeyPair, Signature};
use ring::signature;
use rand::{Rng, thread_rng};
use ring::rand::SystemRandom;
use ring::rand::SecureRandom;
use std::sync::{Arc, Mutex};
use crate::types::transaction::{Transaction, SignedTransaction};
use crate::types::transaction::sign;
use crate::types::hash::{H256, Hashable};
use crate::types::address::Address;
use crate::types::mempool::Mempool;
use crate::network::server::Handle as ServerHandle;
use crate::network::message::Message;
use ring::signature::KeyPair;


#[derive(Clone)]
pub struct TransactionGenerator {
    mempool: Arc<Mutex<Mempool>>,
    server: ServerHandle,
}

impl TransactionGenerator {
    pub fn new(mp:&Arc<Mutex<Mempool>>, server: &ServerHandle) -> Self {
        Self {
            mempool: Arc::clone(mp),
            server: server.clone(),
        }
    }

    pub fn start(self, theta: u64) {
        thread::Builder::new()
            .name("transaction-generator".to_string())
            .spawn(move || {
                self.generate_transactions(theta);
            })
            .unwrap();
        info!("Transaction generator started");
    }

    fn generate_transactions(&self, theta: u64) {
        loop {
            // create transaction
            let mut rng = rand::thread_rng();
            let value = rng.gen_range(0..i32::MAX);
            let n = 0;
            let sender: [u8; 32] = rng.gen();
            let receiver: [u8; 32] = rng.gen();
            
            let trans = Transaction {
                Sender: Address::from_public_key_bytes(&sender), 
                Receiver: Address::from_public_key_bytes(&receiver), 
                Value: value,
                nonce: n,
            };

            // Generate a random seed.
            let rng = SystemRandom::new();
            let mut seed = [0u8; 32];
            rng.fill(&mut seed);

            // Generate a key pair based on the random seed.
            let key_pair = signature::Ed25519KeyPair::from_seed_unchecked(&seed).expect("Key pair generation failed");
            let pk = key_pair.public_key().as_ref().to_vec();            

            let sign = sign(&trans, &key_pair);
            let signature_vector: Vec<u8> = sign.as_ref().to_vec();

            let signed = SignedTransaction {
                signer_pk : pk,
                transaction : trans,
                signature : signature_vector,
            };


            // broadcast this right here
            let mut trans_vec: Vec<H256> = Vec:: new();
            trans_vec.push(signed.hash());
            self.server.broadcast(Message:: NewTransactionHashes(trans_vec));
            {
                let mut mempool_lock = self.mempool.lock().unwrap();
                mempool_lock.add_transaction(signed);
            }

            if theta != 0 {
                let interval = time::Duration::from_millis(10 * theta);
                thread::sleep(interval);
            }
        }
    }
}
