use std::time::Duration;
use heapless::{binary_heap::{Max, Min}, Vec};
use tokio::time::Instant;

use super::data_types::{Update, Side, PriceLevel, Snapshot};

#[derive(Default)]
pub struct OrderBook {
    pub name: heapless::String<8>,
    pub pair: heapless::String<8>,
    pub bids: Box<heapless::BinaryHeap<usize, Max, 65536>>,
    pub asks: Box<heapless::BinaryHeap<usize, Min, 65536>>,
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

