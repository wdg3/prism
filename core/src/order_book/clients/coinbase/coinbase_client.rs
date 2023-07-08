use tokio::time::Instant;

use crate::order_book::clients::client::WebSocketClient;

use super::{coinbase_adapter::CoinbaseAdapter, data_types::{Snapshot, Message}, data_types::Update};

pub struct CoinbaseReceiveClient {
    adapter: CoinbaseAdapter,
    client: WebSocketClient,
}

impl<'a> CoinbaseReceiveClient {
    pub async fn new() -> CoinbaseReceiveClient {
        return CoinbaseReceiveClient {
            adapter: CoinbaseAdapter::new().await,
            client: WebSocketClient::new("wss://ws-feed.exchange.coinbase.com".to_string()).await
        }
    }

    pub async fn init(&mut self) {
        let sub_message: String = "{\"type\":\"subscribe\",\"product_ids\":[\"ETH-USD\"],\"channels\":[\"level2\"]}".to_string();
        self.client.send(tokio_tungstenite::tungstenite::protocol::Message::Text(sub_message)).await;
        self.receive().await;
    }

    async fn receive(&mut self) {
        let mut count: usize = 0;
        let mut total: usize = 0;
        let mut counts: bool = false;
        while let Some(msg) = self.client.receive().await {
            let start = Instant::now();
            match msg {
                Ok(msg) => {
                    let (message, _) = serde_json_core::from_str::<Message>(&msg.to_text().unwrap()).expect("Parsing error");
                    match message.msg_type {
                        "subscriptions" => {
                            println!("Received subscription confirmation");
                            counts = false;
                        },
                        "snapshot" => {
                            counts = false;
                            self.handle_snapshot(&msg.to_text().unwrap())
                        },
                        "l2update" => {
                            counts = true;
                            self.handle_update(&msg.to_text().unwrap());
                        },
                        other => println!("Unknown message type {:?}: {:?}", other, msg),
                    }
                },
                Err(err) => println!("{:?}", err)
            }
            if counts {
                let duration = start.elapsed();
                count = count + 1;
                total = total + duration.as_micros() as usize;
                let avg: f64 = (total as f64) / (count as f64);
            }
        }
    }

    fn handle_snapshot(&mut self, snapshot: &str) {
        let result = serde_json_core::from_str::<Snapshot>(snapshot);
        match result {
            Ok((snapshot, _)) => {
                self.adapter.init_order_book(snapshot);
            },
            Err(err) => println!("Error parsing: {:?} for {:?}", err, snapshot),
        }        
    }

    fn handle_update(&mut self, update: & str) {
        let result = serde_json_core::from_str::<Update>(update);
        match result {
            Ok((update, _)) => {
                self.adapter.update(update);
            }
            Err(err) => println!("Error parsing: {:?} for {:?}", err, update),
        }
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