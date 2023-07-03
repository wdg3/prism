mod order_book;
use order_book::clients::client::WebSocketClient;
use order_book::order_book::OrderBook;
use order_book::order_book::MultiBook;
use order_book::order_book::Spread;
use futures_util::{StreamExt, SinkExt};
use tokio_tungstenite::tungstenite::protocol::Message;

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

    for spread in multi_book.spreads {
        println!("{:?}", spread);
    }

    let coinbase_client = WebSocketClient{address: "wss://ws-feed.exchange.coinbase.com".to_string()};
    let mut ws_stream = coinbase_client.init().await;

    let sub = "{\"type\": \"subscribe\",\"product_ids\": [\"ETH-USD\"],\"channels\": [\"ticker\"]}".to_string();
    ws_stream.send(Message::Text(sub)).await;

    while let Some(msg) = ws_stream.next().await {
        println!("{:?}", msg);
    }
}