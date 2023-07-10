mod order_book;

use std::time::Duration;

use tokio::runtime::Builder;
use tokio::time::{sleep_until, Instant};

use crate::order_book::clients::coinbase::coinbase_client::CoinbaseReceiveClient;
use crate::order_book::clients::kraken::kraken_client::KrakenReceiveClient;
use crate::order_book::order_book::MultiBook;

#[tokio::main]
async fn main() {
    const NUM_BOOKS: usize = 2;
    const NUM_PAIRS: usize = (NUM_BOOKS * (NUM_BOOKS - 1)) / 2;

    let runtime = Builder::new_multi_thread()
        .worker_threads(3)
        .thread_name("prism")
        .thread_stack_size(64 * 1024 * 1024)
        .enable_all()
        .build()
        .unwrap();
    
    
    let multi_book: MultiBook<NUM_BOOKS, NUM_PAIRS> = MultiBook::new();
    let coinbase_book = multi_book.coinbase_book.clone();
    let kraken_book = multi_book.kraken_book.clone();
    let coinbase_task = runtime.spawn(async move {
        let mut coinbase_client = CoinbaseReceiveClient::new(coinbase_book).await;
        coinbase_client.init().await;
    });
    
    let kraken_task = runtime.spawn(async move {
        let mut kraken_client = KrakenReceiveClient::new(kraken_book).await;
        kraken_client.init().await;
    });

    let multi_book_task = runtime.spawn(async move {
        loop {
            sleep_until(Instant::now() + Duration::from_millis(1000)).await;
            multi_book.print().await;
        }
    });
    
    coinbase_task.await.unwrap();
    kraken_task.await.unwrap();
    multi_book_task.await.unwrap();
}

    /*let gemini_task = tokio::spawn(async move {
        let mut gemini_client = WebSocketClient::new("wss://api.gemini.com/v1/marketdata/ETHUSD".to_string()).await;
        gemini_client.receive().await;
    });


    let binance_task = tokio::spawn(async move {
        let mut binance_client = WebSocketClient::new("wss://testnet.binance.vision/ws-api/v3".to_string()).await;
        let binance_sub: String = "{\"method\": \"SUBSCRIBE\",\"params\": {\"symbol\": \"ethusd@aggTrade\"},\"id\": 1}".to_string();
        binance_client.send(Message::Text(binance_sub)).await;
        binance_client.receive().await;
    });*/