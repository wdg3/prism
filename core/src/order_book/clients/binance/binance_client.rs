use std::{sync::Arc, time::Duration};

use chrono::Utc;
use futures_util::StreamExt;
use tokio::{net::{TcpStream, TcpSocket}, sync::Mutex};
use tokio_tungstenite::{WebSocketStream, accept_async};

use crate::order_book::{multi_book::MultiBook, clients::binance::data_types::InboundMessage};

use super::{binance_adapter::BinanceAdapter, data_types::{Update, Change, Side, PriceLevel}};

pub struct BinanceReceiveClient {
    adapter: BinanceAdapter,
    stream: WebSocketStream<TcpStream>,
}

impl BinanceReceiveClient {
    pub async fn new(books: heapless::Vec::<Arc<Mutex<MultiBook<3, 6>>>, 2>) -> BinanceReceiveClient {
        let addr = "0.0.0.0:6969".parse().unwrap();
        let socket = TcpSocket::new_v4().expect("Error creating socket");
        socket.set_nodelay(true).unwrap();
        socket.bind(addr).unwrap();
        let (connection, _) = socket.listen(1024).expect("No connections to accept").accept().await.expect("Error accepting");
        let stream = accept_async(connection).await.expect("Failed to accept connection");
        return BinanceReceiveClient {
            adapter: BinanceAdapter::new(books),
            stream: stream,
        }
    }

    pub async fn init(&mut self) {
        self.receive().await;
    }
    
    async fn receive(&mut self) {
        let mut total = 0;
        let mut count = 0;
        while let Some(msg) = self.stream.next().await {
            let (message, _) = serde_json_core::from_str::<InboundMessage>(msg.unwrap().to_text().unwrap()).unwrap();
            match message.sent {
                Some(t) => {
                    self.handle_trade(message).await;
                    let now = Utc::now();
                    let dur = now.timestamp_millis() - t;
                    count = count + 1;
                    total = total + dur;
                    let avg: f64 = (total as f64) / (count as f64);
                    if count % 1000 == 1 {
                        println!("Binance avg. sent to handled time: {:?}", Duration::new(0, (avg * 1000000.0) as u32));
                    }
                },
                None => self.handle_book_update(message).await
            }
        }
    }

    async fn handle_book_update(&mut self, message: InboundMessage) {
        let update = Update {
            best_bid: Change {
                side: Side::Buy,
                level: PriceLevel {
                    level: (message.bid_level.unwrap().parse::<f64>().unwrap() * 100.) as usize,
                    amount: message.bid_amount.unwrap().parse::<f64>().unwrap(),
                }
            },
            best_ask: Change {
                side: Side::Sell,
                level: PriceLevel {
                    level: (message.ask_level.unwrap().parse::<f64>().unwrap() * 100.) as usize,
                    amount: message.ask_amount.unwrap().parse::<f64>().unwrap(),
                }
            }
        };
        self.adapter.handle_book_update(update, &message.pair).await;
    }

    async fn handle_trade(&mut self, message: InboundMessage) {
        let trade = Change {
            side: match message.buy.unwrap() {
                true => Side::Buy,
                false => Side::Sell,
            },
            level: PriceLevel {
                level: (message.price.unwrap().parse::<f64>().unwrap() * 100.) as usize,
                amount: message.amount.unwrap().parse::<f64>().unwrap(),
            }
        };
        self.adapter.handle_trade(trade, &message.pair).await;
    }
}