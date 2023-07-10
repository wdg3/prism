use chrono::{DateTime, TimeZone, Utc};
use serde::{Deserialize, Deserializer, de::{Visitor, SeqAccess}};

#[derive(Debug)]
pub struct Message {
    pub content: Content,
}

#[derive(Deserialize, Debug, PartialEq)]
pub struct Content {
    #[serde(alias = "as", alias = "a")]
    pub asks: Option<heapless::Vec<PriceLevel, 100>>,
    #[serde(alias = "bs", alias = "b")]
    pub bids: Option<heapless::Vec<PriceLevel, 100>>,
    #[serde(rename = "c")]
    checksum: Option<heapless::String<32>>,
}

#[derive(Clone, Default, Debug, PartialEq)]
pub struct PriceLevel {
    pub level: usize,
    pub amount: f64,
    pub timestamp: DateTime<Utc>,
    pub sequence: i64,
    pub republished: bool,
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
        let _ = seq.next_element::<i32>().unwrap().unwrap();
        let res = seq.next_element::<Content>();
        let c = res.unwrap().unwrap();
        let _ = seq.next_element::<&str>();
        let _ = seq.next_element::<&str>();
        Ok(Message { content: c })
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
        formatter.write_str("A Kraken L2 order book update")
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let level = (seq.next_element::<&str>().unwrap().unwrap().parse::<f64>().unwrap() * 100.) as usize;
        let amount = seq.next_element::<&str>().unwrap().unwrap().parse::<f64>().unwrap();
        let timestamp_float = seq.next_element::<&str>().unwrap().unwrap().parse::<f64>().unwrap() * 1000000 as f64;
        let timestamp = Utc.timestamp_nanos((timestamp_float * (1000 as f64)) as i64);
        let rep_opt = seq.next_element::<&str>().unwrap();
        let republish = match rep_opt {
            Some(_) => true,
            None => false,
        };
        Ok(PriceLevel {
            level: level,
            amount: amount,
            timestamp: timestamp,
            sequence: 0,
            republished: republish,
        })
    }
}

#[test]
fn test_message() {
    let input = r#"
[
  0,
  {
    "as": [
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
    "bs": [
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
fn test_content_snapshot() {
    let input = r#"
  {
    "as": [
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
    "bs": [
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
fn test_content_update() {
    let input = r#"
    {
      "a": [
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
      "b": [
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
      ],
      "c": "974942666"
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