use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Primitive type aliases
// ---------------------------------------------------------------------------

/// A decimal number encoded as a string.
pub type DecimalNumber = String;

/// An amount of an Account's home currency.
/// Encoded as a decimal string; precision depends on the home currency.
/// Note: AccountUnits is also re-exported from transaction.rs for historical reasons.
pub type Currency = String;

// ---------------------------------------------------------------------------
// Enums
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum Direction {
    Long,
    Short,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum AcceptDatetimeFormat {
    Unix,
    #[serde(rename = "RFC3339")]
    Rfc3339,
}

// ---------------------------------------------------------------------------
// Tag
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize, Deserialize)]
pub struct Tag {
    #[serde(rename = "type")]
    pub tag_type: String,
    pub name: String,
}

// ---------------------------------------------------------------------------
// ConversionFactor / HomeConversionFactors
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize, Deserialize)]
pub struct ConversionFactor {
    pub factor: DecimalNumber,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HomeConversionFactors {
    #[serde(rename = "gainQuoteHome")]
    pub gain_quote_home: ConversionFactor,
    #[serde(rename = "lossQuoteHome")]
    pub loss_quote_home: ConversionFactor,
    #[serde(rename = "gainBaseHome")]
    pub gain_base_home: ConversionFactor,
    #[serde(rename = "lossBaseHome")]
    pub loss_base_home: ConversionFactor,
}
