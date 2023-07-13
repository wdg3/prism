use std::{time::Duration, sync::Arc};

use tokio::{time::Instant, sync::Mutex};

use crate::order_book::{clients::client::WebSocketClient, order_book::MultiBook};

use super::{gemini_adapter::GeminiAdapter, data_types::{Update, Snapshot, Message}};

pub struct GeminiReceiveClient {
    adapter: GeminiAdapter,
    client: WebSocketClient,
    pair: heapless::String<8>,
}

impl<'a> GeminiReceiveClient {
    pub async fn new(multi_book: Arc<Mutex<MultiBook<3, 6>>>, pair: heapless::String<8>) -> GeminiReceiveClient {
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
                    let res = if init {
                        Message::Update {content: serde_json_core::from_str::<Update>(&msg.to_text().unwrap())}
                    } else {
                        Message::Snapshot {content: serde_json_core::from_str::<Snapshot>(&msg.to_text().unwrap())}
                    };
                    match res {
                        Message::Snapshot {content: snapshot} => {
                            match snapshot {
                                Ok((m, _)) => {
                                    println!("{:?}", msg);
                                    self.handle_snapshot(m).await;
                                    init = true;
                                },
                                Err(_) => {},
                            }
                        },
                        Message::Update {content: update} => {
                            match update {
                                Ok((msg, _)) => {
                                    let parse_dur = start.elapsed();
                                    self.handle_update(msg).await;
                                    let duration = start.elapsed();
                                    count = count + 1;
                                    total = total + duration.as_nanos() as usize;
                                    let avg: f64 = (total as f64) / (count as f64);
                                    //println!("Gemini: message parsed in {:?}", parse_dur);
                                    //println!("Gemini: message handled in {:?}", duration);
                                    //println!("Gemini: average message handle time for {:?} messages: {:?}", count, Duration::new(0, avg as u32));
                                },
                                Err(_) => (),
                            }
                        },

                    }
                },
                Err(err) => println!("Gemini: {:?}", err)
            }
        }
    }

    async fn handle_snapshot(&mut self, snapshot: Snapshot) {
        self.adapter.init_order_book(snapshot).await;
    }

    async fn handle_update(&mut self, update: Update) {
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