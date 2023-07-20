mod client;
mod data_types;
use client::WebSocketClient;
use data_types::{Trade, BookUpdate, OutboundMessage};
use chrono::Utc;
use futures_util::SinkExt;
use tokio::net::{TcpSocket, TcpStream};
use std::time::Duration;
//use serde_json::Value;
use tokio_tungstenite::{tungstenite::protocol::Message, client_async, WebSocketStream};

#[tokio::main]
async fn main() {
    let addr = "207.2.15.83:6969".parse().unwrap();
    let socket = TcpSocket::new_v4().expect("Failed to create socket");
    socket.set_nodelay(true).expect("Failed to disable Nagle's algorithm");
    let tcp = socket.connect(addr).await.expect("Failed to connect");
    let (mut book_client, _) = client_async("wss://ip-207-2-15-83.ec2.internal:6969", tcp).await.expect("Client failed to connect");
    println!("Connected!");
    let mut binance_client = WebSocketClient::new("wss://stream.binance.com/ws/btcusdt@aggTrade".to_string()).await;
    let binance_sub: String = format!("{{\"method\": \"SUBSCRIBE\",\"params\": [\"btcusdt@aggTrade\", \"btcusdt@bookTicker\", \"ethusdt@aggTrade\", \"ethusdt@bookTicker\"],\"id\": 1}}").to_string();
    let mut count = 0;
    let mut total = 0;
    binance_client.send(Message::Text(binance_sub)).await;
    while let Some(msg) = binance_client.receive().await {
        //let value = serde_json::from_str::<Value>(&msg.unwrap().to_text().unwrap()).unwrap();
        //let event_time = value.get("E");
        //let trade_time = value.get("T");
        let raw = heapless::String::<256>::from(msg.unwrap().to_text().unwrap());
        let message = match serde_json_core::from_str::<data_types::Message>(&raw) {
            Ok((m, _)) => m,
            Err(_) => continue,
        };
        println!("{:?}", message);
        match message.event_type {
            Some(_) => {
                let (trade, _) = serde_json_core::from_str::<Trade>(&raw).unwrap();
                let sent = trade.sent;
                handle_trade(trade, &mut book_client).await;
                let now = Utc::now();
                let dur = now.timestamp_millis() - sent;
                println!("Sent to handled time: {:?}", dur);
                count = count + 1;
                total = total + dur;
                let avg: f64 = (total as f64) / (count as f64);
                println!("Avg. sent to handled time: {:?}", Duration::new(0, (avg * 1000000.0) as u32));  
            },
            None => {
                match message.pair {
                    None => (),
                    Some(_) => {
                        println!("{:?}", raw);
                        let (update, _) = serde_json_core::from_str::<BookUpdate>(&raw).unwrap();
                        handle_book_update(update, &mut book_client).await;
                    },
                };
            }, 
        };
    }
}


async fn handle_trade(trade: Trade, client: &mut WebSocketStream<TcpStream>) {

    let out = OutboundMessage {
        message_type: heapless::String::<8>::from("trade"),
        pair: trade.pair,
        sent: Some(trade.sent),
        price: Some(trade.price),
        amount: Some(trade.amount),
        bid_level: None,
        ask_level: None,
        bid_amount: None,
        ask_amount: None,
    };
    let _ = client.send(Message::Text(serde_json_core::to_string::<OutboundMessage, 256>(&out).unwrap().to_string())).await;

}

async fn handle_book_update(update: BookUpdate, client: &mut WebSocketStream<TcpStream>) {
    let out = OutboundMessage {
        message_type: heapless::String::<8>::from("book"),
        pair: update.pair,
        sent: None,
        price: None,
        amount: None,
        bid_level: Some(update.bid_level),
        ask_level: Some(update.ask_level),
        bid_amount: Some(update.bid_amount),
        ask_amount: Some(update.ask_amount),
    };
    let _ = client.send(Message::Text(serde_json_core::to_string::<OutboundMessage, 256>(&out).unwrap().to_string())).await;
}
