use crossbeam::channel::{unbounded, Receiver, Sender, TryRecvError};
use log::{debug, info};
use crate::types::block::Block;
use crate::network::server::Handle as ServerHandle;
use std::thread;
use std::sync::{Arc, Mutex};
use crate::blockchain::Blockchain;
use std::collections::HashMap;
use crate::network::message::Message;
use crate::types::hash::{H256, Hashable};
use crate::types::mempool::Mempool;

#[derive(Clone)]
pub struct Worker {
    server: ServerHandle,
    finished_block_chan: Receiver<Block>,
    blockchain : Arc<Mutex<Blockchain>>,
    mempool: Arc<Mutex<Mempool>>,
}

impl Worker {
    pub fn new(
        server: &ServerHandle,
        finished_block_chan: Receiver<Block>,
        bc: &Arc<Mutex<Blockchain>>,
        mp: &Arc<Mutex<Mempool>>,
    ) -> Self {
        Self {
            server: server.clone(),
            finished_block_chan,
            blockchain : Arc::clone(bc),
            mempool : Arc::clone(mp),
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
            {
                let mut mempool_lock = self.mempool.lock().unwrap();
                for transaction in _block.get_transactions(){
                    mempool_lock.rm_transaction(transaction.hash());
                }
            }
            {
                let mut blockchain_lock = self.blockchain.lock().unwrap();
                blockchain_lock.insert(&_block);
                block_vec.push(_block.hash());
                self.server.broadcast(Message:: NewBlockHashes(block_vec));
            }
        }
    }
}
