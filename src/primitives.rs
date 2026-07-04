use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Primitive type aliases
// ---------------------------------------------------------------------------

/// A decimal number serialised as a string (e.g. `"1.23456"`).
///
/// OANDA encodes all numeric values this way to avoid floating-point precision loss.
pub type DecimalNumber = String;

/// An ISO 4217 currency code (e.g. `"USD"`, `"JPY"`, `"EUR"`).
pub type Currency = String;

// ---------------------------------------------------------------------------
// Enums
// ---------------------------------------------------------------------------

/// Whether a position or trade is on the buy (long) or sell (short) side.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum Direction {
    /// The buy side — the account holds units expecting the price to rise.
    Long,
    /// The sell side — the account holds units expecting the price to fall.
    Short,
}

/// Format in which the API should return datetime strings.
///
/// Passed as the `Accept-Datetime-Format` request header. Defaults to `RFC3339`
/// if omitted.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum AcceptDatetimeFormat {
    /// Unix epoch seconds as a decimal string (e.g. `"1737100800.000000000"`).
    Unix,
    /// RFC 3339 / ISO 8601 format (e.g. `"2025-01-17T12:00:00.000000000Z"`).
    #[serde(rename = "RFC3339")]
    Rfc3339,
}

/// Serde deserializer for optional OANDA datetime fields.
///
/// OANDA represents "no datetime" as the string `"0"` rather than JSON `null`.
/// This helper maps `"0"` to `None` and any valid RFC 3339 timestamp to `Some`.
pub fn deserialize_datetime<'de, D>(deserializer: D) -> Result<Option<DateTime<Utc>>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum Value {
        DateTime(DateTime<Utc>),
        None(String),
    }

    match Value::deserialize(deserializer)? {
        Value::DateTime(dt) => Ok(Some(dt)),
        Value::None(s) if s == "0" => Ok(None),
        _ => Err(serde::de::Error::custom("unexpected datetime")),
    }
}

// ---------------------------------------------------------------------------
// Tag
// ---------------------------------------------------------------------------

/// A user-defined label attached to a trade or order for categorisation.
#[derive(Debug, Serialize, Deserialize)]
pub struct Tag {
    /// Category of the tag (e.g. `"ORDER"`, `"TRADE"`).
    #[serde(rename = "type")]
    pub tag_type: String,
    /// Human-readable name of the tag.
    pub name: String,
}

// ---------------------------------------------------------------------------
// ConversionFactor / HomeConversionFactors
// ---------------------------------------------------------------------------

/// A multiplier used to convert an instrument's quote or base currency P&L into
/// the account's home currency.
#[derive(Debug, Serialize, Deserialize)]
pub struct ConversionFactor {
    /// The multiplicative conversion rate.
    pub factor: DecimalNumber,
}

/// The set of conversion factors needed to translate unrealised P&L for an
/// open position into the account's home currency.
///
/// Returned as part of dynamic account-state responses. Four factors cover the
/// four cases: gain/loss × quote/base currency.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HomeConversionFactors {
    /// Factor for converting a quote-currency gain to home currency.
    pub gain_quote_home: ConversionFactor,
    /// Factor for converting a quote-currency loss to home currency.
    pub loss_quote_home: ConversionFactor,
    /// Factor for converting a base-currency gain to home currency.
    pub gain_base_home: ConversionFactor,
    /// Factor for converting a base-currency loss to home currency.
    pub loss_base_home: ConversionFactor,
}
