use sha2::{Sha256, Digest};
use serde::{Serialize, Deserialize};
use std::time::{SystemTime, UNIX_EPOCH};

const COINBASE_SENDER: &str = "0";
const COINBASE_AMOUNT: f64 = 1.0;

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

            // Validación de coinbase: exactamente 1 por bloque (excepto génesis) y cantidad esperada
            let coinbase_count = current_block
                .transactions
                .iter()
                .filter(|t| t.sender == COINBASE_SENDER)
                .count();
            if coinbase_count != 1 {
                return false;
            }
            if let Some(cb) = current_block
                .transactions
                .iter()
                .find(|t| t.sender == COINBASE_SENDER)
            {
                if cb.amount != COINBASE_AMOUNT {
                    return false;
                }
            }

            i += 1;
        }
        true
    }
    pub fn mine_block(&mut self,node_adress: String){
        self.add_transaction(Transaction{sender: "0".to_string(), recipient: node_adress, amount: 1.0});
        let block =self.proof_of_work();
        self.create_block(block);
        self.transactions.clear();
        println!("Block mined: {} ", Blockchain::hash(self.get_previous_block()));
    }
    pub fn add_transaction(&mut self, transaction: Transaction)-> u32 {
        let index_next = { let previous = self.get_previous_block(); previous.index + 1 };
        // Validaciones de campos
        if transaction.sender.trim().is_empty() || transaction.recipient.trim().is_empty() {
            println!("Invalid transaction: sender/recipient vacíos");
            return index_next;
        }
        if !transaction.amount.is_finite() || transaction.amount <= 0.0 {
            println!("Invalid transaction: amount debe ser positivo y finito");
            return index_next;
        }
        if transaction.sender == COINBASE_SENDER {
            println!("Invalid transaction: coinbase solo se añade al minado");
            return index_next;
        }

        self.transactions.push(transaction);
        println!("Transaction added: {} ", index_next);
        index_next
    }

}

