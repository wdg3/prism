use std::{mem::size_of, rc::Rc, cell::{RefCell, Ref, Cell}, sync::{Arc, RwLock}};

use arc_swap::ArcSwap;
use heapless::{binary_heap::{Max, Min}};
use tokio::{time::Instant, sync::{Mutex, MutexGuard, RwLockReadGuard}};

use super::clients::coinbase::data_types::{PriceLevel, Snapshot, Update, Side};

#[derive(Default)]
pub struct OrderBook {
    bids: Box<heapless::BinaryHeap<RefCell<PriceLevel>, Max, 16384>>,
    asks: Box<heapless::BinaryHeap<RefCell<PriceLevel>, Min, 16384>>,
    pub bid_lookup: Box<heapless::FnvIndexMap<usize, RefCell<PriceLevel>, 16384>>,
    pub ask_lookup: Box<heapless::FnvIndexMap<usize, RefCell<PriceLevel>, 16384>>,
    best_bid: Option<RefCell<PriceLevel>>,
    best_ask: Option<RefCell<PriceLevel>>,
}

#[derive(Debug)]
struct SafePriceLevel<'a>(&'a RwLock<PriceLevel>);
impl<'a> Ord for SafePriceLevel<'a> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0.read().unwrap().level.cmp(&other.0.read().unwrap().level)
    }
}
impl<'a> PartialOrd for SafePriceLevel<'a> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.0.read().unwrap().level.partial_cmp(&other.0.read().unwrap().level)
    }
}
impl<'a> PartialEq for SafePriceLevel<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.0.read().unwrap().level == other.0.read().unwrap().level && self.0.read().unwrap().amount == other.0.read().unwrap().amount
    }
}
impl<'a> Eq for SafePriceLevel<'a> {}

impl Eq for PriceLevel {}

impl PartialOrd for PriceLevel {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.level.partial_cmp(&other.level)
    }
}

impl Ord for PriceLevel {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.level.cmp(&other.level)
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
        }
    }
    pub fn init(&mut self, snapshot: Snapshot) {
        for bid in snapshot.bids {
            let bid_cell = RefCell::new(bid);
            let _ = self.bids.push(RefCell::clone(&bid_cell));
            match &self.best_bid {
                Option::None => {
                    self.best_bid = Some(RefCell::clone(&bid_cell));
                }
                Some(p) => {
                    if p.borrow().level <= bid.level {
                        self.best_bid = Some(RefCell::clone(&bid_cell));
                    }
                }
            }
            let _ = self.bid_lookup.insert(bid.level, bid_cell);
        }
        for ask in snapshot.asks {
            let ask_cell = RefCell::new(ask);
            let _ = self.asks.push(RefCell::clone(&ask_cell));
            match &self.best_ask {
                Option::None => {
                    self.best_ask = Some(RefCell::clone(&ask_cell));
                }
                Some(p) => {
                    if p.borrow().level >= ask.level {
                        self.best_ask = Some(RefCell::clone(&ask_cell));
                    }
                }
            }
            let _ = self.ask_lookup.insert(ask.level, ask_cell);
        }
        println!("Best bid: {:?}\nBest ask: {:?}", self.best_bid.as_ref().unwrap().borrow(), self.best_ask.as_ref().unwrap().borrow());
    }
    pub fn update(&mut self, update: Update) {
        let start = Instant::now();
        for change in update.changes {
            let (level, amount) = (change.price_level.level, change.price_level.amount);
            match change.side {
                Side::Buy => {
                    OrderBook::update_lookup(&mut self.bid_lookup, level, amount);
                    OrderBook::update_heap::<Max>(&self.bid_lookup, &mut self.bids, level, amount);
                    OrderBook::update_best::<Max>(&self.bids, &mut self.best_bid);
                },
                Side::Sell => {
                    OrderBook::update_lookup(&mut self.ask_lookup, level, amount);
                    OrderBook::update_heap::<Min>(&self.ask_lookup, &mut self.asks, level, amount);
                    OrderBook::update_best::<Min>(&self.asks, &mut self.best_ask);
                },
            }
        }
        let duration = start.elapsed();
        println!("Order book updated in {:?}", duration);
        println!("Best bid: {:?}\nBest ask: {:?}", self.best_bid.as_ref().unwrap().borrow(), self.best_ask.as_ref().unwrap().borrow());
        self.validate();

    }
    fn update_lookup(
        lookup: &mut Box<heapless::FnvIndexMap<usize, RefCell<PriceLevel>, 16384>>,
        level: usize,
        amount: f64) {
        if amount.to_bits() == (0.0 as f64).to_bits() {
            lookup.remove(&level);
        } else if lookup.contains_key(&level) {
            lookup.get(&level).unwrap().borrow_mut().amount = amount;
            assert_eq!(amount, lookup.get(&level).unwrap().borrow().amount);
        } else {
            let _ = lookup.insert(level, RefCell::new(PriceLevel {level: level, amount: amount}));
            assert_eq!(amount, lookup.get(&level).unwrap().borrow().amount);
        }
    }
    fn update_heap<K>(
        lookup: &Box<heapless::FnvIndexMap<usize, RefCell<PriceLevel>, 16384>>,
        heap: &mut Box<heapless::BinaryHeap<RefCell<PriceLevel>, K, 16384>>,
        level: usize,
        amount: f64)
    where K: heapless::binary_heap::Kind {
        while !lookup.contains_key(&heap.peek().unwrap().borrow().level) {
            let _ = heap.pop();
        }
        if !(amount.to_bits() == (0.0 as f64).to_bits()) {
            let _ = heap.push(RefCell::clone(lookup.get(&level).unwrap())).unwrap();
        }
        let top_level = heap.peek().unwrap().borrow().level;
        heap.peek().unwrap().borrow_mut().amount = lookup.get(&top_level).unwrap().borrow().amount;
    }
    fn update_best<K>(
        heap: &Box<heapless::BinaryHeap<RefCell<PriceLevel>, K, 16384>>,
        best: &mut Option<RefCell<PriceLevel>>)
    where K: heapless::binary_heap::Kind {
        best.as_mut().unwrap().borrow_mut().amount = heap.peek().unwrap().borrow().amount;
        best.as_mut().unwrap().borrow_mut().level = heap.peek().unwrap().borrow().level;
    }
    fn validate(&self) {
        let best_bid = self.best_bid.as_ref().unwrap().borrow();
        let best_ask = self.best_ask.as_ref().unwrap().borrow();
        let heap_bid = self.bids.peek().unwrap().borrow();
        let heap_ask = self.asks.peek().unwrap().borrow();
        let bid = self.bid_lookup.get(&best_bid.level).unwrap().borrow();
        let ask = self.ask_lookup.get(&best_ask.level).unwrap().borrow();

        assert!(best_bid.level < best_ask.level);
        assert_eq!(best_bid.level, heap_bid.level);
        assert_eq!(best_ask.level, heap_ask.level);
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