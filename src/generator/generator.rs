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
use crate::types::transaction::verify;
use crate::types::address::Address;
use crate::types::mempool::Mempool;
use crate::blockchain::Blockchain;
use crate::types::state::{State, create_seed_array}; 
use crate::network::server::Handle as ServerHandle;
use crate::network::message::Message;
use ring::signature::KeyPair;


#[derive(Clone)]
pub struct TransactionGenerator {
    mempool: Arc<Mutex<Mempool>>,
    blockchain: Arc<Mutex<Blockchain>>,
    state: Arc<Mutex<State>>,
    server: ServerHandle,
    seed : [u8; 32],
}

impl TransactionGenerator {
    pub fn new(mp:&Arc<Mutex<Mempool>>, bc: &Arc<Mutex<Blockchain>>, 
        st: &Arc<Mutex<State>>, server: &ServerHandle, sd: u16) -> Self {
        Self {
            mempool: Arc::clone(mp),
            blockchain : Arc::clone(bc),
            state: Arc::clone(st),
            server: server.clone(),
            seed: create_seed_array(sd),
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
            print!("BEGINNING OF GENERATOR\n");
            
            // nonce is always 0 to start with
            let n = 0;
            
            let key_pair = Ed25519KeyPair::from_seed_unchecked(&self.seed).unwrap();
            let public_key_bytes = key_pair.public_key().as_ref();

            let mut nonce;
            let mut value;

            let mut sender;
            let mut sender_value;

            let mut rng = rand::thread_rng();
            {
                // generate sender
                let state_lock = self.state.lock().unwrap();

                let mut index = rng.gen_range(0..state_lock.accounts.len());
                print!("generated index\n");
                sender = state_lock.accounts.keys().nth(index).unwrap().clone(); 
                print!("generated sender\n");
                sender_value = state_lock.get_value(sender);

                // if sender value ,= 0, regenerate one
                while sender_value <= 0 {
                    print!("im in sender gen loop\n");
                    index = rng.gen_range(0..state_lock.accounts.len());
                    sender = state_lock.accounts.keys().nth(index).unwrap().clone(); 
                    sender_value = state_lock.get_value(sender);
                } 

                nonce = state_lock.get_nonce(sender) + 1;
            }
            print!("sender val {:?}\n", sender_value);
            value = rng.gen_range(0..sender_value/10 + 1);
            

            // generate receiver with a probability 80 new, rest - old
            let probability = rng.gen_range(0..100);
            let mut receiver;
            if (probability < 25) {
                // generate the address for receiver
                let random_seed: [u8; 32] = rng.gen();
                let rec_key_pair = Ed25519KeyPair::from_seed_unchecked(&random_seed).unwrap();
                let rec_public_key_bytes = rec_key_pair.public_key().as_ref();
                receiver = Address::from_public_key_bytes(rec_public_key_bytes);
            }
            else {
                {
                    let state_lock = self.state.lock().unwrap();
                    let mut index = rng.gen_range(0..state_lock.accounts.len());
                    receiver = state_lock.accounts.keys().nth(index).unwrap().clone(); 
                }
            }
            
            let trans = Transaction {
                Sender: sender, 
                Receiver: receiver, 
                Value: value,
                nonce: n,
            };

            let sign = sign(&trans, &key_pair);
            let signature_vector = sign.as_ref();
            let sign_vec = signature_vector.to_vec();

            let signed = SignedTransaction {
                signer_pk : public_key_bytes.to_vec(),
                transaction : trans,
                signature : sign_vec,
            };

            print!("generator before the ifs\n");
            if (!sender.eq(&receiver) && value != 0) {
                // broadcast this right here
                let mut trans_vec: Vec<H256> = Vec:: new();
                trans_vec.push(signed.hash());
                self.server.broadcast(Message:: NewTransactionHashes(trans_vec));

                print!("transaction val: {:?}\n", signed.transaction.Value);
                print!("creating transaction\n");
                {
                    let mut mempool_lock = self.mempool.lock().unwrap();
                    mempool_lock.add_transaction(signed);
                    print!("mempool size: {:?}\n", mempool_lock.transactions.keys().len());
                }

                if theta != 0 {
                    let interval = time::Duration::from_millis(10 * theta);
                    thread::sleep(interval);
                }
            }
            else {
                print!("i am NOT creating transaction\n");
            }
        }
    }
}
