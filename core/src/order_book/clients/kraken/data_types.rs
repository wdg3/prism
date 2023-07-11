use chrono::{DateTime, TimeZone, Utc};
use serde::{Deserialize, Deserializer, de::{Visitor, SeqAccess}};

#[derive(Debug)]
pub enum Message {
    Single { content: Content },
    Double { content_1: Content, content_2: Content },
}

#[derive(Deserialize, Debug, Default, PartialEq)]
pub struct Content {
    #[serde(alias = "as", alias = "a")]
    pub asks: Option<heapless::Vec<PriceLevel, 1000>>,
    #[serde(alias = "bs", alias = "b")]
    pub bids: Option<heapless::Vec<PriceLevel, 1000>>,
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

/*impl<'de> Deserialize<'de> for Message {
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
        let mut i = 0;
        let mut c1: Option<Content> = None;
        let mut c2: Option<Content> = None;
        let _ = seq.next_element::<i32>().unwrap().unwrap();
        loop {
            let res = match seq.next_element() {
                Ok(val) => val,
                Err(err) => {
                    println!(
                        "Failed to parse event because '{}', the event will be discarded",
                        err
                    );
                    if i > 1 {
                        return Ok(Message { content_1: c1, content_2: c2});
                    } else { 
                        continue;
                    }
                }
            };
            match res {
                Some(item) => {
                    if i == 0 {
                        c1 = Some(item);
                    } else if i == 0{
                        c2 = Some(item);
                    }
                    i = i + 1;
                },
                None => break,
            };
        }
        Ok(Message { content_1: c1, content_2: c2})

    }
}*/

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
        let level = (seq.next_element::<heapless::String<32>>().unwrap().unwrap().parse::<f64>().unwrap() * 100.) as usize;
        let amount = seq.next_element::<heapless::String<32>>().unwrap().unwrap().parse::<f64>().unwrap();
        let timestamp_float = seq.next_element::<heapless::String<32>>().unwrap().unwrap().parse::<f64>().unwrap() * 1000000 as f64;
        let timestamp = Utc.timestamp_nanos((timestamp_float * (1000 as f64)) as i64);
        let rep_opt = seq.next_element::<heapless::String<32>>().unwrap();
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
/*
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
fn test_message_update() {
    let input = "[560,{\"a\":[[\"1897.20000\",\"0.00000000\",\"1689025543.609620\"],[\"1904.43000\",\"157.52791502\",\"1689025172.900666\",\"r\"]],\"c\":\"4237671103\"},\"book-100\",\"ETH/USD\"]";
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

#[test]
fn test_message_republish() {
    let input = "[560,{\"a\":[[\"1874.04000\",\"0.00000000\",\"1689026888.933331\"],[\"1890.18000\",\"0.65840067\",\"1689026285.673117\",\"r\"]]},{\"b\":[[\"1867.85000\",\"0.29882788\",\"1689026888.932974\"]],\"c\":\"1364434776\"},\"book-100\",\"ETH/USD\"]";
    let result = serde_json_core::from_str::<Message>(input);
    match &result {
        Err(e) => println!("{:?}, {:?}", e, e.to_string()),
        Ok(r) => println!("{:?}", r),
    }
    assert!(result.is_ok());
}

#[test]
fn test_content_republish() {
    let input = "{\"a\":[[\"1874.04000\",\"0.00000000\",\"1689026888.933331\"],[\"1890.18000\",\"0.65840067\",\"1689026285.673117\",\"r\"]]}";
    let result = serde_json_core::from_str::<Content>(input);
    match &result {
        Err(e) => println!("{:?}", e.to_string()),
        Ok(r) => println!("{:?}", r),
    }
    assert!(result.is_ok());
}

#[test]
fn test_price_level_republish() {
    let input = "[\"1900.69000\",\"0.06489000\",\"1689021800.678957\",\"r\"]";
    let result = serde_json_core::from_str::<PriceLevel>(input);
    match &result {
        Err(e) => println!("{:?}", e.to_string()),
        Ok(r) => println!("{:?}", r),
    }
    assert!(result.is_ok());
}*/