use redis::Client;
use serde_json::Value;
use std::error::Error;

mod engine;
mod orderbook;
mod types;

use engine::Engine;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let mut engine = Engine::new();
    engine.add_market("BTC_USDT".to_string());
    engine.add_market("ETH_USDT".to_string());

    let client = Client::open("redis://127.0.0.1/")?;
    let mut con = client.get_async_connection().await?;

    loop {
        let response: Option<String> = redis::cmd("RPOP")
            .arg("messages")
            .query_async(&mut con)
            .await?;

        if let Some(message) = response {
            let parsed: Value = serde_json::from_str(&message)?;
            engine.process(parsed).await?;
        }
    }
}
