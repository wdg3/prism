mod order_book;
use order_book::order_book::OrderBook;
use order_book::order_book::MultiBook;
use order_book::order_book::Spread;

fn main() {
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
}