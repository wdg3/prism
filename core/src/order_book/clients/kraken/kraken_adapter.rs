use std::sync::Arc;

use tokio::sync::RwLock;

use crate::order_book::data_types::{PriceLevel, Snapshot, Change, Side, Update};
use crate::order_book::order_book::OrderBook;
use super::data_types::Content;
use super::{kraken_client::KrakenSendClient};

pub struct KrakenAdapter {
    order_book: Arc<RwLock<OrderBook>>,
    send_client: KrakenSendClient,
}

impl<'a> KrakenAdapter {
    pub async fn new(book: Arc<RwLock<OrderBook>>) -> KrakenAdapter {
        return KrakenAdapter {
            order_book: book,
            send_client: KrakenSendClient::new().await,
         }
    }

    pub async fn init_order_book(&mut self, snapshot: Content) {
        let mut bids = Box::new(heapless::Vec::<PriceLevel, 10000>::new());
        let mut asks = Box::new(heapless::Vec::<PriceLevel, 10000>::new());
        for bid in snapshot.bids.unwrap().iter() {
            let _ = bids.push(PriceLevel {level: bid.level, amount: bid.amount, sequence: 0});
        }
        for ask in snapshot.asks.unwrap().iter() {
            let _ = asks.push(PriceLevel {level: ask.level, amount: ask.amount, sequence: 0});
        }
        let initial_book = Snapshot {bids: Box::new(*bids), asks: Box::new(*asks)};
        self.order_book.write().await.init(initial_book);
    }

    fn trade() {

    }
    
    pub async fn update(&mut self, update: Content) {
        let mut changes = heapless::Vec::<Change, 512>::new();
        if update.bids.is_some() {
            for bid in update.bids.unwrap().iter() {
                let _ = changes.push(Change{
                    side: Side::Buy,
                    price_level: PriceLevel {level: bid.level, amount: bid.amount, sequence: 0}});
            }
        }
        if update.asks.is_some() {
            for ask in update.asks.unwrap().iter() {
                let _ = changes.push(Change{
                    side: Side::Sell,
                    price_level: PriceLevel {level: ask.level, amount: ask.amount, sequence: 0}});
            }
        }
        let update = Update {product_id: "", time: "", changes: changes};
        self.order_book.write().await.update(update);
    }
}