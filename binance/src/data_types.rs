use serde::{Deserialize, Deserializer};

#[derive(Debug, Deserialize, PartialEq)]
pub struct Trade {
    #[serde(rename = "E")]
    pub sent: Option<i64>,
    #[serde(rename = "s")]
    pub pair: heapless::String<8>,
    #[serde(rename = "b")]
    pub bid_level: Option<heapless::String<16>>,
    #[serde(rename = "a")]
    pub ask_level: Option<heapless::String<16>>,
    #[serde(rename = "B")]
    pub bid_amount: Option<heapless::String<16>>,
    #[serde(rename = "A")]
    pub ask_amount: Option<heapless::String<16>>,
    #[serde(rename = "p")]
    pub price: Option<heapless::String<16>>,
    #[serde(rename = "q")]
    pub amount: Option<heapless::String<16>>,
}