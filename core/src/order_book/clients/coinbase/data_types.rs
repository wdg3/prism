use serde::{Deserialize, Deserializer, de::{Visitor, SeqAccess}};

#[derive(Deserialize, Debug, PartialEq)]
pub struct Message<'a> {
    #[serde(rename = "type")]
    pub msg_type: &'a str,
    pub time: &'a str,
}

#[derive(Deserialize, Debug, PartialEq)]
pub struct Snapshot {
    pub bids: Box<heapless::Vec<PriceLevel, 65536>>,
    pub asks: Box<heapless::Vec<PriceLevel, 65536>>,
}

#[derive(Deserialize, Debug, PartialEq)]
pub struct Update {
    pub changes: heapless::Vec<Change, 32>
}

#[derive(Copy, Clone, Default, Debug, PartialEq)]
pub struct PriceLevel {
    pub level: usize,
    pub amount: f64,
    pub sequence: i64,
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

#[derive(Debug, Deserialize, PartialEq)]
pub struct Match {
    pub side: Side,
    #[serde(deserialize_with="match_amount")]
    pub size: f64,
    #[serde(deserialize_with="match_price")]
    pub price: usize,
}

pub fn match_price<'de, D>(deserializer: D) -> Result<usize, D::Error>
where D: Deserializer<'de> {
    let input = heapless::String::<16>::deserialize(deserializer).unwrap();
    return Ok((input.parse::<f64>().unwrap() * 100.0) as usize)
}

pub fn match_amount<'de, D>(deserializer: D) -> Result<f64, D::Error>
where D: Deserializer<'de> {
    let input = heapless::String::<16>::deserialize(deserializer).unwrap();
    return Ok(input.parse::<f64>().unwrap())
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
        formatter.write_str("A Coinbase L2 order book update")
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
            sequence: 0,
            })
    }
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
                sequence: 0,
            },
        })
    }
}

#[test]
fn test_message() {

}

#[test]
fn test_snapshot() {
    let input = r#"
    {
        "type": "snapshot",
        "product_id": "ETH-USD",
        "bids": [["10.01", "1100.0"],["11.07", "1110.01"]],
        "asks": [["12.23", "2.3"],["13.13", "13.2"]]
    }"#;
    let result = serde_json_core::from_str::<Snapshot>(input);
    println!("{:?}", result);
    assert!(result.is_ok());
}

#[test]
fn test_update() {

}

#[test]
fn test_price_level() {
    let input = r#"
        ["10.01", "1100.0"]
    "#;
    let result = serde_json_core::from_str::<PriceLevel>(input);
    match &result {
        Err(e) => println!("{:?}", e.to_string()),
        Ok(r) => println!("{:?}", r),
    }
    println!("{:?}", result);
    assert!(result.is_ok());
}

#[test]
fn test_change() {
    let input = r#"
        ["buy", "10.01", "1100.0"]
    "#;
    let result = serde_json_core::from_str::<Change>(input);
    println!("{:?}", result);
    assert!(result.is_ok());
    let (unwrapped, _) = result.unwrap();
    assert_eq!(Side::Buy, unwrapped.side);
    assert_eq!(1001, unwrapped.price_level.level);
    assert_eq!(1100.0, unwrapped.price_level.amount);
}