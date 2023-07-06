mod order_book;
use order_book::order_book::OrderBook;
use order_book::order_book::MultiBook;
use order_book::order_book::Spread;
use tokio::runtime::Builder;

use crate::order_book::clients::coinbase::coinbase_client::CoinbaseReceiveClient;

#[tokio::main]
async fn main() {
    const NUM_BOOKS: usize = 4;
    const NUM_PAIRS: usize = NUM_BOOKS * NUM_BOOKS;

    let coinbase = OrderBook::default();
    let gemini = OrderBook::default();
    let kraken = OrderBook::default();
    let binance = OrderBook::default();

    let spreads = [Spread::default(); NUM_PAIRS];

    let mut multi_book = MultiBook::<NUM_BOOKS, NUM_PAIRS> {
        books: [coinbase, gemini, kraken, binance],
        spreads: spreads,
    };

    for i in 0..NUM_BOOKS {
        for j in 0..NUM_BOOKS {
            multi_book.update_spread(i, j);
        }
    }

    let runtime = Builder::new_multi_thread()
        .worker_threads(4)
        .thread_name("prism")
        .thread_stack_size(128 * 1024 * 1024)
        .enable_all()
        .build()
        .unwrap();

    let coinbase_task = runtime.spawn(async move {
        let mut coinbase_client = CoinbaseReceiveClient::new().await;
        coinbase_client.init().await;
    });

    /*let gemini_task = tokio::spawn(async move {
        let mut gemini_client = WebSocketClient::new("wss://api.gemini.com/v1/marketdata/ETHUSD".to_string()).await;
        gemini_client.receive().await;
    });

    let kraken_task = tokio::spawn(async move {
        let mut kraken_client: WebSocketClient = WebSocketClient::new("wss://ws.kraken.com".to_string()).await;
        let kraken_sub: String = "{\"event\": \"subscribe\",\"pair\": [\"ETH/USD\"],\"subscription\": {\"name\": \"ticker\"}}".to_string();
        kraken_client.send(Message::Text(kraken_sub)).await;
        kraken_client.receive().await;
    });

    let binance_task = tokio::spawn(async move {
        let mut binance_client = WebSocketClient::new("wss://testnet.binance.vision/ws-api/v3".to_string()).await;
        let binance_sub: String = "{\"method\": \"SUBSCRIBE\",\"params\": {\"symbol\": \"ethusd@aggTrade\"},\"id\": 1}".to_string();
        binance_client.send(Message::Text(binance_sub)).await;
        binance_client.receive().await;
    });

    coinbase_task.await.unwrap();
    gemini_task.await.unwrap();
    kraken_task.await.unwrap();
    binance_task.await.unwrap();*/
    coinbase_task.await.unwrap();
}