use std::sync::Arc;

use tokio::sync::Mutex;
use tokio::time::Instant;

use crate::order_book;
use crate::order_book::data_types::Change;
use crate::order_book::{multi_book::MultiBook, data_types::PriceLevel};
use super::data_types::{Update, Snapshot, Side};

use super::gemini_client::GeminiSendClient;

pub struct GeminiAdapter {
    multi_book: Arc<Mutex<MultiBook<3, 6>>>,
    send_client: GeminiSendClient,
    book_idx: usize,
}

impl<'a> GeminiAdapter {
    pub async fn new(book: Arc<Mutex<MultiBook<3, 6>>>) -> GeminiAdapter {
        return GeminiAdapter {
            multi_book: book,
            book_idx: 1,
            send_client: GeminiSendClient::new().await,
         }
    }

    pub async fn init_order_book(&mut self, snapshot: Snapshot) {
        println!("{:?}", snapshot.changes.len());
        let mut bids = Box::new(heapless::Vec::<PriceLevel, 65536>::new());
        let mut asks = Box::new(heapless::Vec::<PriceLevel, 65536>::new());
        for change in snapshot.changes.iter() {
            let p = &change.price_level;
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
        let mut guard = self.multi_book.lock().await;
        guard.books[self.book_idx].init(initial_book);
        guard.update_spread(self.book_idx);
    }

    fn trade() {

    }
    
    pub async fn update(&mut self, update: Update) {
        if update.changes.len() > 16 {
            panic!("Oversized update for Gemini");
        }
        
        //let start = Instant::now();
        let mut changes = heapless::Vec::<Change, 512>::new();
        //GeminiAdapter::elapsed(&start, "initialize update vec");
        //let start = Instant::now();
        for change in update.changes.iter() {
            let side = match change.side {
                Side::Sell => order_book::data_types::Side::Sell,
                Side::Buy => order_book::data_types::Side::Buy,
            };
            let _ = changes.push(Change {
                side: side,
                price_level: PriceLevel {level: change.price_level.level, amount: change.price_level.amount, sequence: 0}
            });
        }
        //GeminiAdapter::elapsed(&start, "populate update vec");
        //let start = Instant::now();
        let update = order_book::data_types::Update {product_id: "", time: "", changes: changes};
        //GeminiAdapter::elapsed(&start, "convert update vec format");
        //let start = Instant::now();
        let mut guard = self.multi_book.lock().await;
        //GeminiAdapter::elapsed(&start, "acquire lock");
        //let start = Instant::now();
        guard.books[self.book_idx].update(update);
        //GeminiAdapter::elapsed(&start, "update multibook order book");
        //let start = Instant::now();
        guard.update_spread(self.book_idx);
        //GeminiAdapter::elapsed(&start, "update multibook spreads");
    }
    fn elapsed(start: &Instant, msg: &str) {
        println!("Timer for {:?}: {:?}", msg, start.elapsed());
    }
}