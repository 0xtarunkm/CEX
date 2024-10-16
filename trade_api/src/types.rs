use std::{collections::HashMap, sync::Arc};

use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;

pub const TICKER: &str = "GOOGLE";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: String,
    pub balances: HashMap<String, f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Order {
    pub user_id: String,
    pub price: f64,
    pub quantity: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderRequest {
    pub side: String,
    pub price: f64,
    pub quantity: f64,
    pub user_id: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct OrderResponse {
    pub message: String,
    pub filled_quantity: f64,
}

#[derive(Clone)]
pub struct AppState {
    pub users: Arc<Mutex<Vec<User>>>,
    pub bids: Arc<Mutex<Vec<Order>>>,
    pub asks: Arc<Mutex<Vec<Order>>>,
}
