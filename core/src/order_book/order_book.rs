#[derive(Default)]
pub struct OrderBook {
    pub bid: i64,
    pub bid_vol: u64,
    pub ask: i64,
    pub ask_vol: u64,
}

impl OrderBook {
    fn change_bid(&mut self, new_bid: i64, new_vol: u64) {
        self.bid = new_bid;
        self.bid_vol = new_vol;
    }
    fn change_ask(&mut self, new_ask: i64, new_vol: u64) {
        self.ask = new_ask;
        self.ask_vol = new_vol;
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Spread {
    pub raw: i64,
    pub percentage: f64,
}

pub struct MultiBook<const S: usize, const T: usize> {
    pub books: [OrderBook; S],
    pub spreads: [Spread; T],
}

impl<const S: usize, const T: usize> MultiBook<S, T> {
    fn change_bid(&mut self, book_idx: usize, new_bid: i64, new_vol: u64) {
        self.books[book_idx].change_bid(new_bid, new_vol);
        for i in 0..S {
            if i != book_idx {
                self.update_spread(book_idx, i);
            }
        }
    }
    fn change_ask(&mut self, book_idx: usize, new_ask: i64, new_vol: u64) {
        self.books[book_idx].change_ask(new_ask, new_vol);
        for i in 0..S {
            if i != book_idx {
                self.update_spread(book_idx, i);
            }
        }
    }
    pub fn update_spread(&mut self, book_idx_1: usize, book_idx_2: usize) {
        let book_1 = &self.books[book_idx_1];
        let book_2 = &self.books[book_idx_2];
        let raw1: i64 = book_2.bid - book_1.ask;
        let raw2: i64 = book_1.bid - book_2.ask;
        let percentage1: f64 = (raw1 as f64) / (book_1.ask as f64);
        let percentage2: f64 = (raw2 as f64) / (book_2.ask as f64);
        self.spreads[book_idx_1 + (S * book_idx_2)] = Spread {
            raw: raw1,
            percentage: percentage1,
        };
        self.spreads[book_idx_2 + (S * book_idx_1)] = Spread {
            raw: raw2,
            percentage: percentage2,
        };
    }
}