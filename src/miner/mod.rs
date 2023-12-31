pub mod worker;

use log::info;
use crossbeam::channel::{unbounded, Receiver, Sender, TryRecvError};
use std::time;
use std::thread;
use rand::Rng;
use std::time::{SystemTime, UNIX_EPOCH};
use std::sync::{Arc, Mutex};
use crate::types::block::Block;
use crate::blockchain::Blockchain;
use crate::types::transaction::SignedTransaction;
use crate::types::transaction;
use crate::types::transaction::{Transaction, verify};
use crate::types::hash::{H256, Hashable};
use crate::types::mempool::Mempool;
use crate::types::state::State;

enum ControlSignal {
    Start(u64), // the number controls the lambda of interval between block generation
    Update, // update the block in mining, it may due to new blockchain tip or new transaction
    Exit,
}

enum OperatingState {
    Paused,
    Run(u64),
    ShutDown,
}

pub struct Context {
    /// Channel for receiving control signal
    control_chan: Receiver<ControlSignal>,
    operating_state: OperatingState,
    finished_block_chan: Sender<Block>,
    blockchain : Arc<Mutex<Blockchain>>,
    mempool: Arc<Mutex<Mempool>>,
    state: Arc<Mutex<State>>,
}

#[derive(Clone)]
pub struct Handle {
    /// Channel for sending signal to the miner thread
    control_chan: Sender<ControlSignal>,
}

pub fn new(bc: &Arc<Mutex<Blockchain>>, mp: &Arc<Mutex<Mempool>>, st: &Arc<Mutex<State>>) -> (Context, Handle, Receiver<Block>) {
    let (signal_chan_sender, signal_chan_receiver) = unbounded();
    let (finished_block_sender, finished_block_receiver) = unbounded();

    let ctx = Context {
        control_chan: signal_chan_receiver,
        operating_state: OperatingState::Paused,
        finished_block_chan: finished_block_sender,
        blockchain : Arc::clone(bc),
        mempool: Arc::clone(mp),
        state: Arc::clone(st),
    };

    let handle = Handle {
        control_chan: signal_chan_sender,
    };

    (ctx, handle, finished_block_receiver)
}

#[cfg(any(test,test_utilities))]
fn test_new() -> (Context, Handle, Receiver<Block>) {
    let blockchain = Blockchain::new();
    let blockchain = Arc::new(Mutex::new(blockchain));
    let mempool = Mempool::new();
    let mempool = Arc::new(Mutex::new(mempool));
    return new(&blockchain, &mempool);
}

impl Handle {
    pub fn exit(&self) {
        self.control_chan.send(ControlSignal::Exit).unwrap();
    }

    pub fn start(&self, lambda: u64) {
        self.control_chan
            .send(ControlSignal::Start(lambda))
            .unwrap();
    }

    pub fn update(&self) {
        self.control_chan.send(ControlSignal::Update).unwrap();
    }
}

impl Context {
    pub fn start(mut self) {
        thread::Builder::new()
            .name("miner".to_string())
            .spawn(move || {
                self.miner_loop();
            })
            .unwrap();
        info!("Miner initialized into paused mode");
    }

    fn miner_loop(&mut self) {
        // main mining loop
        loop {
            // check and react to control signals
            match self.operating_state {
                OperatingState::Paused => {
                    let signal = self.control_chan.recv().unwrap();
                    match signal {
                        ControlSignal::Exit => {
                            info!("Miner shutting down");
                            self.operating_state = OperatingState::ShutDown;
                        }
                        ControlSignal::Start(i) => {
                            info!("Miner starting in continuous mode with lambda {}", i);
                            self.operating_state = OperatingState::Run(i);
                        }
                        ControlSignal::Update => {
                            // in paused state, don't need to update
                        }
                    };
                    continue;
                }
                OperatingState::ShutDown => {
                    return;
                }
                _ => match self.control_chan.try_recv() {
                    Ok(signal) => {
                        match signal {
                            ControlSignal::Exit => {
                                info!("Miner shutting down");
                                self.operating_state = OperatingState::ShutDown;
                            }
                            ControlSignal::Start(i) => {
                                info!("Miner starting in continuous mode with lambda {}", i);
                                self.operating_state = OperatingState::Run(i);
                            }
                            ControlSignal::Update => {
                                unimplemented!()
                            }
                        };
                    }
                    Err(TryRecvError::Empty) => {}
                    Err(TryRecvError::Disconnected) => panic!("Miner control channel detached"),
                },
            }
            if let OperatingState::ShutDown = self.operating_state {
                return;
            }

            // actual mining, create a block
            let mut parent;
            let mut pblock;

            // generating a new block
            {
                // get current tip 
                let blockchain_lock = self.blockchain.lock().unwrap();
                parent = blockchain_lock.tip();
                pblock = blockchain_lock.get_block(parent);
            }
            let mut rng = rand::thread_rng();
            let nonce = rng.gen_range(0..u32::MAX);
            
            let now = SystemTime::now();
            let timestamp = now.duration_since(UNIX_EPOCH).unwrap().as_millis();

            // add transactions from mempoool here
            let transactions : Vec<SignedTransaction>;
            let mintrans : usize = 5;
            let maxtrans : usize = 30;
            {
                let mempool_lock = self.mempool.lock().unwrap();
                transactions = mempool_lock.get_max(maxtrans, mintrans);
            }
            
            let mut ablock = Block :: new(parent, nonce, pblock.get_difficulty(), timestamp, pblock.get_to_genesis() + 1, transactions);
            
            // if block mining finished, send to channel
            let mut is_valid = true;
            let mut bstate: State;
            if ablock.hash() <= ablock.get_difficulty() && ablock.get_transactions().len() > 0 {
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
                        print!("miner/mod trans val: {:?} acc val: {:?}  sender: {:?}\n", trans.transaction.Value, val, s);

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
                    print!("valid block in mod miner");
                    {
                        let mut state_lock = self.state.lock().unwrap();
                        *state_lock =  bstate.clone();
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
                    {
                        let mut blockchain_lock = self.blockchain.lock().unwrap();
                        blockchain_lock.insert(&ablock, bstate);
                        print!("inserted block in mod miner\n");
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

                    self.finished_block_chan.send(ablock.clone()).expect("Send finished block error");
                } 
            }

           
            if let OperatingState::Run(i) = self.operating_state {
                if i != 0 {
                    let interval = time::Duration::from_micros(i as u64);
                    thread::sleep(interval);
                }
            }
        }
    }
}

// DO NOT CHANGE THIS COMMENT, IT IS FOR AUTOGRADER. BEFORE TEST

#[cfg(test)]
mod test {
    use ntest::timeout;
    use crate::types::hash::Hashable;

    #[test]
    #[timeout(60000)]
    fn miner_three_block() {
        let (miner_ctx, miner_handle, finished_block_chan) = super::test_new();
        miner_ctx.start();
        miner_handle.start(0);
        let mut block_prev = finished_block_chan.recv().unwrap();
        for _ in 0..2 {
            let block_next = finished_block_chan.recv().unwrap();
            assert_eq!(block_prev.hash(), block_next.get_parent());
            block_prev = block_next;
        }
    }

    #[test]
    #[timeout(60000)]
    fn miner_ten_block() {
        let (miner_ctx, miner_handle, finished_block_chan) = super::test_new();
        miner_ctx.start();
        miner_handle.start(0);
        let mut block_prev = finished_block_chan.recv().unwrap();
        for _ in 0..9 {
            let block_next = finished_block_chan.recv().unwrap();
            assert_eq!(block_prev.hash(), block_next.get_parent());
            block_prev = block_next;
        }
    }
}


// DO NOT CHANGE THIS COMMENT, IT IS FOR AUTOGRADER. AFTER TEST