use serde_json::Value;
use std::collections::HashMap;
use std::error::Error;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::orderbook::{Orderbook, OrderbookMessage};
use crate::types::{Order, Side, UserBalance};

pub struct Engine {
    orderbooks: HashMap<String, Arc<Mutex<Orderbook>>>,
    balances: Arc<Mutex<HashMap<String, UserBalance>>>,
}

impl Engine {
    pub fn new() -> Self {
        Engine {
            orderbooks: HashMap::new(),
            balances: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn add_market(&mut self, market: String) {
        let orderbook = Arc::new(Mutex::new(Orderbook::new(market.clone())));
        self.orderbooks.insert(market, orderbook);
    }

    pub async fn process(&self, message: Value) -> Result<(), Box<dyn std::error::Error>> {
        match message["type"].as_str() {
            Some("PLACE_ORDER") => {
                let market = message["market"].as_str().unwrap().to_string();
                let base_asset = message["base_asset"].as_str().unwrap().to_string();
                let order: Order = serde_json::from_value(message["order"].clone())?;
                self.place_order(market, order, &base_asset).await?;
            }
            Some("CANCEL_ORDER") => {
                let market = message["market"].as_str().unwrap().to_string();
                let order_id = message["orderId"].as_u64().unwrap();
                self.cancel_order(market, order_id).await?;
            }
            Some("GET_DEPTH") => {
                let market = message["market"].as_str().unwrap().to_string();
                self.get_depth(market).await?;
            }
            Some("ADD_BALANCE") => {
                let user_id = message["userId"].as_str().unwrap().to_string();
                let asset = message["asset"].as_str().unwrap().to_string();
                let amount = message["amount"].as_u64().unwrap();
                self.add_balance(user_id, asset, amount).await?;
            }
            _ => println!("Unknown message type"),
        }
        Ok(())
    }

    async fn place_order(
        &self,
        market: String,
        order: Order,
        base_asset: &str
    ) -> Result<(), Box<dyn Error>> {
        if let Some(orderbook) = self.orderbooks.get(&market) {
            if self.lock_funds(&order, base_asset).await? {
                let mut orderbook = orderbook.lock().await;
                orderbook.process(OrderbookMessage::PlaceOrder(order)).await;
            } else {
                println!("Insufficient funds to place order");
            }
        } else {
            println!("Orderbook not found for {market}");
        }
        Ok(())
    }

    async fn cancel_order(
        &self,
        market: String,
        order_id: u64,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(orderbook) = self.orderbooks.get(&market) {
            let mut orderbook = orderbook.lock().await;
            orderbook
                .process(OrderbookMessage::CancelOrder(order_id))
                .await;
        } else {
            println!("Orderbook not found for {market}");
        }
        Ok(())
    }

    async fn get_depth(&self, market: String) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(orderbook) = self.orderbooks.get(&market) {
            let orderbook = orderbook.lock().await;
            let depth = orderbook.get_depth();
            println!("Market depth for {}: {:?}", market, depth);
        } else {
            println!("Orderbook not found for {market}");
        }
        Ok(())
    }

    async fn add_balance(
        &self,
        user_id: String,
        asset: String,
        amount: u64,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut balances = self.balances.lock().await;
        let user_balance = balances.entry(user_id).or_insert_with(UserBalance::new);
        *user_balance.available.entry(asset).or_insert(0) += amount;
        Ok(())
    }

    async fn lock_funds(&self, order: &Order, base_asset: &str) -> Result<bool, Box<dyn std::error::Error>> {
        let mut balances = self.balances.lock().await;
        let user_balance = balances
            .entry(order.user_id.clone())
            .or_insert_with(UserBalance::new);

        let asset = if order.side == Side::Buy {
            "USDT"
        } else {
            base_asset
        };

        let required_amount = if order.side == Side::Buy {
            order.price * order.quantity
        } else {
            order.quantity
        };

        if let Some(available) = user_balance.available.get_mut(asset) {
            if *available >= required_amount {
                *available -= required_amount;
                *user_balance.locked.entry(asset.to_string()).or_insert(0) += required_amount;
                Ok(true)
            } else {
                Ok(false)
            }
        } else {
            Ok(false)
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[tokio::test]
    async fn test_add_market() {
        let mut engine = Engine::new();
        engine.add_market("BTC_USDT".to_string());
        assert!(engine.orderbooks.contains_key("BTC_USDT"));
    }

    #[tokio::test]
    async fn test_process_place_order() {
        let mut engine = Engine::new();
        engine.add_market("BTC_USDT".to_string());

        let message = json!({
            "type": "PLACE_ORDER",
            "market": "BTC_USDT",
            "base_asset": "BTC",
            "order": {
                "id": 1,
                "user_id": "user1",
                "side": "Buy",
                "price": 50000,
                "quantity": 1,
                "filled": 0
            }
        });

        let result = engine.process(message).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_process_cancel_order() {
        let mut engine = Engine::new();
        engine.add_market("BTC_USDT".to_string());

        let message = json!({
            "type": "CANCEL_ORDER",
            "market": "BTC_USDT",
            "orderId": 1
        });

        let result = engine.process(message).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_process_get_depth() {
        let mut engine = Engine::new();
        engine.add_market("BTC_USDT".to_string());

        let message = json!({
            "type": "GET_DEPTH",
            "market": "BTC_USDT"
        });

        let result = engine.process(message).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_process_add_balance() {
        let engine = Engine::new();

        let message = json!({
            "type": "ADD_BALANCE",
            "userId": "user1",
            "asset": "BTC",
            "amount": 1
        });

        let result = engine.process(message).await;
        assert!(result.is_ok());

        let balances = engine.balances.lock().await;
        let user_balance = balances.get("user1").unwrap();
        assert_eq!(*user_balance.available.get("BTC").unwrap(), 1);
    }

    #[tokio::test]
    async fn test_process_unknown_message() {
        let engine = Engine::new();

        let message = json!({
            "type": "UNKNOWN_TYPE"
        });

        let result = engine.process(message).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_place_order_insufficient_funds() {
        let mut engine = Engine::new();
        engine.add_market("BTC_USDT".to_string());

        let add_balance_message = json!({
            "type": "ADD_BALANCE",
            "userId": "user1",
            "asset": "USDT",
            "amount": 10000
        });
        engine.process(add_balance_message).await.unwrap();

        let place_order_message = json!({
            "type": "PLACE_ORDER",
            "market": "BTC_USDT",
            "base_asset": "BTC",
            "order": {
                "id": 1,
                "user_id": "user1",
                "side": "Buy",
                "price": 50000,
                "quantity": 1,
                "filled": 0
            }
        });

        let result = engine.process(place_order_message).await;
        assert!(result.is_ok());

        let orderbook = engine.orderbooks.get("BTC_USDT").unwrap().lock().await;
        assert_eq!(orderbook.get_depth().0.len(), 0);
    }
}