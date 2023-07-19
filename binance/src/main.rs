mod client;
use client::WebSocketClient;
use chrono::Utc;
use std::time::Duration;
use serde_json::Value;
use tokio_tungstenite::tungstenite::protocol::Message;

#[tokio::main]
async fn main() {
    let mut binance_client = WebSocketClient::new("wss://stream.binance.com/ws/btcusdt@aggTrade".to_string()).await;
    let binance_sub: String = format!("{{\"method\": \"SUBSCRIBE\",\"params\": [\"btcusdt@aggTrade\"],\"id\": 1}}").to_string();
    let mut count = 0;
    let mut total = 0;
    binance_client.send(Message::Text(binance_sub)).await;
    while let Some(msg) = binance_client.receive().await {
        let value = serde_json::from_str::<Value>(&msg.unwrap().to_text().unwrap()).unwrap();
        let event_time = value.get("E");
        let trade_time = value.get("T");
        match (event_time, trade_time) {
            (Some(e), Some(t)) => {
                let now = Utc::now();
                let dur = now.timestamp_millis() - e.as_i64().unwrap();
                println!("Sent to handled time: {:?}", dur);
                count = count + 1;
                total = total + dur;
                let avg: f64 = (total as f64) / (count as f64);
                println!("Avg. sent to handled time: {:?}", Duration::new(0, (avg * 1000000.0) as u32));
            },
            _ => ()
        }
    }
}
