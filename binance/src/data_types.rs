use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, PartialEq)]
pub struct Message {
    #[serde(rename = "e")]
    pub event_type: Option<heapless::String<16>>,
    #[serde(rename = "s")]
    pub pair: Option<heapless::String<8>>,
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct Trade {
    #[serde(rename = "E")]
    pub sent: i64,
    #[serde(rename = "s")]
    pub pair: heapless::String<8>,
    #[serde(rename = "p")]
    pub price: heapless::String<16>,
    #[serde(rename = "q")]
    pub amount: heapless::String<16>,
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct BookUpdate {
    #[serde(rename = "s")]
    pub pair: heapless::String<8>,
    #[serde(rename = "b")]
    pub bid_level: heapless::String<16>,
    #[serde(rename = "a")]
    pub ask_level: heapless::String<16>,
    #[serde(rename = "B")]
    pub bid_amount: heapless::String<16>,
    #[serde(rename = "A")]
    pub ask_amount: heapless::String<16>,
}

#[derive(Debug, Serialize, PartialEq)]
pub struct OutboundMessage {
    pub message_type: heapless::String<8>,
    pub pair: heapless::String<8>,
    pub sent: Option<i64>,
    pub price: Option<heapless::String<16>>,
    pub amount: Option<heapless::String<16>>,
    pub bid_level: Option<heapless::String<16>>,
    pub ask_level: Option<heapless::String<16>>,
    pub bid_amount: Option<heapless::String<16>>,
    pub ask_amount: Option<heapless::String<16>>,
}
