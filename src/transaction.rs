use serde::{Serialize, Deserialize};



#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Transaction {
    pub sender: String,
    pub recipient: String,
    pub amount: u128,
    pub nonce: u64,
    pub chain_id: u32,
    pub public_key: Option<String>,
    pub signature: Option<String>,
}


// Mensaje canónico para firmar/verificar una transacción (solo campos económicos)
pub fn transaction_message(sender: &str, recipient: &str, amount: u128, nonce: u64, chain_id: u32) -> Vec<u8> {
    serde_json::to_vec(&serde_json::json!({
        "sender": sender,
        "recipient": recipient,
        "amount": amount,
        "nonce": nonce,
        "chain_id": chain_id
    })).unwrap()
}
