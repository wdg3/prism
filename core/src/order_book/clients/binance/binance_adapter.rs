use std::sync::Arc;

use tokio::sync::Mutex;

use crate::order_book::{multi_book::MultiBook, self};

use super::data_types::{Update, Change, Side};

pub struct BinanceAdapter {
    books: heapless::Vec::<Arc<Mutex<MultiBook<3, 6>>>, 2>,
    book_idx: usize,
    pair_map: heapless::FnvIndexMap<heapless::String<8>, usize, 2>,
}

impl BinanceAdapter {
    pub fn new(books: heapless::Vec::<Arc<Mutex<MultiBook<3, 6>>>, 2>) -> BinanceAdapter {
        let mut pair_map = heapless::FnvIndexMap::<heapless::String::<8>, usize, 2>::new();
        pair_map.insert(heapless::String::<8>::from("ETHUSDT"), 0).unwrap();
        pair_map.insert(heapless::String::<8>::from("BTCUSDT"), 1).unwrap();
        return BinanceAdapter {
            books: books,
            book_idx: 2,
            pair_map: pair_map,
         }
    }

    pub async fn handle_book_update(&mut self, message: Update, pair: &heapless::String<8>) {
        let mut changes = heapless::Vec::<order_book::data_types::Change, 512>::new();
        let book = &mut self.books[*self.pair_map.get(&pair).unwrap()].lock().await.books[self.book_idx];
        let (curr_bid, curr_ask) = (book.best_bid, book.best_ask);
        if curr_bid.is_some() && message.best_bid.level.level != curr_bid.unwrap() {
            let _ = changes.push(order_book::data_types::Change {
                side: order_book::data_types::Side::Buy,
                price_level: order_book::data_types::PriceLevel {
                    level: curr_bid.unwrap(),
                    amount: 0.0,
                    sequence: 0,
                }
            });
        }
        if curr_ask.is_some() && message.best_ask.level.level != curr_ask.unwrap() {
            let _ = changes.push(order_book::data_types::Change {
                side: order_book::data_types::Side::Sell,
                price_level: order_book::data_types::PriceLevel {
                    level: curr_ask.unwrap(),
                    amount: 0.0,
                    sequence: 0,
                }
            });
        }
        let _ = changes.push(order_book::data_types::Change {
            side: order_book::data_types::Side::Buy,
            price_level: order_book::data_types::PriceLevel {
                level: message.best_bid.level.level,
                amount: message.best_bid.level.amount,
                sequence: 0,
            }
        });
        let _ = changes.push(order_book::data_types::Change {
            side: order_book::data_types::Side::Sell,
            price_level: order_book::data_types::PriceLevel {
                level: message.best_ask.level.level,
                amount: message.best_ask.level.amount,
                sequence: 0,
            }
        });
        let update = order_book::data_types::Update {product_id: "", time: "", changes: changes};
        book.update(update);
    }

    pub async fn handle_trade(&mut self, trade: Change, pair: &heapless::String<8>) {
        let side = match trade.side {
            Side::Buy => order_book::data_types::Side::Buy,
            Side::Sell => order_book::data_types::Side::Sell,
        };
        let new = order_book::data_types::Match {side: side, size: trade.level.amount, price: trade.level.level};
        let mut guard = self.books[*self.pair_map.get(&pair).unwrap()].lock().await;
        if guard.books[self.book_idx].best_ask.is_some() && guard.books[self.book_idx].best_bid.is_some() {
            guard.books[self.book_idx].update_impulse(new);
            guard.update_spread(self.book_idx);
        }
    }
}