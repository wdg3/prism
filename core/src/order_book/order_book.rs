use std::{time::Duration, sync::Arc};
use heapless::{binary_heap::{Max, Min}, Vec};
use tokio::{time::Instant, sync::RwLock};

use super::data_types::{Update, Side, PriceLevel, Snapshot};

#[derive(Default)]
pub struct OrderBook {
    bids: Box<heapless::BinaryHeap<usize, Max, 16384>>,
    asks: Box<heapless::BinaryHeap<usize, Min, 16384>>,
    pub bid_lookup: Box<heapless::FnvIndexMap<usize, PriceLevel, 16384>>,
    pub ask_lookup: Box<heapless::FnvIndexMap<usize, PriceLevel, 16384>>,
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
        OrderBook::init_side(&mut self.bid_lookup, &mut self.bids, &snapshot.bids);
        OrderBook::init_side(&mut self.ask_lookup, &mut self.asks, &snapshot.asks);
        OrderBook::update_best::<Max>(&self.bids, &mut self.best_bid);
        OrderBook::update_best::<Min>(&self.asks, &mut self.best_ask);
        println!("Best bid: {:?}\nBest ask: {:?}", self.best_bid.as_ref().unwrap(), self.best_ask.as_ref().unwrap());
    }
    fn init_side<K>(
        lookup: &mut Box<heapless::FnvIndexMap<usize, PriceLevel, 16384>>,
        heap: &mut Box<heapless::BinaryHeap<usize, K, 16384>>,
        snapshot: &Vec<PriceLevel, 10000>)
    where K: heapless::binary_heap::Kind {
            for price_level in snapshot {
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
        self.validate();

    }
    fn update_lookup(
        lookup: &mut Box<heapless::FnvIndexMap<usize, PriceLevel, 16384>>,
        level: usize,
        amount: f64,
        seq: i64) {
        if amount.to_bits() == (0.0 as f64).to_bits() {
            lookup.remove(&level).unwrap();
        } else {
            let _ = lookup.insert(level, PriceLevel{ level: level, amount: amount, sequence: seq }).unwrap();
        }
    }
    fn update_heap<K>(
        lookup: &Box<heapless::FnvIndexMap<usize, PriceLevel, 16384>>,
        heap: &mut Box<heapless::BinaryHeap<usize, K, 16384>>,
        level: usize,
        amount: f64)
    where K: heapless::binary_heap::Kind {
        while !lookup.contains_key(&heap.peek().unwrap()) {
            let _ = heap.pop().unwrap();
        }
        if heap.len() == 16384 {
            OrderBook::heap_from_lookup(lookup, heap);
        }
        if !(amount.to_bits() == (0.0 as f64).to_bits() && !lookup.contains_key(&level)) {
            let _ = heap.push(level).unwrap();
        }
    }
    fn update_best<K>(
        heap: &Box<heapless::BinaryHeap<usize, K, 16384>>,
        best: &mut Option<usize>)
    where K: heapless::binary_heap::Kind {
        *best = Some(*heap.peek().unwrap());
    }
    fn heap_from_lookup<K>(
        lookup: &Box<heapless::FnvIndexMap<usize, PriceLevel, 16384>>,
        heap: &mut Box<heapless::BinaryHeap<usize, K, 16384>>)
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
    fn validate(&self) {
        let best_bid = &self.best_bid.unwrap();
        let best_ask = &self.best_ask.unwrap();
        let heap_bid = self.bids.peek().unwrap();
        let heap_ask = self.asks.peek().unwrap();
        let bid = self.bid_lookup.get(&best_bid).unwrap();
        let ask = self.ask_lookup.get(&best_ask).unwrap();

        assert!(best_bid < best_ask);
        assert_eq!(best_bid, heap_bid);
        assert_eq!(best_ask, heap_ask);
        assert_eq!(heap_bid, &bid.level);
        assert_eq!(heap_ask, &ask.level);
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Spread {
    pub raw: i64,
    pub percentage: f64,
}

pub struct MultiBook<const S: usize, const T: usize> {
    pub coinbase_book: Arc<RwLock<OrderBook>>,
    pub kraken_book: Arc<RwLock<OrderBook>>,
    pub spreads: heapless::Vec<Spread, T>,
}

impl<const S: usize, const T: usize> MultiBook<S, T> {
    pub fn new() -> Self {
        return MultiBook {
            coinbase_book: Arc::new(RwLock::new(OrderBook::new())),
            kraken_book: Arc::new(RwLock::new(OrderBook::new())),
            spreads: heapless::Vec::new(),
        }
    }

    pub fn update_spread(&mut self, book_idx_1: usize, book_idx_2: usize) {
    }
    pub async fn print(&self) {
        self.print_book(self.coinbase_book.clone(), "Coinbase").await;
        self.print_book(self.kraken_book.clone(), "Kraken").await;
    }
    async fn print_book(&self, book: Arc<RwLock<OrderBook>>, name: &str) {
        let guard = book.read().await;
        let best_bid = guard.best_bid.as_ref();
        let best_ask = guard.best_ask.as_ref();
        if best_bid.is_some() && best_ask.is_some() {
            let bid = guard.bid_lookup.get(best_bid.unwrap());
            let ask = guard.ask_lookup.get(best_ask.unwrap());
            println!("{:?} best bid: {:?}\n{:?} best ask: {:?}", name, bid, name, ask);
        }
    }
}