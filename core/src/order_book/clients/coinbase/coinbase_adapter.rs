use crate::order_book::data_types::{Change, Side};
use crate::order_book::{order_book::OrderBook, data_types::PriceLevel};
use crate::order_book;
use super::coinbase_client;
use super::{coinbase_client::CoinbaseSendClient, data_types::{Snapshot, Update}};

pub struct CoinbaseAdapter {
    order_book: OrderBook,
    send_client: CoinbaseSendClient,
}

impl<'a> CoinbaseAdapter {
    pub async fn new() -> CoinbaseAdapter {
        return CoinbaseAdapter {
            order_book: OrderBook::new(),
            send_client: CoinbaseSendClient::new().await,
         }
    }

    pub fn init_order_book(&mut self, snapshot: Snapshot) {
        let mut bids = heapless::Vec::<PriceLevel, 10000>::new();
        let mut asks = heapless::Vec::<PriceLevel, 10000>::new();
        for bid in snapshot.bids.iter() {
            bids.push(PriceLevel {level: bid.level, amount: bid.amount, sequence: 0});
        }
        for ask in snapshot.asks.iter() {
            asks.push(PriceLevel {level: ask.level, amount: ask.amount, sequence: 0});
        }
        self.order_book.init(order_book::data_types::Snapshot {bids: bids, asks: asks});
    }

    fn trade() {

    }
    
    pub fn update(&mut self, update: Update) {
        let mut changes = heapless::Vec::<Change, 32>::new();
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
        self.order_book.update(order_book::data_types::Update {product_id: "", time: "", changes: changes});
    }
}