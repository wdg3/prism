use std::time::Duration;

use tokio::time::Instant;

use crate::order_book::clients::client::WebSocketClient;

use super::{kraken_adapter::KrakenAdapter, data_types::Message};
use super::super::super::clients::coinbase::{data_types::{Snapshot, Update}};

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
        let sub_message: String = "{\"event\": \"subscribe\",\"pair\": [\"ETH/USD\"],\"subscription\": {\"name\": \"book\", \"depth\": 10}}".to_string();
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
                    let res = serde_json_core::from_str::<Message>(&msg.to_text().unwrap());
                    match res {
                        Ok(msg) => {
                            println!("{:?}", msg);
                        },
                        Err(e) => {
                            println!("Error parsing: {:?} for {:?}", e, &msg.to_text().unwrap());
                        }
                    }/*
                    match message.msg_type {
                        "subscriptions" => {
                            println!("Received subscription confirmation");
                        },
                        "snapshot" => {
                            self.handle_snapshot(&msg.to_text().unwrap())
                        },
                        "l2update" => {
                            let duration = start.elapsed();
                            count = count + 1;
                            total = total + duration.as_nanos() as usize;
                            let avg: f64 = (total as f64) / (count as f64);
                            println!("Message parsed in {:?}", duration);
                            println!("Average message parse time: {:?}", Duration::new(0, avg as u32));
                            self.handle_update(&msg.to_text().unwrap());
                        },
                        other => println!("Unknown message type {:?}: {:?}", other, msg),
                    }*/
                },
                Err(err) => println!("{:?}", err)
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