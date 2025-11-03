use std::collections::HashSet;
use k256::ecdsa::{VerifyingKey, Signature, signature::DigestVerifier};
use sha2::{Digest, Sha256};
use hex;

use crate::blockchain::{Blockchain, Block};
use crate::transaction::{Transaction, transaction_message};


pub struct Node {
    blockchain : Blockchain,
    node_adress: String,
    neighbor_nodes: HashSet<String>,
}

impl Node {
    pub async fn new(node_adress: String, neighbor_nodes: HashSet<String>, blockchain: Blockchain) -> Self {
        let node = Node {
            blockchain: blockchain,
            node_adress: node_adress,
            neighbor_nodes: neighbor_nodes,
        };
        // No forzar sincronización en el arranque; los otros nodos pueden no estar aún levantados
        node
    }
    pub async fn resolve_conflicts(&mut self) -> bool {
        let mut longest_blockchain = self.blockchain.clone();
        for node in &self.neighbor_nodes {
            let url = format!("http://{}/chain", node);
            if let Ok(response) = reqwest::get(url).await {
                if response.status().is_success() {
                    if let Ok(blockchain) = response.json::<Blockchain>().await {
                        if blockchain.len() > longest_blockchain.len() && blockchain.is_chain_valid() {
                            longest_blockchain = blockchain;
                        }
                    }
                }
            }
        }
        if longest_blockchain.len() > self.blockchain.len() {
            self.blockchain = longest_blockchain;
            true
        } else {
            false
        }
    }
    pub async fn connect_node(&mut self, addr: String) -> bool{
        let url = format!("http://{}/chain", addr);
        if let Ok(response) = reqwest::get(url).await {
            if response.status().is_success() {
                self.neighbor_nodes.insert(addr);
                return true;
            }
        }
        false
    }
    pub async fn mine_block(&mut self,node_adress: String) -> Block {
        self.blockchain.mine_block(node_adress);
        self.resolve_conflicts().await;
        let new_block = self.blockchain.get_previous_block().clone();
        new_block
    }
    pub async fn add_transaction(&mut self, transaction: Transaction) -> u32 {
        // Verificación de firma para transacciones no-coinbase
        let index_next = {
            let previous = self.blockchain.get_previous_block().clone();
            previous.index + 1
        };

        if transaction.sender != "0" {
            let Some(sig_hex) = transaction.signature.as_ref() else {
                println!("Invalid transaction: falta firma");
                return index_next;
            };
            let Some(pk_hex) = transaction.public_key.as_ref() else {
                println!("Invalid transaction: falta clave pública");
                return index_next;
            };

            if !verify_transaction_signature(&transaction, pk_hex, sig_hex) {
                println!("Invalid transaction: firma inválida");
                return index_next;
            }
        }

        self.blockchain.add_transaction(transaction);
        self.resolve_conflicts().await;
        let previous_block = self.blockchain.get_previous_block().clone();
        previous_block.index + 1
    }

    pub fn get_blockchain(&self) -> Blockchain {
        self.blockchain.clone()
    }
}

fn verify_transaction_signature(tx: &Transaction, pubkey_hex: &str, sig_hex: &str) -> bool {
    // Derivar dirección desde pubkey comprimida (mismo método que Wallet)
    let pk_bytes = match hex::decode(pubkey_hex) { Ok(b) => b, Err(_) => return false };
    let vk = match VerifyingKey::from_sec1_bytes(&pk_bytes) { Ok(v) => v, Err(_) => return false };

    let derived_addr = {
        let digest = Sha256::digest(&pk_bytes);
        hex::encode(&digest[..20])
    };
    if derived_addr != tx.sender { return false; }

    let msg = transaction_message(&tx.sender, &tx.recipient, tx.amount, tx.nonce, tx.chain_id);
    let sig_bytes = match hex::decode(sig_hex) { Ok(b) => b, Err(_) => return false };
    let sig = match Signature::from_der(&sig_bytes) { Ok(s) => s, Err(_) => return false };
    let mut hasher = Sha256::new();
    hasher.update(&msg);
    vk.verify_digest(hasher, &sig).is_ok()
}