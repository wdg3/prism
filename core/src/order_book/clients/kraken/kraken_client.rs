use std::{time::Duration, sync::Arc};

use serde_json::Value;
use tokio::{time::Instant, sync::Mutex};

use crate::order_book::{clients::{client::WebSocketClient}, order_book::MultiBook};

use super::{kraken_adapter::KrakenAdapter, data_types::{Message}};

pub struct KrakenReceiveClient {
    adapter: KrakenAdapter,
    client: WebSocketClient,
    pair: heapless::String<8>,
}

impl<'a> KrakenReceiveClient {
    pub async fn new(multi_book: Arc<Mutex<MultiBook<3, 6>>>, pair: heapless::String<8>) -> KrakenReceiveClient {
        return KrakenReceiveClient {
            adapter: KrakenAdapter::new(multi_book).await,
            client: WebSocketClient::new("wss://ws.kraken.com".to_string()).await,
            pair: pair,
        }
    }

    pub async fn init(&mut self) {
        let p = match self.pair.as_str() {
            "ETH-USD" => "ETH/USD",
            "BTC-USD" => "XBT/USD",
            "BTC-USDT" => "XBT/USDT",
            "ETH-USDT" => "ETH/USDT",
            _ => panic!("Bad pair: {:?}", self.pair),
        };
        let sub_message: String = format!("{{\"event\": \"subscribe\",\"pair\": [{:?}],\"subscription\": {{\"name\": \"book\", \"depth\": 1000}}}}", p).to_string();
        self.client.send(tokio_tungstenite::tungstenite::protocol::Message::Text(sub_message)).await;
        self.receive().await;
    }

    async fn receive(&mut self) {
        let mut count: usize = 0;
        let mut total: usize = 0;
        let mut init = false;
        while let Some(msg) = self.client.receive().await {
            let start = Instant::now();
            match msg {
                Ok(msg) => {
                    match serde_json::from_str::<Value>(&msg.to_text().unwrap()) {
                        Ok(res) => {
                            match res {
                                Value::Array(arr) => {
                                    let message: Message;
                                    if arr.len() == 4 {
                                        message = Message::Single {
                                            content: serde_json::from_value(arr.get(1).unwrap().clone()).unwrap(),
                                        }
                                    } else if arr.len() == 5 {
                                        message = Message::Double {
                                            content_1: serde_json::from_value(arr.get(1).unwrap().clone()).unwrap(),
                                            content_2: serde_json::from_value(arr.get(2).unwrap().clone()).unwrap(),
                                        }
                                    } else {
                                        panic!("{:?}, {:?}", arr.len(), arr);
                                    }
                                    if !init {
                                        self.handle_snapshot(message).await;
                                        init = true;
                                    } else {
                                        self.handle_update(message).await;
                                        let duration = start.elapsed();
                                        count = count + 1;
                                        total = total + duration.as_nanos() as usize;
                                        let avg: f64 = (total as f64) / (count as f64);
                                        //println!("Kraken: message handled in {:?}", duration);
                                        //println!("Kraken: average message handle time for {:?} messages: {:?}", count, Duration::new(0, avg as u32));
                                    }
                                },
                                _ => (),
                            }
                        },
                        Err(e) => println!("Kraken parsing error for {:?}: {:?}", msg, e)
                    }
                },
                Err(err) => {
                    println!("Kraken: {:?}\nAttempting reset.", err);
                    return
                }
            }
        }
    }

    async fn handle_snapshot(&mut self, snapshot: Message) {
        self.adapter.init_order_book(snapshot).await;
    }

    async fn handle_update(&mut self, update: Message) {
        self.adapter.update(update).await;
    }
}

pub struct KrakenSendClient {
    client: WebSocketClient,
}

impl KrakenSendClient {
    pub async fn new() -> Self {
        return KrakenSendClient {
            client: WebSocketClient::new("wss://ws.kraken.com".to_string()).await
        }
    }
}