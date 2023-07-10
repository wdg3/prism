use std::sync::Arc;

use tokio::sync::RwLock;

use crate::order_book;
use crate::order_book::data_types::Change;
use crate::order_book::{order_book::OrderBook, data_types::PriceLevel};
use super::data_types::{Content, Side};

use super::gemini_client::GeminiSendClient;

pub struct GeminiAdapter {
    order_book: Arc<RwLock<OrderBook>>,
    send_client: GeminiSendClient,
}

impl<'a> GeminiAdapter {
    pub async fn new(book: Arc<RwLock<OrderBook>>) -> GeminiAdapter {
        return GeminiAdapter {
            order_book: book,
            send_client: GeminiSendClient::new().await,
         }
    }

    pub async fn init_order_book(&mut self, snapshot: Content) {
        let mut bids = Box::new(heapless::Vec::<PriceLevel, 10000>::new());
        let mut asks = Box::new(heapless::Vec::<PriceLevel, 10000>::new());
        for change in snapshot.changes.iter() {
            let p = change.price_level;
            match change.side {
                Side::Buy => {
                    let _ = bids.push(PriceLevel {level: p.level, amount: p.amount, sequence: 0});
                },
                Side::Sell => {
                    let _ = asks.push(PriceLevel {level: p.level, amount: p.amount, sequence: 0});
                }
            }
        }
        let initial_book = order_book::data_types::Snapshot {bids: Box::new(*bids), asks: Box::new(*asks)};
        self.order_book.write().await.init(initial_book);
    }

    fn trade() {

    }
    
    pub async fn update(&mut self, update: Content) {
        let mut changes = heapless::Vec::<Change, 512>::new();
        for change in update.changes {
            let side = match change.side {
                Side::Sell => order_book::data_types::Side::Sell,
                Side::Buy => order_book::data_types::Side::Buy,
            };
            let _ = changes.push(Change {
                side: side,
                price_level: PriceLevel {level: change.price_level.level, amount: change.price_level.amount, sequence: 0}
            });
        }
        let update = order_book::data_types::Update {product_id: "", time: "", changes: changes};
        self.order_book.write().await.update(update);
    }
}