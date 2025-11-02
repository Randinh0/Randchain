use crate::blockchain::Blockchain;




pub struct Node {
    blockchain : Blockchain,
    node_adress: String,
    neighbor_nodes: HashSet<String>,
}

impl Node {
    pub fn new(node_adress: String) -> Self {


    }
    pub fn resolve_conflicts(&self) -> bool {
        let mut longest_chain = self.blockchain.clone();
        for node in self.neighbor_nodes {
            let response = reqwest::get(format!("http://{}/chain", node)).await.unwrap();
            let chain: Blockchain = response.json().await.unwrap();
            if chain.len() > longest_chain.len() {
                longest_chain = chain;
            }
        }
        if longest_chain.len() > self.blockchain.len() {
            self.blockchain = longest_chain;

        }
    }
}