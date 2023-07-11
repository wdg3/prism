use std::{time::Duration, sync::Arc};

use tokio::{time::Instant, sync::RwLock};

use crate::order_book::{clients::client::WebSocketClient, order_book::MultiBook};

use super::{coinbase_adapter::CoinbaseAdapter, data_types::{Snapshot, Message}, data_types::Update};

pub struct CoinbaseReceiveClient {
    adapter: CoinbaseAdapter,
    client: WebSocketClient,
}

impl<'a> CoinbaseReceiveClient {
    pub async fn new(book: Arc<RwLock<MultiBook<3, 6>>>) -> CoinbaseReceiveClient {
        return CoinbaseReceiveClient {
            adapter: CoinbaseAdapter::new(book).await,
            client: WebSocketClient::new("wss://ws-feed.exchange.coinbase.com".to_string()).await
        }
    }

    pub async fn init(&mut self) {
        let sub_message: String = "{\"type\":\"subscribe\",\"product_ids\":[\"BTC-USD\"],\"channels\":[\"level2\"]}".to_string();
        self.client.send(tokio_tungstenite::tungstenite::protocol::Message::Text(sub_message)).await;
        self.receive().await;
    }

    async fn receive(&mut self) {
        let mut count: usize = 0;
        let mut total: usize = 0;
        while let Some(msg) = self.client.receive().await {
            let start = Instant::now();
            match msg {
                Ok(msg) => {
                    let (message, _) = serde_json_core::from_str::<Message>(&msg.to_text().unwrap()).expect("Parsing error");
                    match message.msg_type {
                        "subscriptions" => {
                            ()
                        },
                        "snapshot" => {
                            self.handle_snapshot(&msg.to_text().unwrap()).await
                        },
                        "l2update" => {
                            self.handle_update(&msg.to_text().unwrap(), start).await;
                            let duration = start.elapsed();
                            count = count + 1;
                            total = total + duration.as_nanos() as usize;
                            let avg: f64 = (total as f64) / (count as f64);
                            //println!("Coinbase: message handled in {:?}", duration);
                            //println!("Coinbase: average message handle time for {:?} messages: {:?}", count, Duration::new(0, avg as u32));
                        },
                        other => println!("Unknown message type {:?}: {:?}", other, msg),
                    }
                },
                Err(err) => println!("Coinbase: {:?}", err)
            }
        }
    }

    async fn handle_snapshot(&mut self, snapshot: &str) {
        let result = serde_json_core::from_str::<Snapshot>(snapshot);
        match result {
            Ok((snapshot, _)) => {
                self.adapter.init_order_book(snapshot).await
            },
            Err(err) => println!("Error parsing: {:?} for {:?}", err, snapshot),
        }        
    }

    async fn handle_update(&mut self, update: &str, start: Instant) -> Duration {
        let result = serde_json_core::from_str::<Update>(update);
        match result {
            Ok((update, _)) => {
                self.adapter.update(update).await
            }
            Err(err) => println!("Error parsing: {:?} for {:?}", err, update),
        }
        start.elapsed()
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