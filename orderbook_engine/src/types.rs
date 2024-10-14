use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Side {
    Buy,
    Sell,
}

pub enum OrderbookResult {
    Trades(Vec<Trade>),
    CancelledOrder(Order),
    OrderNotFound,
    Depth((Vec<(u64, u64)>, Vec<(u64, u64)>)),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Order {
    pub id: u64,
    pub user_id: String,
    pub side: Side,
    pub price: u64,
    pub quantity: u64,
    pub filled: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Trade {
    pub id: u64,
    pub price: u64,
    pub quantity: u64,
    pub maker_order_id: u64,
    pub taker_order_id: u64,
    pub maker_user_id: String,
    pub taker_user_id: String,
}

#[derive(Debug, Clone)]
pub struct UserBalance {
    pub available: HashMap<String, u64>,
    pub locked: HashMap<String, u64>,
}

impl UserBalance {
    pub fn new() -> Self {
        UserBalance {
            available: HashMap::new(),
            locked: HashMap::new(),
        }
    }
}
