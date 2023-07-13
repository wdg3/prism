use serde::{de::{Visitor, SeqAccess}, Deserializer, Deserialize};

#[derive(Debug, Deserialize)]
pub struct Message {
    pub data: Update,
}

#[derive(Debug, Deserialize)]
pub struct Update {
    pub bids: heapless::Vec<PriceLevel, 128>,
    pub asks: heapless::Vec<PriceLevel, 128>,
}

#[derive(Debug)]
pub struct PriceLevel {
    pub level: usize,
    pub amount: f64,
}

impl<'de> Deserialize<'de> for PriceLevel {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_seq(PriceLevelVisitor)
    }
}

struct PriceLevelVisitor;

impl<'de> Visitor<'de> for PriceLevelVisitor {
    type Value = PriceLevel;

    fn expecting(&self, formatter: &mut core::fmt::Formatter) -> core::fmt::Result {
        formatter.write_str("A Bitstamp L2 order book update")
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let level = (seq.next_element::<&str>().unwrap().unwrap().parse::<f64>().unwrap() * 100.) as usize;
        let amount = seq.next_element::<&str>().unwrap().unwrap().parse::<f64>().unwrap();
        Ok(PriceLevel {
            level: level,
            amount: amount,
        })
    }
}