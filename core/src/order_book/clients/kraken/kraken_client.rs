use std::time::Duration;

use tokio::time::Instant;

use crate::order_book::clients::client::WebSocketClient;

use super::{kraken_adapter::KrakenAdapter, data_types::{Message, Content}};

pub struct KrakenReceiveClient {
    adapter: KrakenAdapter,
    client: WebSocketClient,
}

impl<'a> KrakenReceiveClient {
    pub async fn new() -> KrakenReceiveClient {
        return KrakenReceiveClient {
            adapter: KrakenAdapter::new().await,
            client: WebSocketClient::new("wss://ws.kraken.com".to_string()).await
        }
    }

    pub async fn init(&mut self) {
        let sub_message: String = "{\"event\": \"subscribe\",\"pair\": [\"ETH/USD\"],\"subscription\": {\"name\": \"book\", \"depth\": 100}}".to_string();
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
                    let res = serde_json_core::from_str::<Message>(&msg.to_text().unwrap());
                    match res {
                        Ok((msg, _)) => {
                            let duration = start.elapsed();
                            count = count + 1;
                            total = total + duration.as_nanos() as usize;
                            let avg: f64 = (total as f64) / (count as f64);
                            println!("Kraken: message parsed in {:?}", duration);
                            println!("Kraken: average message parse time: {:?}", Duration::new(0, avg as u32));
                            if !init {
                                self.handle_snapshot(msg.content);
                                init = true;
                            } else {
                                self.handle_update(msg.content);
                            }
                        },
                        Err(_) => {}
                    }
                },
                Err(err) => println!("{:?}", err)
            }
        }
    }

    fn handle_snapshot(&mut self, snapshot: Content) {
        self.adapter.init_order_book(snapshot);      
    }

    fn handle_update(&mut self, update: Content) {
        self.adapter.update(update);
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