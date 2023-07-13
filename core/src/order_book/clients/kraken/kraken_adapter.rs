use std::sync::Arc;

use tokio::sync::Mutex;

use crate::order_book::data_types::{PriceLevel, Snapshot, Change, Side, Update};
use crate::order_book::order_book::MultiBook;
use super::data_types::{Message::Single, Message::Double, Message};
use super::{kraken_client::KrakenSendClient};

pub struct KrakenAdapter {
    multi_book: Arc<Mutex<MultiBook<3, 6>>>,
    send_client: KrakenSendClient,
    book_idx: usize,
}

impl<'a> KrakenAdapter {
    pub async fn new(book: Arc<Mutex<MultiBook<3, 6>>>) -> KrakenAdapter {
        return KrakenAdapter {
            multi_book: book,
            book_idx: 2,
            send_client: KrakenSendClient::new().await,
         }
    }

    pub async fn init_order_book(&mut self, snapshot: Message) {
        let mut bids = Box::new(heapless::Vec::<PriceLevel, 65536>::new());
        let mut asks = Box::new(heapless::Vec::<PriceLevel, 65536>::new());
        let (c1, c2) = match snapshot {
            Single{content: c} => (Some(c), None),
            Double{content_1, content_2} => (Some(content_1), Some(content_2)),
        };
        if c1.is_some() {
            let c = c1.unwrap();
            for bid in c.bids.unwrap().iter() {
                let _ = bids.push(PriceLevel {level: bid.level, amount: bid.amount, sequence: 0});
            }
            for ask in c.asks.unwrap().iter() {
                let _ = asks.push(PriceLevel {level: ask.level, amount: ask.amount, sequence: 0});
            }
        }
        if c2.is_some() {
            let c = c2.unwrap();
            for bid in c.bids.unwrap().iter() {
                if !bid.republished {
                    let _ = bids.push(PriceLevel {level: bid.level, amount: bid.amount, sequence: 0});
                }
            }
            for ask in c.asks.unwrap().iter() {
                if !ask.republished {
                    let _ = asks.push(PriceLevel {level: ask.level, amount: ask.amount, sequence: 0});
                }
            }
        }
        let initial_book = Snapshot {bids: Box::new(*bids), asks: Box::new(*asks)};
        let mut guard = self.multi_book.lock().await;
        guard.books[self.book_idx].init(initial_book);
        guard.update_spread(self.book_idx);
    }

    fn trade() {

    }
    
    pub async fn update(&mut self, update: Message) {
        let mut changes = heapless::Vec::<Change, 512>::new();
        let (c1, c2) = match update {
            Single{content: c} => (Some(c), None),
            Double{content_1, content_2 } => (Some(content_1), Some(content_2)),
        };
        if c1.is_some() {
            let u = c1.unwrap();
            if u.bids.is_some() {
                for bid in u.bids.unwrap().iter() {
                    let _ = changes.push(Change{
                        side: Side::Buy,
                        price_level: PriceLevel {level: bid.level, amount: bid.amount, sequence: 0}});
                }
            }
            if u.asks.is_some() {
                for ask in u.asks.unwrap().iter() {
                    let _ = changes.push(Change{
                        side: Side::Sell,
                        price_level: PriceLevel {level: ask.level, amount: ask.amount, sequence: 0}});
                }
            }
        }
        if c2.is_some() {
            let u = c2.unwrap();
            if u.bids.is_some() {
                for bid in u.bids.unwrap().iter() {
                    let _ = changes.push(Change{
                        side: Side::Buy,
                        price_level: PriceLevel {level: bid.level, amount: bid.amount, sequence: 0}});
                }
            }
            if u.asks.is_some() {
                for ask in u.asks.unwrap().iter() {
                    let _ = changes.push(Change{
                        side: Side::Sell,
                        price_level: PriceLevel {level: ask.level, amount: ask.amount, sequence: 0}});
                }
            }
        }
        let update = Update {product_id: "", time: "", changes: changes};
        let mut guard = self.multi_book.lock().await;
        guard.books[self.book_idx].update(update);
        guard.update_spread(self.book_idx);
    }
}