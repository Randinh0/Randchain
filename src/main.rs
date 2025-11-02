mod blockchain;
use crate::blockchain::{Blockchain, Block, Transaction};
use axum::{routing::{post, get}, Router, extract::State, Json};
use std::{net::SocketAddr, sync::{Arc, Mutex}};
use tokio::net::TcpListener;

type AppState = Arc<Mutex<Blockchain>>;

#[tokio::main]
async fn main() {
    let state: AppState = Arc::new(Mutex::new(Blockchain::new()));

    let app = Router::new()
        .route("/mine", post(mine))
        .route("/is_valid", get(is_valid))
        .route("/add_transaction", post(add_transaction))
        .with_state(state);

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    let listener = TcpListener::bind(addr).await.expect("No se pudo abrir el puerto");
    println!("Servidor escuchando en http://{}", addr);

    axum::serve(listener, app)
        .await
        .expect("Error al iniciar el servidor");
}

async fn mine(State(state): State<AppState>) -> Json<Block> {
    let mut blockchain = state.lock().expect("poisoned mutex");
    blockchain.mine_block();
    let new_block = blockchain.get_previous_block().clone();
    Json(new_block)
}


async fn is_valid(State(state): State<AppState>) -> Json<&'static str> {
    let blockchain = state.lock().expect("poisoned mutex");
    let is_valid = blockchain.is_chain_valid();
    let response = if is_valid {
        println!("Blockchain is valid");
        "Blockchain is valid"
    } else {
        println!("Blockchain is not valid");
        "Blockchain is not valid"
    };
    Json(response)
}

async fn add_transaction(
    State(state): State<AppState>,
    Json(tx): Json<Transaction>,
) -> Json<u32> {
    let mut blockchain = state.lock().expect("poisoned mutex");
    Json(blockchain.add_transaction(tx))
}