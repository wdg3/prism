use std::sync::Arc;

use tokio::sync::Mutex;

use crate::order_book::multi_book::MultiBook;

pub struct BinanceAdapter {
    multi_book: Arc<Mutex<MultiBook<3, 6>>>,
    book_idx: usize,
}

impl BinanceAdapter {
    pub fn new(book: Arc<Mutex<MultiBook<3, 6>>>) -> BinanceAdapter {
        return BinanceAdapter {
            multi_book: book,
            book_idx: 2,
         }
    }
}