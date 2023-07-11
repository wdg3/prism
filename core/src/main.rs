mod order_book;

use std::sync::Arc;
use std::time::Duration;

use tokio::runtime::Builder;
use tokio::sync::RwLock;

use crate::order_book::clients::coinbase::coinbase_client::CoinbaseReceiveClient;
use crate::order_book::clients::gemini::gemini_client::GeminiReceiveClient;
use crate::order_book::clients::kraken::kraken_client::KrakenReceiveClient;
use crate::order_book::order_book::MultiBook;

#[tokio::main]
async fn main() {
    const NUM_BOOKS: usize = 3;
    const NUM_PAIRS: usize = 2 * (NUM_BOOKS * (NUM_BOOKS - 1)) / 2;

    let runtime = Builder::new_multi_thread()
        .worker_threads(3)
        .thread_name("prism")
        .thread_stack_size(64 * 1024 * 1024)
        .enable_all()
        .build()
        .unwrap();
    
    
    let multi_book: MultiBook<NUM_BOOKS, NUM_PAIRS> = MultiBook::new();
    let multi_lock = Arc::new(RwLock::new(multi_book));
    let cb_multi_lock = multi_lock.clone();
    let gemini_multi_lock = multi_lock.clone();
    let kraken_multi_lock = multi_lock.clone();

    let coinbase_task = runtime.spawn(async move {
        let mut coinbase_client = CoinbaseReceiveClient::new(cb_multi_lock).await;
        coinbase_client.init().await;
    });
    
    let gemini_task = runtime.spawn(async move {
        let mut gemini_client = GeminiReceiveClient::new(gemini_multi_lock).await;
        gemini_client.init().await;
    });

    let kraken_task = runtime.spawn(async move {
        let mut kraken_client = KrakenReceiveClient::new(kraken_multi_lock).await;
        kraken_client.init().await;
    });
    
    coinbase_task.await.unwrap();
    gemini_task.await.unwrap();
    kraken_task.await.unwrap();
}

    /*let binance_task = tokio::spawn(async move {
        let mut binance_client = WebSocketClient::new("wss://testnet.binance.vision/ws-api/v3".to_string()).await;
        let binance_sub: String = "{\"method\": \"SUBSCRIBE\",\"params\": {\"symbol\": \"ethusd@aggTrade\"},\"id\": 1}".to_string();
        binance_client.send(Message::Text(binance_sub)).await;
        binance_client.receive().await;
    });*/