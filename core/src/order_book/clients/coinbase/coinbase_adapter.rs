use crate::order_book::order_book::OrderBook;
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
        self.order_book.init(snapshot);
    }

    fn trade() {

    }
    
    pub fn update(&mut self, update: Update) {
        self.order_book.update(update);
    }
}