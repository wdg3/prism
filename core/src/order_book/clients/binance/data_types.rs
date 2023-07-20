use serde::Deserialize;

#[derive(Debug, Deserialize, PartialEq)]
pub struct InboundMessage {
    pub message_type: heapless::String<8>,
    pub pair: heapless::String<8>,
    pub sent: Option<i64>,
    pub price: Option<heapless::String<16>>,
    pub amount: Option<heapless::String<16>>,
    pub bid_level: Option<heapless::String<16>>,
    pub ask_level: Option<heapless::String<16>>,
    pub bid_amount: Option<heapless::String<16>>,
    pub ask_amount: Option<heapless::String<16>>,
    pub buy: Option<bool>,
}

#[derive(Debug, PartialEq)]
pub struct Update {
    pub best_bid: Change,
    pub best_ask: Change,
}

#[derive(Debug, PartialEq)]
pub struct Change {
    pub side: Side,
    pub level: PriceLevel,
}

#[derive(Debug, PartialEq)]
pub enum Side {
    Buy,
    Sell,
}

#[derive(Debug, PartialEq)]
pub struct PriceLevel {
    pub level: usize,
    pub amount: f64,
}