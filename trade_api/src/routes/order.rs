use axum::{extract::State, Json};

use crate::types::{AppState, Order, OrderRequest, OrderResponse, User, TICKER};

pub async fn place_order(
    State(state): State<AppState>,
    Json(order_request): Json<OrderRequest>
) -> Json<OrderResponse> {
    let mut bids = state.bids.lock().await;
    let mut asks = state.asks.lock().await;
    let mut users = state.users.lock().await;

    
    todo!()
}

async fn fill_orders(
    side: &str,
    price: f64,
    quantity: f64,
    user_id: &str,
    bids: &mut Vec<Order>,
    asks: &mut Vec<Order>,
    users: &mut Vec<User>,
) -> f64 {
    let mut remaining_quantity = quantity;

    if side == "bid" {
        while let Some(ask) = asks.last_mut() {
            if ask.price > price {
                break;
            }
            if ask.quantity > remaining_quantity {
                ask.quantity -= remaining_quantity;
            }
        }
    }

    todo!()
}

fn flip_balance(
    user_id1: &str,
    user_id2: &str,
    quantity: f64,
    price: f64,
    users: &mut Vec<User>
) {
    let user1 = users.iter_mut().find(|u| u.id == user_id1).unwrap();
    let user2 = users.iter_mut().find(|u| u.id == user_id2).unwrap();

    *user1.balances.entry(TICKER.to_string()).or_insert(0.0) -= quantity;
    *user2.balances.entry(TICKER.to_string()).or_insert(0.0) += quantity;
    *user1.balances.entry("USD".to_string()).or_insert(0.0) += quantity * price;
    *user2.balances.entry("USD".to_string()).or_insert(0.0) -= quantity * price;
}