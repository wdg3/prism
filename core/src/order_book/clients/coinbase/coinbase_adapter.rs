use crate::order_book::order_book::OrderBook;
use super::coinbase_client::CoinbaseSendClient;

pub struct CoinbaseAdapter {
    order_book: OrderBook,
    send_client: CoinbaseSendClient,
}

impl CoinbaseAdapter {
    pub async fn new() -> Self {
        return CoinbaseAdapter {
            order_book: OrderBook::default(),
            send_client: CoinbaseSendClient::new().await,
         }
    }

    fn trade() {

    }
    
    fn update() {
        
    }
}