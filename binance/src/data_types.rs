use serde::{Deserialize, Deserializer};

#[derive(Debug, Deserialize, PartialEq)]
pub struct Trade {
    #[serde(rename = "E")]
    pub sent: i64,
}