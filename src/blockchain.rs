use sha2::{Sha256, Digest};
use serde::{Serialize, Deserialize};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Blockchain {
    pub chain: Vec<Block>,
    pub transactions: Vec<Transaction>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Block {
    pub index: u32,
    pub timestamp: u64,
    pub data: String,
    pub proof: u64,
    pub previous_hash: String,
    pub transactions: Vec<Transaction>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Transaction {
    pub sender: String,
    pub recipient: String,
    pub amount: f64,
}

pub struct Wallet {
    pub private_key: String,
    pub public_key: String,
    pub address: String,
}


impl Blockchain {
    pub fn new() -> Self {
        let mut blockchain = Blockchain { chain: Vec::new() , transactions: Vec::new()};
        let block = Block{
            index: 1,
            timestamp:  SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
            data: String::new(),
            proof: 0,
            previous_hash: "0".to_string(),
            transactions: Vec::new()
        };
        blockchain.create_block(block);
        blockchain
    }

    pub fn len(&self) -> usize {
        self.chain.len()
    }

    fn create_block(&mut self, block: Block) -> &Block {
        self.chain.push(block);
        self.chain.last().unwrap()
    }

    pub fn get_previous_block(&self) -> &Block {
        self.chain.last().expect("Blockchain should have at least one block")
    }

    fn proof_of_work(&self) -> Block {
        let mut block = Block {
            index: (self.chain.len() as u32) + 1,
            timestamp:  SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
            data: String::new(),
            proof: 0,
            previous_hash: Blockchain::hash(self.get_previous_block()),
            transactions: self.transactions.clone(),
        };

        loop {
            let hash_hex = Blockchain::hash_from_fields(
                block.index,
                block.timestamp,
                &block.data,
                block.proof,
                &block.previous_hash,
                block.transactions.clone(),
            );
            if hash_hex.starts_with("0000") {
                return block;
            }
            block.proof = block.proof.wrapping_add(1);
        }
    }

    pub fn hash(block: &Block) -> String {
        Blockchain::hash_from_fields(block.index, block.timestamp, &block.data, block.proof, &block.previous_hash, block.transactions.clone())
    }

    fn hash_from_fields(index: u32, timestamp: u64, data: &str, proof: u64, previous_hash: &str, transactions: Vec<Transaction>) -> String {
        let mut hasher = Sha256::new();
        let payload = format!("{}{}{}{}{}{}", index, timestamp, data, proof, previous_hash, serde_json::to_string(&transactions).unwrap());
        hasher.update(payload.as_bytes());
        let result = hasher.finalize();
        format!("{:x}", result)
    }

    pub fn is_chain_valid(&self) -> bool {
        if self.chain.len() <= 1 {
            return true;
        }
        let mut i = 1;
        while i < self.chain.len() {
            let previous_block = &self.chain[i - 1];
            let current_block = &self.chain[i];

            if current_block.previous_hash != Blockchain::hash(previous_block) {
                return false;
            }

            let current_hash = Blockchain::hash(current_block);
            if !current_hash.starts_with("0000") {
                return false;
            }

            i += 1;
        }
        true
    }
    pub fn mine_block(&mut self){
        let block =self.proof_of_work();
        self.create_block(block);
        println!("Block mined: {} ", Blockchain::hash(self.get_previous_block()));
    }
    pub fn add_transaction(&mut self, transaction: Transaction)-> u32 {
        self.transactions.push(transaction);
        let previous_block = self.get_previous_block();
        println!("Transaction added: {} ", previous_block.index + 1);
        previous_block.index + 1
    }

}

