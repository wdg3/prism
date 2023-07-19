mod order_book;

use std::sync::Arc;
use std::time::Duration;

use order_book::clients::bitstamp::bitstamp_client::BitstampReceiveClient;
use tokio::runtime::Builder;
use tokio::sync::Mutex;
use tokio::task::JoinHandle;

use crate::order_book::clients::coinbase::coinbase_client::CoinbaseReceiveClient;
use crate::order_book::clients::gemini::gemini_client::GeminiReceiveClient;
use crate::order_book::clients::kraken::kraken_client::KrakenReceiveClient;
use crate::order_book::multi_book::MultiBook;

const NUM_EXCHANGES: usize = 3;
const NUM_EXCHANGE_PAIRS: usize = 2 * (NUM_EXCHANGES * (NUM_EXCHANGES - 1)) / 2;
const NUM_CURRENCY_PAIRS: usize = 2;
const CURRENCY_PAIRS: [&'static str; NUM_CURRENCY_PAIRS] = ["ETH-USD", "BTC-USD"];
const NUM_MULTI_BOOKS: usize = NUM_EXCHANGES * NUM_CURRENCY_PAIRS;

async fn init_pair(pair: heapless::String<8>, runtime: &tokio::runtime::Runtime) {
    let runtime = Builder::new_multi_thread()
        .worker_threads(3)
        .thread_name("prism")
        .thread_stack_size(64 * 1024 * 1024)
        .enable_all()
        .build()
        .unwrap();
}

#[tokio::main]
async fn main() {
    let runtime = Builder::new_multi_thread()
        .worker_threads(12)
        .thread_name("prism")
        .thread_stack_size(64 * 1024 * 1024)
        .enable_all()
        .build()
        .unwrap();
    let mut pair_task_vec = heapless::Vec::<JoinHandle<()>, NUM_MULTI_BOOKS>::new();
    let mut multi_book_vec = heapless::Vec::<Arc<Mutex<MultiBook<NUM_EXCHANGES, NUM_EXCHANGE_PAIRS>>>, NUM_CURRENCY_PAIRS>::new();
    
    for pair in CURRENCY_PAIRS.iter() {
        let multi_book: MultiBook<NUM_EXCHANGES, NUM_EXCHANGE_PAIRS> = MultiBook::new(
            heapless::String::from(pair.to_owned()),
            [
                heapless::String::from("coinbase"), 
                //heapless::String::from("gemini"),
                heapless::String::from("kraken"),
                heapless::String::from("bitstamp"),
            ]
        );
        let multi_lock = Arc::new(Mutex::new(multi_book));
        let _ = multi_book_vec.push(multi_lock.clone());
        let cb_multi_lock = multi_lock.clone();
        //let gemini_multi_lock = multi_lock.clone();
        let kraken_multi_lock = multi_lock.clone();
        //let bitstamp_multi_lock = multi_lock.clone();
    
        let coinbase_task = runtime.spawn(async move {
            loop {
                let lock = cb_multi_lock.clone();
                let pair = heapless::String::<8>::from(*pair);
                let mut coinbase_client = CoinbaseReceiveClient::new(lock, pair).await;
                coinbase_client.init().await;
            }
        });
        
        /*let gemini_task = runtime.spawn(async move {
            loop {
                let lock = gemini_multi_lock.clone();
                let pair = heapless::String::<8>::from(*pair);
                let mut gemini_client = GeminiReceiveClient::new(lock, pair).await;
                gemini_client.init().await;
            }
        });*/
    
        let kraken_task = runtime.spawn(async move {
            loop {
                let lock = kraken_multi_lock.clone();
                let pair = heapless::String::<8>::from(*pair);
                let mut kraken_client = KrakenReceiveClient::new(lock, pair).await;
                kraken_client.init().await;
            }
        });
        /*let bitstamp_task = runtime.spawn(async move {
            loop {
                let lock = bitstamp_multi_lock.clone();
                let pair = heapless::String::<8>::from(*pair);
                let mut bitstamp_client = BitstampReceiveClient::new(lock, pair).await;
                bitstamp_client.init().await;
            }
        });*/

        let _ = pair_task_vec.push(coinbase_task);
        //let _ = pair_task_vec.push(gemini_task);
        let _ = pair_task_vec.push(kraken_task);
        //let _ = pair_task_vec.push(bitstamp_task);
    }
    let monitor_task = runtime.spawn(async move {
        loop {
            tokio::time::sleep(Duration::from_secs(60 * 10)).await;
            for lock in multi_book_vec.iter() {
                lock.lock().await.print()
            }
        }
    });
    for pair_task in pair_task_vec {
        pair_task.await.unwrap();
    }
    monitor_task.await.unwrap();
}

    /*let binance_task = tokio::spawn(async move {
        let mut binance_client = WebSocketClient::new("wss://data-stream.binance.vision/ws-api/v3".to_string()).await;
        let binance_sub: String = "{\"method\": \"SUBSCRIBE\",\"params\": {\"symbol\": \"ethusd@aggTrade\"},\"id\": 1}".to_string();
        binance_client.send(Message::Text(binance_sub)).await;
        binance_client.receive().await;
    });*/