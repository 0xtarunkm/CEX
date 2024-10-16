use std::collections::HashMap;
use axum::{extract::State, Json};
use crate::types::AppState;

pub async fn get_depth(State(state): State<AppState>) -> Json<HashMap<String, HashMap<String, f64>>> {
    let bids = state.bids.lock().await;  // Lock asynchronously
    let asks = state.asks.lock().await;  // Lock asynchronously
    let mut depth = HashMap::new();

    for bid in bids.iter() {
        depth.entry(bid.price.to_string())
            .or_insert_with(HashMap::new)
            .insert("bid".to_string(), bid.quantity);
    }

    for ask in asks.iter() {
        depth.entry(ask.price.to_string())
            .or_insert_with(HashMap::new)
            .insert("ask".to_string(), ask.quantity);
    }

    Json(depth)
}
