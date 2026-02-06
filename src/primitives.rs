use serde::{Serialize, Deserialize};

pub type DecimalNumber = String;

pub type Currency = String;

#[derive(Debug, Serialize, Deserialize)]
pub struct Tag {
    #[serde(rename = "type")]
    pub tag_type: String,
    pub name: String,
}