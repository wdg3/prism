use serde::{Deserialize, Deserializer, de::{Visitor, SeqAccess}};

#[derive(Deserialize, Debug, PartialEq)]
pub struct Content {
    pub changes: heapless::Vec<Change, 5000>,
}

#[derive(Debug, PartialEq)]
pub struct Change {
    pub side: Side,
    pub price_level: PriceLevel,
}

#[derive(Deserialize, Debug, PartialEq)]
#[serde(rename_all = "lowercase")] 
pub enum Side {
    Buy,
    Sell,
}

#[derive(Copy, Clone, Default, Debug, PartialEq)]
pub struct PriceLevel {
    pub level: usize,
    pub amount: f64,
    pub sequence: i64,
}

impl<'de> Deserialize<'de> for Change {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_seq(ChangeVisitor)
    }
}

struct ChangeVisitor;

impl<'de> Visitor<'de> for ChangeVisitor {
    type Value = Change;

    fn expecting(&self, formatter: &mut core::fmt::Formatter) -> core::fmt::Result {
        formatter.write_str("A Gemini L2 order book update")
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let side = seq.next_element().unwrap().unwrap();
        let level = (seq.next_element::<&str>().unwrap().unwrap().parse::<f64>().unwrap() * 100.) as usize;
        let amount = seq.next_element::<&str>().unwrap().unwrap().parse::<f64>().unwrap();
        Ok(Change {
            side: side,
            price_level: PriceLevel {
                level: level,
                amount: amount,
                sequence: 0,
            },
        })
    }
}