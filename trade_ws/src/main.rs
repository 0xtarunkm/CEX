use std::net::SocketAddr;
use axum::{routing::get, Router};

mod user;
mod user_manager;

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/ws", get(hello));
        
    let addr = SocketAddr::from(([127, 0, 0, 1], 3001));
    println!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap()
}

async fn hello() {
    println!("hello");
}