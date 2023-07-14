use std::time::Duration;
use chrono::Local;
use heapless::{binary_heap::{Max, Min}, Vec};
use tokio::time::Instant;

use super::data_types::{Update, Side, PriceLevel, Snapshot};

#[derive(Default)]
pub struct OrderBook {
    name: heapless::String<8>,
    pair: heapless::String<8>,
    bids: Box<heapless::BinaryHeap<usize, Max, 65536>>,
    asks: Box<heapless::BinaryHeap<usize, Min, 65536>>,
    pub bid_lookup: Box<heapless::FnvIndexMap<usize, PriceLevel, 65536>>,
    pub ask_lookup: Box<heapless::FnvIndexMap<usize, PriceLevel, 65536>>,
    pub best_bid: Option<usize>,
    pub best_ask: Option<usize>,
    average_update: f64,
    num_updates: usize,
    count: i64,
}

impl Eq for PriceLevel {}

impl PartialOrd for PriceLevel {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        if self.level != other.level {
            self.level.partial_cmp(&other.level)
        } else {
            self.sequence.partial_cmp(&other.sequence)
        }
    }
}

impl Ord for PriceLevel {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        if self.level != other.level {
            self.level.cmp(&other.level)
        } else {
            self.sequence.cmp(&other.sequence)
        }
    }
}

impl OrderBook {
    pub fn new(name: heapless::String<8>, pair: heapless::String<8>) -> Self {
        return OrderBook {
            name: name,
            pair: pair,
            bids: Box::new(heapless::BinaryHeap::new()),
            asks: Box::new(heapless::BinaryHeap::new()),
            bid_lookup: Box::new(heapless::FnvIndexMap::new()),
            ask_lookup: Box::new(heapless::FnvIndexMap::new()),
            best_bid: Option::None,
            best_ask: Option::None,
            average_update: 0.0,
            num_updates: 0,
            count: 0,
        }
    }
    pub fn init(&mut self, snapshot: Snapshot) {
        OrderBook::init_side(&mut self.bid_lookup, &mut self.bids, &mut self.best_bid, &snapshot.bids);
        OrderBook::init_side(&mut self.ask_lookup, &mut self.asks, &mut self.best_ask, &snapshot.asks);
        OrderBook::update_best::<Max>(&self.bids, &mut self.best_bid);
        OrderBook::update_best::<Min>(&self.asks, &mut self.best_ask);
    }
    fn init_side<K>(
        lookup: &mut Box<heapless::FnvIndexMap<usize, PriceLevel, 65536>>,
        heap: &mut Box<heapless::BinaryHeap<usize, K, 65536>>,
        best: &mut Option<usize>,
        snapshot: &Vec<PriceLevel, 65536>)
    where K: heapless::binary_heap::Kind {
            for price_level in snapshot {
                lookup.clear();
                heap.clear();
                *best = None;
                let level: usize = price_level.level;
                let _ = lookup.insert(level, price_level.clone()).unwrap();
                let _ = heap.push(level).unwrap();
            }
        }
    pub fn update(&mut self, update: Update) {
        let start = Instant::now();
        self.count = self.count + 1;
        for change in update.changes {
            let (level, amount) = (change.price_level.level, change.price_level.amount);
            match change.side {
                Side::Buy => {
                    OrderBook::update_lookup(&mut self.bid_lookup, level, amount, self.count);
                    OrderBook::update_heap::<Max>(&self.bid_lookup, &mut self.bids, level, amount);
                    OrderBook::update_best::<Max>(&self.bids, &mut self.best_bid);
                },
                Side::Sell => {
                    OrderBook::update_lookup(&mut self.ask_lookup, level, amount, -1 * self.count);
                    OrderBook::update_heap::<Min>(&self.ask_lookup, &mut self.asks, level, amount);
                    OrderBook::update_best::<Min>(&self.asks, &mut self.best_ask);
                },
            }
        }
        //self.print(&start);
        if self.best_bid.is_some() && self.best_ask.is_some() {
            self.validate();
        }

    }
    fn update_lookup(
        lookup: &mut Box<heapless::FnvIndexMap<usize, PriceLevel, 65536>>,
        level: usize,
        amount: f64,
        seq: i64) {
        if amount.to_bits() == (0.0 as f64).to_bits() && lookup.contains_key(&level){
            lookup.remove(&level).unwrap();                                                                                                                                                     
        } else if amount.to_bits() != (0.0 as f64).to_bits() {
            let _ = lookup.insert(level, PriceLevel{ level: level, amount: amount, sequence: seq }).unwrap();
        }
    }
    fn update_heap<K>(
        lookup: &Box<heapless::FnvIndexMap<usize, PriceLevel, 65536>>,
        heap: &mut Box<heapless::BinaryHeap<usize, K, 65536>>,
        level: usize,
        amount: f64)
    where K: heapless::binary_heap::Kind {
        if heap.len() >= 65536 {
            OrderBook::heap_from_lookup(lookup, heap);
        }
        while heap.len() > 0 && !lookup.contains_key(&heap.peek().unwrap()) {
            let _ = heap.pop().unwrap();
        }
        if !(amount.to_bits() == (0.0 as f64).to_bits()) {
            let _ = heap.push(level).unwrap();
        }
    }
    fn update_best<K>(
        heap: &Box<heapless::BinaryHeap<usize, K, 65536>>,
        best: &mut Option<usize>)
    where K: heapless::binary_heap::Kind {
        match heap.peek() {
            None => *best = None,
            Some(b) => *best = Some(*b),
        }
        //*best = Some(*heap.peek().unwrap());
    }
    fn heap_from_lookup<K>(
        lookup: &Box<heapless::FnvIndexMap<usize, PriceLevel, 65536>>,
        heap: &mut Box<heapless::BinaryHeap<usize, K, 65536>>)
    where K: heapless::binary_heap::Kind {
        heap.clear();
        lookup.values().for_each(|v| {
            let _ = heap.push(v.level);
        });
    }
    fn print(&mut self, start: &Instant) {
        let duration = start.elapsed();
        self.num_updates += 1;
        self.average_update = ((self.average_update * ((self.num_updates - 1) as f64)) + (duration.as_nanos() as f64)) / (self.num_updates as f64);
        println!("Order book updated in {:?}", duration);
        println!("Average update time: {:?}", Duration::new(0, self.average_update as u32));
        println!("Best bid: {:?}\nBest ask: {:?}", self.bid_lookup.get(self.best_bid.as_ref().unwrap()), self.ask_lookup.get(self.best_ask.as_ref().unwrap()));
        println!("Bid lookup size: {:?}, ask lookup size: {:?}", self.bid_lookup.len(), self.ask_lookup.len());
        println!("Bid heap size: {:?}, ask heap size: {:?}", self.bids.len(), self.asks.len());
    }
    fn validate(&mut self) {
        let best_bid = &self.best_bid.unwrap();
        let best_ask = &self.best_ask.unwrap();
        let heap_bid = self.bids.peek().unwrap();
        let heap_ask = self.asks.peek().unwrap();
        let bid = self.bid_lookup.get(&best_bid).unwrap();
        let ask = self.ask_lookup.get(&best_ask).unwrap();

        if best_bid >= best_ask || best_bid != heap_bid || best_ask != heap_ask || heap_bid != &bid.level || heap_ask != &ask.level {
            self.print(&Instant::now());
            panic!("{:?}/{:?}: invalid order book state", self.name, self.pair);
        }
    }
}

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
        for i in 0..S {
            if i != book_idx {
                let forward_buy = self.get_best(Side::Sell, &self.books[book_idx]);
                let forward_sell = self.get_best(Side::Buy, &self.books[i]);
                let reverse_buy = self.get_best(Side::Sell, &self.books[i]);
                let reverse_sell = self.get_best(Side::Buy, &self.books[book_idx]);
                if forward_buy.is_some() && forward_sell.is_some() {
                    let spread = self.spread_from_levels(
                        forward_buy.unwrap().0 as isize, 
                        forward_sell.unwrap().0 as isize,
                        [forward_buy.unwrap().1, forward_sell.unwrap().1]);
                    let mut spread_idx = (book_idx * S) + i;
                    if i < book_idx {
                        spread_idx -= book_idx;
                    } else {
                        spread_idx -= book_idx + 1;
                    }
                    self.spreads[spread_idx] = spread;
                }
                if reverse_buy.is_some() && reverse_sell.is_some() {
                    let spread = self.spread_from_levels(
                        reverse_buy.unwrap().0 as isize, 
                        reverse_sell.unwrap().0 as isize,
                        [reverse_buy.unwrap().1, reverse_sell.unwrap().1]);
                    let mut spread_idx = (i * S) + book_idx;
                    if book_idx < i {
                        spread_idx -= i;
                    } else {
                        spread_idx -= i + 1;
                    }
                    self.spreads[spread_idx] = spread;
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