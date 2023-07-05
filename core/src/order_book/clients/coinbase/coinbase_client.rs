use std::str::FromStr;

use serde_json::Value;
use tokio::time::Instant;
use tokio_tungstenite::tungstenite::Message;

use crate::order_book::clients::client::WebSocketClient;

use super::{coinbase_adapter::CoinbaseAdapter, data_types::Snapshot, data_types::Update};

pub struct CoinbaseReceiveClient {
    adapter: CoinbaseAdapter,
    client: WebSocketClient,
}

impl CoinbaseReceiveClient {
    pub async fn new() -> Self {
        return CoinbaseReceiveClient {
            adapter: CoinbaseAdapter::new().await,
            client: WebSocketClient::new("wss://ws-feed.exchange.coinbase.com".to_string()).await
        }
    }

    pub async fn init(&mut self) {
        let sub_message: String = "{\"type\":\"subscribe\",\"product_ids\":[\"ETH-USD\"],\"channels\":[\"level2\"]}".to_string();
        self.client.send(Message::Text(sub_message)).await;
        self.receive().await;
    }

    async fn receive(&mut self) {
        let mut count: usize = 0;
        let mut total: usize = 0;
        while let Some(msg) = self.client.receive().await {
            match msg {
                Ok(msg) => {
                    let v: Value = serde_json::from_str(&msg.to_string()).expect("Parsing error");
                    match v["type"].as_str() {
                        Some("subscriptions") => println!("Received subscription confirmation"),
                        Some("snapshot") => self.handle_snapshot(v),
                        Some("l2update") => {
                            let start = Instant::now();
                            self.handle_update(v);
                            let duration = start.elapsed();
                            let now = Instant::now();
                            count = count + 1;
                            total = total + duration.as_micros() as usize;
                            let avg: f64 = (total as f64) / (count as f64);
                            println!("Update parsed in {:?}", duration);
                            println!("Average parse time: {:?} microseconds", avg);
                        },
                        Some(other) => println!("Unknown message type {:?}: {:?}", other, msg),
                        None => println!("No message type found: {:?}", msg),
                    }
                },
                Err(err) => println!("{:?}", err)
            }
        }
    }

    fn handle_snapshot(&mut self, snapshot: Value) {
        let snapshot: Snapshot = serde_json::from_str(&snapshot.to_string()).unwrap();
        println!("Snapshot: {:?} bids, {:?} asks", snapshot.bids.len(), snapshot.asks.len());
    }

    fn handle_update(&mut self, update: Value) {
        let update: Update = serde_json::from_str(&update.to_string()).unwrap();
        let now = chrono::Utc::now();
        let then = chrono::DateTime::from_str(&update.time).unwrap();
        let elapsed = now - then;
        println!("Sent at {:?} to deserialized at {:?} - total time: {:?}", update.time, now, elapsed);

        //println!("Update at {:?}: {:?} changes", update.time, update.changes.len())
        
    }
}

pub struct CoinbaseSendClient {
    client: WebSocketClient,
}

impl CoinbaseSendClient {
    pub async fn new() -> Self {
        return CoinbaseSendClient {
            client: WebSocketClient::new("wss://ws-feed.exchange.coinbase.com".to_string()).await
        }
    }
}