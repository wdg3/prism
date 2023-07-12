use std::{time::Duration, sync::Arc};

use tokio::{time::Instant, sync::RwLock};

use crate::order_book::{clients::client::WebSocketClient, order_book::MultiBook};

use super::{gemini_adapter::GeminiAdapter, data_types::{Content}};

pub struct GeminiReceiveClient {
    adapter: GeminiAdapter,
    client: WebSocketClient,
    pair: heapless::String<8>,
}

impl<'a> GeminiReceiveClient {
    pub async fn new(multi_book: Arc<RwLock<MultiBook<3, 6>>>, pair: heapless::String<8>) -> GeminiReceiveClient {
        return GeminiReceiveClient {
            adapter: GeminiAdapter::new(multi_book).await,
            client: WebSocketClient::new("wss://api.gemini.com/v2/marketdata".to_string()).await,
            pair: pair,
        }
    }

    pub async fn init(&mut self) {
        let p = match self.pair.as_str() {
            "ETH-USD" => "ETHUSD",
            "BTC-USD" => "BTCUSD",
            "ETH-USDT" => "ETHUSDT",
            "BTC-USDT" => "BTCUSDT",
            _ => panic!("Bad pair: {:?}", self.pair),
        };
        let sub_message: String = format!("{{\"type\":\"subscribe\",\"subscriptions\":[{{\"name\":\"l2\",\"symbols\":[{:?}]}}]}}", p).to_string();
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
                    let res = serde_json_core::from_str::<Content>(&msg.to_text().unwrap());
                    match res {
                        Ok((msg, _)) => {
                            let duration = start.elapsed();
                            count = count + 1;
                            total = total + duration.as_nanos() as usize;
                            let avg: f64 = (total as f64) / (count as f64);
                            //println!("Gemini: message parsed in {:?}", duration);
                            //println!("Gemini: average message parse time: {:?}", Duration::new(0, avg as u32));
                            if !init {
                                self.handle_snapshot(msg).await;
                                init = true;
                            } else {
                                self.handle_update(msg).await;
                            }
                        },
                        Err(_) => {}
                    }
                },
                Err(err) => println!("Gemini: {:?}", err)
            }
        }
    }

    async fn handle_snapshot(&mut self, snapshot: Content) {
        self.adapter.init_order_book(snapshot).await;
    }

    async fn handle_update(&mut self, update: Content) {
        self.adapter.update(update).await;
    }
}

pub struct GeminiSendClient {
    client: WebSocketClient,
}

impl GeminiSendClient {
    pub async fn new() -> Self {
        return GeminiSendClient {
            client: WebSocketClient::new("wss://api.gemini.com/v2/marketdata/".to_string()).await
        }
    }
}