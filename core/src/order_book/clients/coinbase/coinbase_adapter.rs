use std::sync::Arc;

use tokio::sync::RwLock;

use crate::order_book::data_types::{Change, Side};
use crate::order_book::{order_book::OrderBook, data_types::PriceLevel};
use crate::order_book;
use super::{coinbase_client::CoinbaseSendClient, data_types::{Snapshot, Update}};

pub struct CoinbaseAdapter {
    order_book: Arc<RwLock<OrderBook>>,
    send_client: CoinbaseSendClient,
}

impl<'a> CoinbaseAdapter {
    pub async fn new(book: Arc<RwLock<OrderBook>>) -> CoinbaseAdapter {
        return CoinbaseAdapter {
            order_book: book,
            send_client: CoinbaseSendClient::new().await,
         }
    }

    pub async fn init_order_book(&mut self, snapshot: Snapshot) {
        let mut bids = Box::new(heapless::Vec::<PriceLevel, 10000>::new());
        let mut asks = Box::new(heapless::Vec::<PriceLevel, 10000>::new());
        for bid in snapshot.bids.iter() {
            let _ = bids.push(PriceLevel {level: bid.level, amount: bid.amount, sequence: 0});
        }
        for ask in snapshot.asks.iter() {
            let _ = asks.push(PriceLevel {level: ask.level, amount: ask.amount, sequence: 0});
        }
        let initial_book = order_book::data_types::Snapshot {bids: Box::new(*bids), asks: Box::new(*asks)};
        self.order_book.write().await.init(initial_book);
    }

    fn trade() {

    }
    
    pub async fn update(&mut self, update: Update) {
        let mut changes = heapless::Vec::<Change, 512>::new();
        for change in update.changes {
            let side = match change.side {
                super::data_types::Side::Buy => Side::Buy,
                super::data_types::Side::Sell => Side::Sell,
            };
            let _ = changes.push(Change {
                side: side,
                price_level: PriceLevel {
                    level: change.price_level.level,
                    amount: change.price_level.amount,
                    sequence: 0
                }});
        }
        let update = order_book::data_types::Update {product_id: "", time: "", changes: changes};
        self.order_book.write().await.update(update);
    }
}