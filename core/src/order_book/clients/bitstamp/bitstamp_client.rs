use std::{time::Duration, sync::Arc};

use tokio::{time::Instant, sync::Mutex};

use crate::order_book::{clients::{client::WebSocketClient}, order_book::MultiBook};

use super::{bitstamp_adapter::BitstampAdapter, data_types::{Update, Message}};

pub struct BitstampReceiveClient {
    adapter: BitstampAdapter,
    client: WebSocketClient,
    pair: heapless::String<8>,
}

impl<'a> BitstampReceiveClient {
    pub async fn new(book: Arc<Mutex<MultiBook<3, 6>>>, pair: heapless::String<8>) -> BitstampReceiveClient {
        return BitstampReceiveClient {
            adapter: BitstampAdapter::new(book).await,
            client: WebSocketClient::new("wss://ws.bitstamp.net".to_string()).await,
            pair: pair,
        }
    }

    pub async fn init(&mut self) {
        let p = match self.pair.as_str() {
            "ETH-USD" => "ethusd",
            "BTC-USD" => "btcusd",
            "ETH-USDT" => "ethusdt",
            "BTC-USDT" => "btcusdt",
            _ => panic!("Bad pair: {:?}", self.pair),
        };
        let sub_message: String = format!("{{\"event\": \"bts:subscribe\",\"data\": {{\"channel\": \"diff_order_book_{}\"}}}}", p).to_string();
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
                    match serde_json_core::from_str::<Message>(&msg.to_text().unwrap()) {
                        Ok((message, _)) => {
                            let update = message.data;
                            if init {
                                self.handle_update(update).await;
                                let duration = start.elapsed();
                                count = count + 1;
                                total = total + duration.as_nanos() as usize;
                                let avg: f64 = (total as f64) / (count as f64);
                                //println!("Bitstamp: message handled in {:?}", duration);
                                //println!("Bitstamp: average message handle time for {:?} messages: {:?}", count, Duration::new(0, avg as u32));
                            } else {
                                self.handle_snapshot(update).await;
                                init = true;
                            }
                        },
                        Err(_) => (),
                    }
                },
                Err(err) => {
                    println!("Bitstamp: {:?}\nAttempting reset.", err);
                    return
                }
            }
        }
    }

    async fn handle_snapshot(&mut self, snapshot: Update) {
        self.adapter.init_order_book(snapshot).await;
    }

    async fn handle_update(&mut self, update: Update) {
        self.adapter.update(update).await;
    }
}

pub struct BitstampSendClient {
    client: WebSocketClient,
}

impl BitstampSendClient {
    pub async fn new() -> Self {
        return BitstampSendClient {
            client: WebSocketClient::new("wss://ws.bitstamp.net".to_string()).await
        }
    }
}