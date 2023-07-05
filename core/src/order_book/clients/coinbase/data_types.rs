use serde::{Deserialize, Deserializer, de::{Visitor, SeqAccess}};

#[derive(Deserialize)]
pub struct Snapshot {
    product_id: String,
    pub bids: Vec<PriceLevel>,
    pub asks: Vec<PriceLevel>,
}

#[derive(Deserialize)]
pub struct Update {
    product_id: String,
    pub time: String,
    pub changes: Vec<Change>
}

#[derive(Deserialize)]
pub struct PriceLevel {
    #[serde(deserialize_with = "str_to_price")]
    level: usize,
    #[serde(deserialize_with = "str_to_amount")]
    amount: f64,
}

pub struct Change {
    side: String,
    price_level: PriceLevel,
}

fn str_to_price<'de, D>(deserializer: D) -> Result<usize, D::Error>
where
    D: Deserializer<'de>,
{
    let s: String = Deserialize::deserialize(deserializer)?;
    Ok((s.parse::<f64>().unwrap() * 100.) as usize)
}

fn str_to_amount<'de, D>(deserializer: D) -> Result<f64, D::Error>
where
    D: Deserializer<'de>,
{
    let s: String = Deserialize::deserialize(deserializer)?;
    Ok(s.parse::<f64>().unwrap())
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
        formatter.write_str("A Coinbase L2 order book update")
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
            },
        })
    }
}