use crate::order_book::data_types::{PriceLevel, Snapshot, Change, Side, Update};
use crate::order_book::order_book::OrderBook;
use super::data_types::Content;
use super::{kraken_client::KrakenSendClient};

pub struct KrakenAdapter {
    order_book: OrderBook,
    send_client: KrakenSendClient,
}

impl<'a> KrakenAdapter {
    pub async fn new() -> KrakenAdapter {
        return KrakenAdapter {
            order_book: OrderBook::new(),
            send_client: KrakenSendClient::new().await,
         }
    }

    pub fn init_order_book(&mut self, snapshot: Content) {
        let mut bids = heapless::Vec::<PriceLevel, 10000>::new();
        let mut asks = heapless::Vec::<PriceLevel, 10000>::new();
        for bid in snapshot.bids.unwrap().iter() {
            let _ = bids.push(PriceLevel {level: bid.level, amount: bid.amount, sequence: 0});
        }
        for ask in snapshot.asks.unwrap().iter() {
            let _ = asks.push(PriceLevel {level: ask.level, amount: ask.amount, sequence: 0});
        }
        self.order_book.init(Snapshot {bids: bids, asks: asks});
    }

    fn trade() {

    }
    
    pub fn update(&mut self, update: Content) {
        let mut changes = heapless::Vec::<Change, 32>::new();
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
        self.order_book.update(Update {product_id: "", time: "", changes: changes});
    }
}