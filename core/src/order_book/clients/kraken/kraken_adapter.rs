use crate::order_book::order_book::OrderBook;
use super::{kraken_client::KrakenSendClient};
use super::super::super::clients::coinbase::{data_types::{Snapshot, Update}};

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

    pub fn init_order_book(&mut self, snapshot: Snapshot) {
        self.order_book.init(snapshot);
    }

    fn trade() {

    }
    
    pub fn update(&mut self, update: Update) {
        self.order_book.update(update);
    }
}