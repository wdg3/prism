use std::borrow::BorrowMut;

use heapless::{binary_heap::{Max, Min}};
use tokio::{time::Instant};

use super::clients::coinbase::data_types::{PriceLevel, Snapshot, Update, Side};

#[derive(Default)]
pub struct OrderBook {
    bids: Box<heapless::BinaryHeap<PriceLevel, Max, 16384>>,
    asks: Box<heapless::BinaryHeap<PriceLevel, Min, 16384>>,
    pub bid_lookup: Box<heapless::FnvIndexMap<usize, PriceLevel, 16384>>,
    pub ask_lookup: Box<heapless::FnvIndexMap<usize, PriceLevel, 16384>>,
    best_bid: Option<PriceLevel>,
    best_ask: Option<PriceLevel>,
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

impl<'a> OrderBook {
    pub fn new() -> Self {
        return OrderBook {
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
        for bid in snapshot.bids {
            let _ = self.bids.push(bid);
            match self.best_bid {
                Option::None => {
                    self.best_bid = Some(bid);
                }
                Some(p) => {
                    if p.level <= bid.level {
                        self.best_bid = Some(bid);
                    }
                }
            }
            let _ = self.bid_lookup.insert(bid.level, bid);
        }
        for ask in snapshot.asks {
            let _ = self.asks.push(ask);
            match self.best_ask {
                Option::None => {
                    self.best_ask = Some(ask);
                }
                Some(p) => {
                    if p.level >= ask.level {
                        self.best_ask = Some(ask);
                    }
                }
            }
            let _ = self.ask_lookup.insert(ask.level, ask);
        }
        println!("Best bid: {:?}\nBest ask: {:?}", self.best_bid.as_ref().unwrap(), self.best_ask.as_ref().unwrap());
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
        let duration = start.elapsed();
        self.num_updates += 1;
        self.average_update = self.average_update + ((duration.as_nanos() as f64) / (self.num_updates as f64 * 1000.0));
        println!("Order book updated in {:?}", duration);
        println!("Average update time: {:?} microseconds", self.average_update);
        println!("Best bid: {:?}\nBest ask: {:?}", self.best_bid.as_ref().unwrap(), self.best_ask.as_ref().unwrap());
        println!("Bid lookup size: {:?}, ask lookup size: {:?}", self.bid_lookup.len(), self.ask_lookup.len());
        println!("Bid heap size: {:?}, ask heap size: {:?}", self.bids.len(), self.asks.len());
        let start = Instant::now();
        let _ = OrderBook::heap_from_lookup::<Max>(&self.bid_lookup, &mut self.bids);
        let duration = start.elapsed();
        println!("Bid heap reset time: {:?}", duration);
        let start = Instant::now();
        let _ = OrderBook::heap_from_lookup::<Min>(&self.ask_lookup, &mut self.asks);
        let duration = start.elapsed();
        println!("Ask heap reset time: {:?}", duration);
        self.validate();

    }
    fn update_lookup(
        lookup: &mut Box<heapless::FnvIndexMap<usize, PriceLevel, 16384>>,
        level: usize,
        amount: f64,
        seq: i64) {
        if amount.to_bits() == (0.0 as f64).to_bits() {
            lookup.remove(&level);
        } else {
            let _ = lookup.insert(level, PriceLevel {level: level, amount: amount, sequence: seq});
        }
    }
    fn update_heap<K>(
        lookup: &Box<heapless::FnvIndexMap<usize, PriceLevel, 16384>>,
        heap: &mut Box<heapless::BinaryHeap<PriceLevel, K, 16384>>,
        level: usize,
        amount: f64)
    where K: heapless::binary_heap::Kind {
        while !lookup.contains_key(&heap.peek().unwrap().level) || heap.peek().unwrap().level == level {
            let _ = heap.pop();
        }
        if !(amount.to_bits() == (0.0 as f64).to_bits()) {
            let _ = heap.push(*lookup.get(&level).unwrap());
        }
    }
    fn update_best<K>(
        heap: &Box<heapless::BinaryHeap<PriceLevel, K, 16384>>,
        best: &mut Option<PriceLevel>)
    where K: heapless::binary_heap::Kind {
        *best = Some(*heap.peek().unwrap());
        //best.as_mut().unwrap().amount = heap.peek().unwrap().amount;
        //best.as_mut().unwrap().level = heap.peek().unwrap().level;
        //best.as_mut().unwrap().sequence = heap.peek().unwrap().sequence;
    }
    fn heap_from_lookup<K>(
        lookup: &Box<heapless::FnvIndexMap<usize, PriceLevel, 16384>>,
        heap: &mut Box<heapless::BinaryHeap<PriceLevel, K, 16384>>)
    where K: heapless::binary_heap::Kind {
        heap.clear();
        lookup.values().for_each(|v| {
            let _ = heap.push(*v);
        });
    }
    fn validate(&self) {
        let best_bid = self.best_bid.unwrap();
        let best_ask = self.best_ask.unwrap();
        let heap_bid = self.bids.peek().unwrap();
        let heap_ask = self.asks.peek().unwrap();
        let bid = self.bid_lookup.get(&best_bid.level).unwrap();
        let ask = self.ask_lookup.get(&best_ask.level).unwrap();

        assert!(best_bid.level < best_ask.level);
        assert_eq!(best_bid.level, heap_bid.level);
        assert_eq!(best_ask.level, heap_ask.level);
        assert_eq!(heap_bid, bid);
        assert_eq!(heap_ask, ask);
        assert_eq!(heap_bid.level, bid.level);
        assert_eq!(heap_ask.level, ask.level);
        assert_eq!(heap_bid.amount, bid.amount);
        assert_eq!(heap_ask.amount, ask.amount);
        assert_eq!(best_bid.amount, bid.amount);
        assert_eq!(best_ask.amount, ask.amount);
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

impl<'a, const S: usize, const T: usize> MultiBook<S, T> {
    fn change_bid(&mut self, book_idx: usize, new_bid: i64, new_vol: u64) {
    }
    fn change_ask(&mut self, book_idx: usize, new_ask: i64, new_vol: u64) {
    }
    pub fn update_spread(&mut self, book_idx_1: usize, book_idx_2: usize) {
    }
}