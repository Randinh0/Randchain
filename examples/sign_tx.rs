use std::env;

use reqwest::Client;
// no extra imports

use blockchain::blockchain::Blockchain; // ensures crate linkage
use blockchain::transaction::Transaction;
use blockchain::wallet::Wallet;

#[tokio::main]
async fn main() {
    // Args: [node_url] [amount] [nonce] [chain_id]
    let args: Vec<String> = env::args().collect();
    let node_url = args.get(1).cloned().unwrap_or_else(|| "http://127.0.0.1:8000".to_string());
    let amount: u128 = args.get(2).and_then(|s| s.parse().ok()).unwrap_or(10);
    let nonce: u64 = args.get(3).and_then(|s| s.parse().ok()).unwrap_or(1);
    let chain_id: u32 = args.get(4).and_then(|s| s.parse().ok()).unwrap_or(1);

    // Generar wallets de ejemplo (emisor y destinatario)
    let sender = Wallet::new();
    let recipient = Wallet::new();

    println!("Sender address: {}", sender.address);
    println!("Recipient address: {}", recipient.address);
    println!("Sender priv (hex): {}", sender.private_key);
    println!("Sender pub  (hex): {}", sender.public_key);

    // Firmar transacción
    let tx: Transaction = sender.create_signed_transaction(
        recipient.address.clone(),
        amount,
        nonce,
        chain_id,
    );

    // Enviar al nodo
    let url = format!("{}/add_transaction", node_url);
    let client = Client::new();
    let resp = client.post(&url).json(&tx).send().await;

    match resp {
        Ok(r) => {
            let status = r.status();
            let text = r.text().await.unwrap_or_default();
            println!("Node response: {} - {}", status, text);
        }
        Err(e) => {
            eprintln!("Request error: {}", e);
        }
    }
}


