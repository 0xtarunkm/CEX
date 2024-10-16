

use std::sync::Arc;

use axum::{routing::{get, post}, Router};
use tokio::sync::Mutex;
use types::{AppState, User};

mod types;
mod routes;
use routes::*;

#[tokio::main]
async fn main() {
    let state = AppState {
        users: Arc::new(Mutex::new(vec![
            User {
                id: "1".to_string(),
                balances: [("GOOGLE".to_string(), 10.0), ("USD".to_string(), 50000.0)]
                    .iter().cloned().collect(),
            },
            User {
                id: "2".to_string(),
                balances: [("GOOGLE".to_string(), 100.0), ("USD".to_string(), 50000.0)]
                    .iter().cloned().collect()
            }
        ])),
        bids: Arc::new(Mutex::new(Vec::new())),
        asks: Arc::new(Mutex::new(Vec::new()))
    };

    let app = Router::new()
        .route("/order", post(hello))
        .route("/depth", get(get_depth))
        .with_state(state);

    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap()
}

async fn hello() {
    println!("hello");
}