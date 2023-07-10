use chrono::{DateTime, TimeZone, Utc};
use serde::{Deserialize, Deserializer, de::{Visitor, SeqAccess}};
use serde::de::Error;

#[derive(Debug)]
pub struct Message {
    channel_id: usize,
    content: Content,
}

#[derive(Deserialize, Debug, PartialEq)]
#[serde(untagged)]
pub enum Content {
    Snapshot(Snapshot),
    Update(Update),
}

#[derive(Deserialize, Debug, PartialEq)]
pub struct Snapshot {
    //#[serde(rename = "as")]
    asks: heapless::Vec<PriceLevel, 100>,
    //#[serde(rename = "bs")]
    bids: heapless::Vec<PriceLevel, 100>,
}

#[derive(Deserialize, Debug, PartialEq)]
pub struct Update {
    a: heapless::Vec<PriceLevel, 100>,
    b: heapless::Vec<PriceLevel, 100>,
    c: usize,
}

#[derive(Copy, Clone, Default, Debug, PartialEq)]
pub struct PriceLevel {
    pub level: usize,
    pub amount: f64,
    pub timestamp: DateTime<Utc>,
    pub sequence: i64,
}

impl<'de> Deserialize<'de> for Message {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_seq(MessageVisitor)
    }
}

struct MessageVisitor;

impl<'de> Visitor<'de> for MessageVisitor {
    type Value = Message;

    fn expecting(&self, formatter: &mut core::fmt::Formatter) -> core::fmt::Result {
        formatter.write_str("A Kraken L2 order book update")
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
        A::Error: serde::de::Error,
    {  
        let channel_id = seq.next_element().unwrap().unwrap();
        println!("{:?}", channel_id);
        let res = seq.next_element::<Content>();
        println!("{:?}", res);
        let c = res.unwrap().unwrap();
        let _ = seq.next_element::<&str>();
        let _ = seq.next_element::<&str>();
        Ok(Message { channel_id: channel_id, content: c })
    }
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
        let timestamp_float = seq.next_element::<&str>().unwrap().unwrap().parse::<f64>().unwrap() * 1000000 as f64;
        let timestamp = Utc.timestamp_nanos((timestamp_float * (1000 as f64)) as i64);
        Ok(PriceLevel {
            level: level,
            amount: amount,
            timestamp: timestamp,
            sequence: 0,
            })
    }
}

#[test]
fn test_message() {
    let input = r#"
[
  0,
  {
    "asks": [
      [
        "5541.30000",
        "2.50700000",
        "1534614248.123678"
      ],
      [
        "5541.80000",
        "0.33000000",
        "1534614098.345543"
      ],
      [
        "5542.70000",
        "0.64700000",
        "1534614244.654432"
      ]
    ],
    "bids": [
      [
        "5541.20000",
        "1.52900000",
        "1534614248.765567"
      ],
      [
        "5539.90000",
        "0.30000000",
        "1534614241.769870"
      ],
      [
        "5539.50000",
        "5.00000000",
        "1534613831.243486"
      ]
    ]
  },
  "book-100",
  "XBT/USD"
]"#;
    let result = serde_json_core::from_str::<Message>(input);
    match &result {
        Err(e) => println!("{:?}", e.to_string()),
        Ok(r) => println!("{:?}", r),
    }
    assert!(result.is_ok());

}

#[test]
fn test_content() {
    let input = r#"
  {
    "asks": [
      [
        "5541.30000",
        "2.50700000",
        "1534614248.123678"
      ],
      [
        "5541.80000",
        "0.33000000",
        "1534614098.345543"
      ],
      [
        "5542.70000",
        "0.64700000",
        "1534614244.654432"
      ]
    ],
    "bids": [
      [
        "5541.20000",
        "1.52900000",
        "1534614248.765567"
      ],
      [
        "5539.90000",
        "0.30000000",
        "1534614241.769870"
      ],
      [
        "5539.50000",
        "5.00000000",
        "1534613831.243486"
      ]
    ]
  }
"#;
    let result = serde_json_core::from_str::<Content>(input);
    match &result {
        Err(e) => println!("{:?}", e.to_string()),
        Ok(r) => println!("{:?}", r),
    }
    assert!(result.is_ok());

}

#[test]
fn test_price_level() {
    let input = r#"
    [
        "5539.50000",
        "5.00000000",
        "1534613831.243486"
    ]"#;
    let result = serde_json_core::from_str::<PriceLevel>(input);
    match &result {
        Err(e) => println!("{:?}", e.to_string()),
        Ok(r) => println!("{:?}", r),
    }
    assert!(result.is_ok());
}