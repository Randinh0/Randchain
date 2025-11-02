use crate::blockchain::{Blockchain, Block, Transaction};
use std::collections::HashSet;



pub struct Node {
    blockchain : Blockchain,
    node_adress: String,
    neighbor_nodes: HashSet<String>,
}

impl Node {
    pub async fn new(node_adress: String, neighbor_nodes: HashSet<String>, blockchain: Blockchain) -> Self {
        let mut node = Node {
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
    pub async fn mine_block(&mut self) -> Block {
        self.blockchain.mine_block();
        self.resolve_conflicts().await;
        let new_block = self.blockchain.get_previous_block().clone();
        new_block
    }
    pub async fn add_transaction(&mut self, transaction: Transaction) -> u32 {
        self.blockchain.add_transaction(transaction);
        self.resolve_conflicts().await;
        let previous_block = self.blockchain.get_previous_block().clone();
        previous_block.index + 1
    }

    pub fn get_blockchain(&self) -> Blockchain {
        self.blockchain.clone()
    }
}