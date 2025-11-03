mod blockchain;
mod node;
mod wallet;
mod transaction;

use crate::blockchain::{Blockchain, Block};
use crate::wallet::Wallet;
use crate::node::Node;
use crate::transaction::Transaction;

use axum::{routing::{post, get}, Router, extract::State, Json};
use std::net::SocketAddr;
use std::collections::HashSet;
use std::sync::Arc;
use tokio::{net::TcpListener, sync::Mutex};

type AppState = Arc<Mutex<Node>>;

#[tokio::main]
async fn main() {
    // Direcciones de los tres nodos
    let addr_a_str = "127.0.0.1:8000".to_string();
    let addr_b_str = "127.0.0.1:8001".to_string();
    let addr_c_str = "127.0.0.1:8002".to_string();

    // Conjuntos de vecinos (cada nodo conoce a los otros dos)
    let mut neighbors_a: HashSet<String> = HashSet::new();
    neighbors_a.insert(addr_b_str.clone());
    neighbors_a.insert(addr_c_str.clone());

    let mut neighbors_b: HashSet<String> = HashSet::new();
    neighbors_b.insert(addr_a_str.clone());
    neighbors_b.insert(addr_c_str.clone());

    let mut neighbors_c: HashSet<String> = HashSet::new();
    neighbors_c.insert(addr_a_str.clone());
    neighbors_c.insert(addr_b_str.clone());

    // Crear nodos
    let node_a = Node::new(addr_a_str.clone(), neighbors_a, Blockchain::new()).await;
    let node_b = Node::new(addr_b_str.clone(), neighbors_b, Blockchain::new()).await;
    let node_c = Node::new(addr_c_str.clone(), neighbors_c, Blockchain::new()).await;

    // Estados compartidos
    let state_a: AppState = Arc::new(Mutex::new(node_a));
    let state_b: AppState = Arc::new(Mutex::new(node_b));
    let state_c: AppState = Arc::new(Mutex::new(node_c));

    // Routers
    let app_a = Router::new()
        .route("/mine", post(mine))
        .route("/resolve_conflicts", get(resolve_conflicts))
        .route("/chain", get(chain))
        .route("/add_transaction", post(add_transaction))
        .route("/connect_node", post(connect_node))
        .with_state(state_a.clone());

    let app_b = Router::new()
        .route("/mine", post(mine))
        .route("/resolve_conflicts", get(resolve_conflicts))
        .route("/chain", get(chain))
        .route("/add_transaction", post(add_transaction))
        .route("/connect_node", post(connect_node))
        .with_state(state_b.clone());

    let app_c = Router::new()
        .route("/mine", post(mine))
        .route("/resolve_conflicts", get(resolve_conflicts))
        .route("/chain", get(chain))
        .route("/add_transaction", post(add_transaction))
        .route("/connect_node", post(connect_node))
        .with_state(state_c.clone());

    // Listeners
    let sock_a = SocketAddr::from(([127, 0, 0, 1], 8000));
    let sock_b = SocketAddr::from(([127, 0, 0, 1], 8001));
    let sock_c = SocketAddr::from(([127, 0, 0, 1], 8002));

    let listener_a = TcpListener::bind(sock_a).await.expect("No se pudo abrir el puerto 8000");
    let listener_b = TcpListener::bind(sock_b).await.expect("No se pudo abrir el puerto 8001");
    let listener_c = TcpListener::bind(sock_c).await.expect("No se pudo abrir el puerto 8002");

    println!("Servidor A escuchando en http://{}", sock_a);
    println!("Servidor B escuchando en http://{}", sock_b);
    println!("Servidor C escuchando en http://{}", sock_c);

    // Ejecutar los tres servidores en paralelo
    tokio::spawn(async move {
        if let Err(e) = axum::serve(listener_b, app_b).await { eprintln!("Servidor B detenido: {}", e); }
    });
    tokio::spawn(async move {
        if let Err(e) = axum::serve(listener_c, app_c).await { eprintln!("Servidor C detenido: {}", e); }
    });

    // Mantener el principal en primer plano
    axum::serve(listener_a, app_a)
        .await
        .expect("Error al iniciar el Servidor A");
}

async fn mine(State(state): State<AppState>) -> Json<Block> {
    let mut node = state.lock().await;
    let new_block = node.mine_block("juan".to_string()).await;
    Json(new_block)
}


async fn resolve_conflicts(State(state): State<AppState>) -> Json<&'static str> {
    let mut node = state.lock().await;
    let replaced = node.resolve_conflicts().await;
    let response = if replaced { "true" } else { "false" };
    Json(response)
}

async fn add_transaction(
    State(state): State<AppState>,
    Json(tx): Json<Transaction>,
) -> Json<u32> {
    let mut node = state.lock().await;
    Json(node.add_transaction(tx).await)
}

async fn chain(State(state): State<AppState>) -> Json<Blockchain> {
    let node = state.lock().await;
    Json(node.get_blockchain())
}

#[derive(serde::Deserialize)]
struct ConnectReq { address: String }

async fn connect_node(State(state): State<AppState>, Json(req): Json<ConnectReq>) -> Json<bool> {
    let mut node = state.lock().await;
    Json(node.connect_node(req.address).await)
}

async fn create_wallet() -> Json<Wallet> {
    let wallet = Wallet::new();
    Json(wallet)
}
