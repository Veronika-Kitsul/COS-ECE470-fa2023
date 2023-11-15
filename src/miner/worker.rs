use crossbeam::channel::{unbounded, Receiver, Sender, TryRecvError};
use log::{debug, info};
use crate::types::block::Block;
use crate::network::server::Handle as ServerHandle;
use std::thread;
use std::sync::{Arc, Mutex};
use crate::blockchain::Blockchain;
use crate::network::message::Message;
use crate::types::hash::{H256, Hashable};
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
            let _block = self.finished_block_chan.recv().expect("Receive finished block error");
            let mut block_vec : Vec<H256> = Vec::new();
            let mut bstate: State;
            {
                let mut state_lock = self.state.lock().unwrap();
                for trans in _block.get_transactions() {
                    let r = trans.transaction.Receiver;
                    let s = trans.transaction.Sender;

                    // receiver
                    if (state_lock.contains_account(r)) {
                        state_lock.add_account(r, state_lock.get_nonce(r), state_lock.get_value(r) + trans.transaction.Value);
                    }
                    else {
                        state_lock.add_account(r, 0, trans.transaction.Value);
                    }

                    // sender
                    if (state_lock.contains_account(s)) {
                        state_lock.add_account(s, state_lock.get_nonce(s) + 1, state_lock.get_value(s) - trans.transaction.Value);
                    }
                    else {
                        state_lock.add_account(s, 1, -trans.transaction.Value);
                    }
                }
                bstate = state_lock.clone();
            }
            {
                let mut blockchain_lock = self.blockchain.lock().unwrap();
                blockchain_lock.insert(&_block, bstate);
                block_vec.push(_block.hash());
                self.server.broadcast(Message:: NewBlockHashes(block_vec));
            }
        }
    }
}
