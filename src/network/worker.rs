use crate::network::message::Message;
use super::peer;
use super::server::Handle as ServerHandle;
use crate::types::hash::{H256, Hashable};
use std::sync::{Arc, Mutex};
use crate::blockchain::Blockchain;
use crate::types::block::Block;
use crate::types::state::State;
use crate::types::transaction;
use crate::network::worker::transaction::SignedTransaction;
use crate::types::transaction::{Transaction, verify};
use crate::types::mempool::Mempool;
use std::collections::HashMap;

use log::{debug, warn, error};

use std::thread;

#[cfg(any(test,test_utilities))]
use super::peer::TestReceiver as PeerTestReceiver;
#[cfg(any(test,test_utilities))]
use super::server::TestReceiver as ServerTestReceiver;
#[derive(Clone)]
pub struct Worker {
    msg_chan: smol::channel::Receiver<(Vec<u8>, peer::Handle)>,
    num_worker: usize,
    server: ServerHandle,
    blockchain: Arc<Mutex<Blockchain>>,
    mempool : Arc<Mutex<Mempool>>,
    state: Arc<Mutex<State>>,
}

impl Worker {
    pub fn new(
        num_worker: usize,
        msg_src: smol::channel::Receiver<(Vec<u8>, peer::Handle)>,
        server: &ServerHandle, bc: &Arc<Mutex<Blockchain>>, mp: &Arc<Mutex<Mempool>>, st: &Arc<Mutex<State>>
    ) -> Self {
        Self {
            msg_chan: msg_src,
            num_worker,
            server: server.clone(),
            blockchain: Arc::clone(bc),
            mempool : Arc::clone(mp),
            state : Arc::clone(st),
        }
    }

    pub fn start(self) {
        let num_worker = self.num_worker;
        for i in 0..num_worker {
            let cloned = self.clone();
            thread::spawn(move || {
                cloned.worker_loop();
                warn!("Worker thread {} exited", i);
            });
        }
    }

    fn worker_loop(&self) {
        let mut orphans : HashMap<H256, Block> = HashMap::new();
        loop {
            print!("IN NETWORK\n");
            let result = smol::block_on(self.msg_chan.recv());
            if let Err(e) = result {
                error!("network worker terminated {}", e);
                break;
            }

            let msg = result.unwrap();
            let (msg, mut peer) = msg;
            let msg: Message = bincode::deserialize(&msg).unwrap();
            match msg {
                Message::Ping(nonce) => {
                    debug!("Ping: {}", nonce);
                    peer.write(Message::Pong(nonce.to_string()));
                }
                Message::Pong(nonce) => {
                    debug!("Pong: {}", nonce);
                }
                Message::NewBlockHashes(hash_vector) => {
                    let mut blocks : Vec<H256> = Vec::new();
                    {
                        let blockchain_lock = self.blockchain.lock().unwrap();
                        for blockhash in hash_vector {
                            if !blockchain_lock.contains_block(blockhash) {
                                blocks.push(blockhash);
                            }
                        }
                    }
                    if blocks.len() > 0 {
                        peer.write(Message::GetBlocks(blocks));
                    }
                }
                Message::GetBlocks(hash_vector) => {
                    let mut blocks : Vec<Block> = Vec::new();
                    {
                        let blockchain_lock = self.blockchain.lock().unwrap();
                        for blockhash in hash_vector {
                            if blockchain_lock.contains_block(blockhash) {
                                blocks.push(blockchain_lock.get_block(blockhash));
                            }
                        }
                    }
                    if blocks.len() > 0 {
                        peer.write(Message::Blocks(blocks));
                    }
                }
                Message::Blocks(block_vec) => {
                    let mut blocks_added: Vec<Block> = Vec::new();

                    {
                        let blockchain_lock = self.blockchain.lock().unwrap();
                        for block in block_vec {

                            print!("in worker network blockchain lock contains block hash: {:?}", blockchain_lock.contains_block(block.hash()));
                            print!("in worker network difficulty works: {:?}", (block.hash() <= block.get_difficulty())); 
                        
                            if !blockchain_lock.contains_block(block.hash()) && (block.hash() <= block.get_difficulty()) {

                                if !blockchain_lock.contains_block(block.get_parent()) {
                                    peer.write(Message::GetBlocks(vec![block.get_parent()]));
                                    orphans.insert(block.get_parent(), block);
                                }
                                else {
                                    if block.get_difficulty() == blockchain_lock.get_block(block.get_parent()).get_difficulty() {
                                        //blockchain_lock.insert(&block);
                                        blocks_added.push(block.clone());
                                        //block_hashes.push(block.hash());
                                        let mut parent = block.hash();
                                        let mut pdiff = block.get_difficulty();

                                        while orphans.contains_key(&parent) {

                                            let oblock = orphans.get(&parent).unwrap().clone();
                                            if oblock.get_difficulty() == pdiff {
                                                //blockchain_lock.insert(&oblock);
                                                blocks_added.push(oblock.clone());
                                                //block_hashes.push(oblock.hash());
                                                orphans.remove(&parent);
                                                parent = oblock.hash();
                                            }
                                            else {
                                                orphans.remove(&parent);
                                                break
                                            }
                                        } 
                                    } 
                                }   
                            }
                        }
                    }
                    

                    //update state based on blocks to be added
                    let mut block_hashes : Vec<H256> = Vec::new();
                    for ablock in &blocks_added {
                        let mut is_valid = true;

                        for trans in ablock.get_transactions() {
                            let r = trans.transaction.Receiver;
                            let s = trans.transaction.Sender;
                                
                            {
                                print!("OSL miner worker 1\n");
                                let mut state_lock = self.state.lock().unwrap();
                                print!("SSL miner worker 1\n");
            
                                // //transaction checks
                                // print!("worker miner: state lock contains: {:?}\n", state_lock.contains_account(s));
                                // print!("worker miner: transaction verification: {:?}\n", 
                                //     transaction::verify(&trans.transaction, trans.signer_pk.as_ref(), trans.signature.as_ref()));
                                // print!("worker miner: transaction value: {:?} and account value: {:?}\n", 
                                //     state_lock.get_value(s), trans.transaction.Value);
            
                                if !state_lock.contains_account(s) || 
                                    !transaction::verify(&trans.transaction, trans.signer_pk.as_ref(), trans.signature.as_ref()) ||
                                    state_lock.get_value(s) < trans.transaction.Value {
            
                                    is_valid = false;
                                }                                
                            }
                            print!("RSL miner worker\n");
                        }

                        // add block
                        if is_valid {
                            print!("the block is valid in network worker\n");
                            let mut bstate : State;
                            {
                                let mut state_lock = self.state.lock().unwrap();

                                for trans in ablock.get_transactions() {
                                    let r = trans.transaction.Receiver;
                                    let s = trans.transaction.Sender;
                                    let nonce = state_lock.get_nonce(s);
                                    let val = state_lock.get_value(s);
                                    state_lock.add_account(s, nonce + 1, val - trans.transaction.Value);
                                    print!("net/worker trans val: {:?} acc val: {:?}  sender: {:?}\n", trans.transaction.Value, val, s);
                                    // receiver
                                    if (state_lock.contains_account(r)) {
                                        let nonce = state_lock.get_nonce(r);
                                        let val = state_lock.get_value(r);
                                        state_lock.add_account(r, nonce,  val + trans.transaction.Value);
                                    }
                                    else {
                                        state_lock.add_account(r, 0, trans.transaction.Value);
                                    }
                                }

                                bstate = state_lock.clone();
                            }
                            {
                                print!("im inserting a block in network worker\n");
                                let mut blockchain_lock = self.blockchain.lock().unwrap();
                                block_hashes.push(ablock.hash());
                                blockchain_lock.insert(&ablock, bstate);
                            }

                            // mempool copy for individual blocks
                            let mempool_copy;
                            {
                                let mut mempool_lock = self.mempool.lock().unwrap();
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
                        } 
                    }
                    
                    
                    if block_hashes.len() > 0 {
                        self.server.broadcast(Message:: NewBlockHashes(block_hashes));
                    }
                }
                Message::NewTransactionHashes(transaction_hashes) => {
                    let mut transactions : Vec<H256> = Vec::new();
                    {
                        let mempool_lock = self.mempool.lock().unwrap();
                        for trans_hash in transaction_hashes {
                            if !mempool_lock.contains_transaction(trans_hash) {
                                transactions.push(trans_hash);
                            }
                        }
                    }
                    if transactions.len() > 0 {
                        peer.write(Message::GetTransactions(transactions));
                    }
                }
                Message::GetTransactions(transaction_hashes) => {
                    let mut transactions : Vec<SignedTransaction> = Vec::new();
                    {
                        let mempool_lock = self.mempool.lock().unwrap();
                        for trans_hash in transaction_hashes {
                            if mempool_lock.contains_transaction(trans_hash) {
                                transactions.push(mempool_lock.get_transaction(trans_hash));
                            }
                        }
                    }
                    if transactions.len() > 0 {
                        peer.write(Message::Transactions(transactions));
                    }
                }
                Message::Transactions(transactions) => {
                    let mut hashes : Vec<H256> = Vec::new();
                    {
                        let mut mempool_lock = self.mempool.lock().unwrap();
                        for transaction in transactions {
                        
                            if !mempool_lock.contains_transaction(transaction.hash()) {
                                hashes.push(transaction.hash());
                                mempool_lock.add_transaction(transaction);
                            }
                        }
                    }
                    if hashes.len() > 0 {
                        self.server.broadcast(Message:: NewTransactionHashes(hashes));
                    }
                }
                _ => { 
                    
                }
            }
        }
    }
}

#[cfg(any(test,test_utilities))]
struct TestMsgSender {
    s: smol::channel::Sender<(Vec<u8>, peer::Handle)>
}
#[cfg(any(test,test_utilities))]
impl TestMsgSender {
    fn new() -> (TestMsgSender, smol::channel::Receiver<(Vec<u8>, peer::Handle)>) {
        let (s,r) = smol::channel::unbounded();
        (TestMsgSender {s}, r)
    }

    fn send(&self, msg: Message) -> PeerTestReceiver {
        let bytes = bincode::serialize(&msg).unwrap();
        let (handle, r) = peer::Handle::test_handle();
        smol::block_on(self.s.send((bytes, handle))).unwrap();
        r
    }
}
#[cfg(any(test,test_utilities))]
/// returns two structs used by tests, and an ordered vector of hashes of all blocks in the blockchain
fn generate_test_worker_and_start() -> (TestMsgSender, ServerTestReceiver, Vec<H256>) {
    let (server, server_receiver) = ServerHandle::new_for_test();
    let (test_msg_sender, msg_chan) = TestMsgSender::new();
    let blockchain = Blockchain::new();
    let blockchain = Arc::new(Mutex::new(blockchain));
    let mempool =  Mempool::new();
    let mempool = Arc::new(Mutex::new(mempool));
    let worker = Worker::new(1, msg_chan, &server, &blockchain, &mp);
    let mut hashes : Vec<H256>;
    {
        let mut blockchain_lock = worker.blockchain.lock().unwrap();
        hashes = blockchain_lock.all_blocks_in_longest_chain();
    }
    worker.start(); 

    (test_msg_sender, server_receiver, hashes)
}

// DO NOT CHANGE THIS COMMENT, IT IS FOR AUTOGRADER. BEFORE TEST

#[cfg(test)]
mod test {
    use ntest::timeout;
    use crate::types::block::generate_random_block;
    use crate::types::hash::Hashable;

    use super::super::message::Message;
    use super::generate_test_worker_and_start;

    #[test]
    #[timeout(60000)]
    fn reply_new_block_hashes() {
        let (test_msg_sender, _server_receiver, v) = generate_test_worker_and_start();
        let random_block = generate_random_block(v.last().unwrap());
        let mut peer_receiver = test_msg_sender.send(Message::NewBlockHashes(vec![random_block.hash()]));
        let reply = peer_receiver.recv();
        if let Message::GetBlocks(v) = reply {
            assert_eq!(v, vec![random_block.hash()]);
        } else {
            panic!();
        }
    }
    #[test]
    #[timeout(60000)]
    fn reply_get_blocks() {
        let (test_msg_sender, _server_receiver, v) = generate_test_worker_and_start();
        let h = v.last().unwrap().clone();
        let mut peer_receiver = test_msg_sender.send(Message::GetBlocks(vec![h.clone()]));
        let reply = peer_receiver.recv();
        if let Message::Blocks(v) = reply {
            assert_eq!(1, v.len());
            assert_eq!(h, v[0].hash())
        } else {
            panic!();
        }
    }
    #[test]
    #[timeout(60000)]
    fn reply_blocks() {
        let (test_msg_sender, server_receiver, v) = generate_test_worker_and_start();
        let random_block = generate_random_block(v.last().unwrap());
        let mut _peer_receiver = test_msg_sender.send(Message::Blocks(vec![random_block.clone()]));
        let reply = server_receiver.recv().unwrap();
        if let Message::NewBlockHashes(v) = reply {
            assert_eq!(v, vec![random_block.hash()]);
        } else {
            panic!();
        }
    }

    #[test]
    #[timeout(60000)]
    fn broadcast_new_blocks() {
        let (test_msg_sender, server_receiver, v) = generate_test_worker_and_start();
        let new_block = generate_random_block(v.last().unwrap());
        test_msg_sender.send(Message::Blocks(vec![new_block.clone()]));
        let reply = server_receiver.recv().unwrap();
        if let Message::NewBlockHashes(hashes) = reply {
            assert_eq!(hashes, vec![new_block.hash()]);
        } else {
            panic!();
        }
    }
}

// DO NOT CHANGE THIS COMMENT, IT IS FOR AUTOGRADER. AFTER TEST