use chrono::Local;

use super::{data_types::Side, order_book::OrderBook};

#[derive(Clone, Copy, Debug, Default)]
pub struct Spread {
    pub raw: isize,
    pub percentage: f64,
    pub seqs: [i64; 2],
}

pub struct MultiBook<const S: usize, const T: usize> {
    pub pair: heapless::String<8>,
    pub books: Box<heapless::Vec<OrderBook, S>>,
    pub spreads: heapless::Vec<Spread, T>,
    last_spreads: heapless::Vec<Spread, T>,
    arb_count: usize,
    o25: usize,
    o20: usize,
    o15: usize,
    o10: usize,
    o05: usize,
    max: f64,
}

impl<const S: usize, const T: usize> MultiBook<S, T> {
    pub fn new(pair: heapless::String<8>, names: [heapless::String<8>; S]) -> Self {
        let mut books = heapless::Vec::<OrderBook, S>::new();
        let mut spreads = heapless::Vec::<Spread, T>::new();
        let mut last_spreads = heapless::Vec::<Spread, T>::new();
        for i in 0..S {
            let _ = books.push(OrderBook::new(names[i].to_owned(), pair.to_owned()));
        }
        for _ in 0..T {
            let _ = spreads.push(Spread::default());
            let _ = last_spreads.push(Spread::default());
        }
        return MultiBook {
            pair: pair,
            books: Box::new(books),
            spreads: spreads,
            last_spreads: last_spreads,
            arb_count: 0,
            o25: 0,
            o20: 0,
            o15: 0,
            o10: 0,
            o05: 0,
            max: 0.0,
        }
    }

    pub fn update_spread(&mut self, book_idx: usize) {
        let price = self.books[book_idx].theoretical_price;
        for i in 0..S {
            if i != book_idx {
                let b = self.books[i].best_bid;
                let a = self.books[i].best_ask;
                if b.is_some() && a.is_some() {
                    let buy_spread = self.spread_from_levels(
                        a.unwrap() as isize,
                        price as isize,
                        [
                            self.get_best(Side::Sell, &self.books[i]).unwrap().1,
                            self.get_best(Side::Buy, &self.books[book_idx]).unwrap().1
                        ]
                    );
                    let sell_spread = self.spread_from_levels(
                        price as isize,
                        b.unwrap() as isize,
                        [
                            self.get_best(Side::Sell, &self.books[book_idx]).unwrap().1,
                            self.get_best(Side::Buy, &self.books[i]).unwrap().1
                        ]
                    );
                    let mut spread_idx = (book_idx * S) + i;
                    if i < book_idx {
                        spread_idx -= book_idx;
                    } else {
                        spread_idx -= book_idx + 1;
                    }
                    self.spreads[spread_idx] = buy_spread;
                    
                    let mut spread_idx = (i * S) + book_idx;
                    if book_idx < i {
                        spread_idx -= i;
                    } else {
                        spread_idx -= i + 1;
                    }
                    self.spreads[spread_idx] = sell_spread;
                }
            }
        }
        for i in 0..T {
            let spread = &self.spreads[i];
            if spread.percentage >= 0.0025 {
                self.o25 += 1;
            }
            if spread.percentage >= 0.002 {
                self.o20 += 1;
            }
            if spread.percentage >= 0.0015 {
                self.o15 += 1;
            }
            if spread.percentage >= 0.001 {
                self.o10 += 1;
            }
            if spread.percentage >= 0.0005 {
                self.o05 += 1;
            }
            if spread.percentage >= self.max {
                self.max = spread.percentage;
            }
            if spread.percentage >= 0.002 && (self.last_spreads[i].seqs[0] == 0 || (spread.seqs[0] != self.last_spreads[i].seqs[0] || spread.seqs[1] != self.last_spreads[i].seqs[1])) {
                self.last_spreads[i] = spread.clone();
                self.arb_count += 1;
                self.print();
                return;
            }
        }
    }
    fn spread_from_levels(&self, ask: isize, bid: isize, seqs: [i64; 2]) -> Spread {
        return Spread {raw: bid - ask, percentage: (bid - ask) as f64 / ask as f64, seqs: seqs}
    }
    pub fn print(&self) {
        println!("{:?}", self.pair);
        for book in self.books.iter() {
            self.print_book(&book);
        }
        for spread in &*self.spreads {
            println!("{:?}", spread);
        }
        let date = Local::now();
        println!("Arbitrage opportunity count: {:?}", self.arb_count);
        println!(">0.2%: {:?}\n>0.15%: {:?}\n>0.1%: {:?}\n>0.05%: {:?}", self.o20, self.o15, self.o10, self.o05);
        println!("Best seen: {:.5}%", self.max * 100.0);
        println!("{}", date.format("%Y-%m-%d %H:%M:%S"));
    }
    fn print_book(&self, book: &OrderBook) {

        if book.best_bid.is_some() && book.best_ask.is_some() {
            let bid = book.bid_lookup.get(&book.best_bid.unwrap());
            let ask = book.ask_lookup.get(&book.best_ask.unwrap());
            let bid_hs = book.bids.len();
            let ask_hs = book.asks.len();
            println!("{:?} best bid: {:?}\n{:?} best ask: {:?}", book.name, bid, book.name, ask);
            println!("Book pressure: {:?}", book.pressure);
            println!("Theoretical price: {:?}", book.theoretical_price);
            println!("Bid heap: {:?} elements\nAsk heap: {:?} elements", bid_hs, ask_hs);
        }
    }
    fn get_best(&self, side: Side, book: &OrderBook) -> Option<(usize, i64)> {
        match side {
            Side::Buy => {
                match book.best_bid {
                    Some(b) => Some((b, book.bid_lookup.get(&b).unwrap().sequence)),
                    None => None,
                }
            },
            Side::Sell => {
                match book.best_ask {
                    Some(a) => Some((a, book.ask_lookup.get(&a).unwrap().sequence)),
                    None => None,
                }
            },
        }
    }
}