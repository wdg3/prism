mod client;
mod data_types;
use client::WebSocketClient;
use data_types::Trade;
use chrono::Utc;
use futures_util::SinkExt;
use tokio::net::TcpStream;
use std::time::Duration;
use serde_json::Value;
use tokio_tungstenite::{tungstenite::protocol::Message, client_async};

#[tokio::main]
async fn main() {
    let tcp = TcpStream::connect("ec2-44-199-208-158.compute-1.amazonaws.com:8080").await.expect("Failed to connect");
    let (mut book_client, _) = client_async("wss://ec2-44-199-208-158.compute-1.amazonaws.com:8080", tcp).await.expect("Client failed to connect");
    println!("Connected!");
    let mut binance_client = WebSocketClient::new("wss://stream.binance.com/ws/btcusdt@aggTrade".to_string()).await;
    let binance_sub: String = format!("{{\"method\": \"SUBSCRIBE\",\"params\": [\"btcusdt@aggTrade\"],\"id\": 1}}").to_string();
    let mut count = 0;
    let mut total = 0;
    binance_client.send(Message::Text(binance_sub)).await;
    while let Some(msg) = binance_client.receive().await {
        //let value = serde_json::from_str::<Value>(&msg.unwrap().to_text().unwrap()).unwrap();
        //let event_time = value.get("E");
        //let trade_time = value.get("T");
        let trade = serde_json_core::from_str::<Trade>(&msg.unwrap().to_text().unwrap());
        let e = match trade {
            Ok((t, _)) => {
                Some(t.sent)
            },
            Err(e) => None, 
        };
        match e {
            None => (),
            Some(e) => {
                let _ = book_client.send(Message::Binary(e.to_be_bytes().to_vec())).await;
                let now = Utc::now();
                let dur = now.timestamp_millis() - e;
                println!("Sent to handled time: {:?}", dur);
                count = count + 1;
                total = total + dur;
                let avg: f64 = (total as f64) / (count as f64);
                println!("Avg. sent to handled time: {:?}", Duration::new(0, (avg * 1000000.0) as u32));   
            }
        }
    }
}