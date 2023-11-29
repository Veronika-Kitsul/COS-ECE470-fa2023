use crossbeam::channel::{Receiver};
use log::{info};
use crate::types::block::Block;
use crate::network::server::Handle as ServerHandle;
use std::thread;
use std::sync::{Arc, Mutex};
use crate::blockchain::Blockchain;
use crate::network::message::Message;
use crate::types::hash::{H256, Hashable};
use crate::types::transaction;
use crate::types::transaction::{Transaction, verify};
use crate::types::mempool::Mempool;
use crate::types::state::State;

#[derive(Clone)]
pub struct Worker {
    server: ServerHandle,
    finished_block_chan: Receiver<Block>,
    blockchain : Arc<Mutex<Blockchain>>,
    mempool: Arc<Mutex<Mempool>>,
    state: Arc<Mutex<State>>,
}

impl Worker {
    pub fn new(
        server: &ServerHandle,
        finished_block_chan: Receiver<Block>,
        bc: &Arc<Mutex<Blockchain>>,
        mp: &Arc<Mutex<Mempool>>,
        st: &Arc<Mutex<State>>,
    ) -> Self {
        Self {
            server: server.clone(),
            finished_block_chan,
            blockchain : Arc::clone(bc),
            mempool : Arc::clone(mp),
            state : Arc::clone(st),
        }
    }

    pub fn start(self) {
        thread::Builder::new()
            .name("miner-worker".to_string())
            .spawn(move || {
                self.worker_loop();
            })
            .unwrap();
        info!("Miner initialized into paused mode");
    }

    fn worker_loop(&self) {
        loop {
            let ablock = self.finished_block_chan.recv().expect("Receive finished block error");
            let mut block_vec : Vec<H256> = Vec::new();
            let mut bstate: State;
            let mut is_valid = true;

            {
                let mut state_lock = self.state.lock().unwrap();
                bstate = state_lock.clone();
            }
            for trans in ablock.get_transactions() {
                let r = trans.transaction.Receiver;
                let s = trans.transaction.Sender;
                
                //transaction checks
                if !bstate.contains_account(s) || 
                    !transaction::verify(&trans.transaction, trans.signer_pk.as_ref(), trans.signature.as_ref()) ||
                    (bstate.get_value(s) - trans.transaction.Value) < 0  {

                    is_valid = false;
                    break;
                }
                else {
                    let r = trans.transaction.Receiver;
                    let s = trans.transaction.Sender;
                    let nonce = bstate.get_nonce(s);
                    let val = bstate.get_value(s);
                    bstate.add_account(s, nonce + 1, val - trans.transaction.Value);

                    // receiver
                    if bstate.contains_account(r) {
                        let nonce = bstate.get_nonce(r);
                        let val = bstate.get_value(r);
                        bstate.add_account(r, nonce,  val + trans.transaction.Value);
                    }
                    else {
                        bstate.add_account(r, 0, trans.transaction.Value);
                    }
                }                                
            }

            // add block
            if is_valid {
                print!("valid block in miner worker\n");
                
                {
                    let mut state_lock = self.state.lock().unwrap();
                    *state_lock = bstate.clone();
                }

                    

                // mempool copy for individual blocks
                let mempool_copy;
                {
                    let mut mempool_lock = self.mempool.lock().unwrap();
                    for t in ablock.get_transaction_hashes() {
                        mempool_lock.rm_transaction(t);
                    }
                    mempool_copy = mempool_lock.transactions.clone();
                }

                // can't loop through mempool - avoid nested locks, so created copy
                let mut thashes: Vec<H256> = Vec::new();
                for (key, trans) in mempool_copy {
                    {
                        let state_lock = self.state.lock().unwrap();
                        let r = trans.transaction.Receiver;
                        let s = trans.transaction.Sender;

                        if (!state_lock.contains_account(s) || 
                        !transaction::verify(&trans.transaction, trans.signer_pk.as_ref(), trans.signature.as_ref()) ||
                        state_lock.get_value(s) < trans.transaction.Value) {
                            thashes.push(trans.hash());
                        }
                    }
                }

                {
                    let mut mempool_lock = self.mempool.lock().unwrap();
                    for t in thashes {
                        mempool_lock.rm_transaction(t);
                    }
                }

                {
                    let mut blockchain_lock = self.blockchain.lock().unwrap();
                    print!("adding block to the chain in miner worker\n");
                    blockchain_lock.insert(&ablock, bstate);
                    block_vec.push(ablock.hash());
                    self.server.broadcast(Message:: NewBlockHashes(block_vec));
                }
            } 
        }
    }
}
