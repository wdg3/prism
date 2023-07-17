use std::sync::Arc;

use tokio::sync::Mutex;

use crate::order_book::data_types::{Change, Side};
use crate::order_book::{multi_book::MultiBook, data_types::PriceLevel};
use crate::order_book;
use super::{bitstamp_client::BitstampSendClient, data_types::Update};

pub struct BitstampAdapter {
    multi_book: Arc<Mutex<MultiBook<3, 6>>>,
    send_client: BitstampSendClient,
    book_idx: usize,
}

impl<'a> BitstampAdapter {
    pub async fn new(book: Arc<Mutex<MultiBook<3, 6>>>) -> BitstampAdapter {
        return BitstampAdapter {
            multi_book: book,
            send_client: BitstampSendClient::new().await,
            book_idx: 2,
         }
    }

    pub async fn init_order_book(&mut self, snapshot: Update) {
        let mut bids = Box::new(heapless::Vec::<PriceLevel, 65536>::new());
        let mut asks = Box::new(heapless::Vec::<PriceLevel, 65536>::new());
        for bid in snapshot.bids.iter() {
            let _ = bids.push(PriceLevel {level: bid.level, amount: bid.amount, sequence: 0});
        }
        for ask in snapshot.asks.iter() {
            let _ = asks.push(PriceLevel {level: ask.level, amount: ask.amount, sequence: 0});
        }
        let initial_book = order_book::data_types::Snapshot {bids: Box::new(*bids), asks: Box::new(*asks)};
        let mut guard = self.multi_book.lock().await;
        guard.books[self.book_idx].init(initial_book);
        guard.update_spread(self.book_idx);
    }

    fn trade() {

    }
    
    pub async fn update(&mut self, update: Update) {
        if update.asks.len() >= 128 || update.bids.len() >= 128 {
            panic!("Oversized update for Bitstamp");
        }
        let mut changes = heapless::Vec::<Change, 512>::new();
        for bid in update.bids.iter() {
            let _ = changes.push(Change {
                side: Side::Buy,
                price_level: PriceLevel {
                    level: bid.level,
                    amount: bid.amount,
                    sequence: 0
                },
            });
        }
        for ask in update.asks.iter() {
            let _ = changes.push(Change {
                side: Side::Sell,
                price_level: PriceLevel {
                    level: ask.level,
                    amount: ask.amount,
                    sequence: 0
                },
            });
        }
        let update = order_book::data_types::Update {product_id: "", time: "", changes: changes};
        let mut guard = self.multi_book.lock().await;
        guard.books[self.book_idx].update(update);
        guard.update_spread(self.book_idx);
    }
}