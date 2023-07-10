#[derive(Debug, PartialEq)]
pub struct Snapshot {
    pub bids: Box<heapless::Vec<PriceLevel, 10000>>,
    pub asks: Box<heapless::Vec<PriceLevel, 10000>>,
}

#[derive(Copy, Clone, Default, Debug, PartialEq)]
pub struct PriceLevel {
    pub level: usize,
    pub amount: f64,
    pub sequence: i64,
}

#[derive(Debug, PartialEq)]
pub struct Update<'a> {
    pub product_id: &'a str,
    pub time: &'a str,
    pub changes: heapless::Vec<Change, 32>
}

#[derive(Debug, PartialEq)]
pub struct Change {
    pub side: Side,
    pub price_level: PriceLevel,
}

#[derive(Debug, PartialEq)]
pub enum Side {
    Buy,
    Sell,
}