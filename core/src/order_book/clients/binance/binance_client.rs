use std::{sync::Arc, time::Duration};

use chrono::Utc;
use futures_util::StreamExt;
use tokio::{net::{TcpStream, TcpSocket}, sync::Mutex};
use tokio_tungstenite::{WebSocketStream, accept_async};

use crate::order_book::{multi_book::MultiBook, clients::binance::data_types::InboundMessage};

use super::binance_adapter::BinanceAdapter;

pub struct BinanceReceiveClient {
    //adapter: BinanceAdapter,
    stream: WebSocketStream<TcpStream>,
    pair: heapless::String<8>,
}

impl BinanceReceiveClient {
    pub async fn new(pair: heapless::String<8>) -> BinanceReceiveClient {
        let addr = "0.0.0.0:6969".parse().unwrap();
        let socket = TcpSocket::new_v4().expect("Error creating socket");
        socket.set_nodelay(true).unwrap();
        socket.bind(addr).unwrap();
        let (connection, _) = socket.listen(1024).expect("No connections to accept").accept().await.expect("Error accepting");
        let stream = accept_async(connection).await.expect("Failed to accept connection");
        return BinanceReceiveClient {
            //adapter: BinanceAdapter::new(book),
            stream: stream,
            pair: pair,
        }
    }

    pub async fn init(&mut self) {
        let p = match self.pair.as_str() {
            "ETH-USD" => "ethusdt",
            "BTC-USD" => "btcusdt",
            _ => panic!("Bad pair: {:?}", self.pair),
        };
        self.receive().await;
    }
    
    async fn receive(&mut self) {
        let mut total = 0;
        let mut count = 0;
        while let Some(msg) = self.stream.next().await {
            let (message, _) = serde_json_core::from_str::<InboundMessage>(msg.unwrap().to_text().unwrap()).unwrap();
            match message.sent {
                Some(t) => {
                    let now = Utc::now();
                    let dur = now.timestamp_millis() - t;
                    println!("Binance sent to handled time: {:?}", dur);
                    count = count + 1;
                    total = total + dur;
                    let avg: f64 = (total as f64) / (count as f64);
                    println!("Binance avg. sent to handled time: {:?}", Duration::new(0, (avg * 1000000.0) as u32));
                },
                None => ()
            }
            println!("{:?}", message);
        }
    }
}