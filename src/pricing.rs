use crate::client::Client;
use crate::errors::{APIError, CommonErrorResponse, ErrorResponse};
use crate::handle_response;
use crate::instrument::InstrumentName;
use crate::primitives::{Currency, DecimalNumber};
use chrono::{DateTime, Utc};
use reqwest::{Request, StatusCode};
use serde::{Deserialize, Deserializer, Serialize};

/// A price expressed as a decimal string (e.g. `"1.08523"`).
///
/// OANDA represents prices as strings to avoid floating-point precision loss.
pub type PriceValue = String;

/// A combination of price component flags encoded as a string.
///
/// Each character selects a price stream to include in a response:
/// - `"B"` — bid prices
/// - `"A"` — ask prices
/// - `"M"` — mid prices
///
/// Multiple components can be combined (e.g. `"BA"` for bid and ask).
pub type PricingComponent = String;

/// A compact specification for a candlestick series, encoded as
/// `"InstrumentName:CandlestickGranularity:PricingComponent"`
/// (e.g. `"EUR_USD:H1:MBA"`).
pub type CandleSpecification = String;

// ---------------------------------------------------------------------------
// Enums
// ---------------------------------------------------------------------------

/// Indicates whether a price is currently actionable by clients.
#[derive(Debug, Serialize, Deserialize)]
pub enum PriceStatus {
    /// The price is valid and the instrument can be traded at this price.
    #[serde(rename = "tradeable")]
    Tradeable,
    /// The price is valid but the instrument cannot currently be traded
    /// (e.g. outside market hours).
    #[serde(rename = "non-tradeable")]
    NonTradeable,
    /// The price is invalid and should not be used.
    #[serde(rename = "invalid")]
    Invalid,
}

// ---------------------------------------------------------------------------
// PriceBucket
// ---------------------------------------------------------------------------

/// A single price level in an order book, pairing a price with the available
/// liquidity at that level.
#[derive(Debug, Serialize, Deserialize)]
pub struct PriceBucket {
    /// The price at this level.
    pub price: PriceValue,
    /// The number of units available at this price level.
    #[serde(deserialize_with = "deserialize_liquidity")]
    pub liquidity: i64,
}

fn deserialize_liquidity<'de, D>(deserializer: D) -> Result<i64, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum Liquidity {
        String(String),
        Integer(i64),
    }

    match Liquidity::deserialize(deserializer)? {
        Liquidity::String(s) => Ok(s.parse::<i64>().unwrap()),
        Liquidity::Integer(i) => Ok(i),
    }
}

// ---------------------------------------------------------------------------
// QuoteHomeConversionFactors (deprecated)
// ---------------------------------------------------------------------------

/// Factors for converting P&L expressed in the instrument's quote currency
/// into the account's home currency.
///
/// **Deprecated** by OANDA — use [`HomeConversions`] instead.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QuoteHomeConversionFactors {
    /// Conversion factor to apply when P&L is positive (gain).
    pub positive_units: DecimalNumber,
    /// Conversion factor to apply when P&L is negative (loss).
    pub negative_units: DecimalNumber,
}

// ---------------------------------------------------------------------------
// UnitsAvailable / UnitsAvailableDetails (deprecated)
// ---------------------------------------------------------------------------

/// The number of units available to trade on each side under a specific
/// position-fill mode.
///
/// **Deprecated** by OANDA.
#[derive(Debug, Serialize, Deserialize)]
pub struct UnitsAvailableDetails {
    /// Maximum units available to open or add to a long position.
    pub long: DecimalNumber,
    /// Maximum units available to open or add to a short position.
    pub short: DecimalNumber,
}

/// Units available to trade broken down by [`OrderPositionFill`](crate::order::OrderPositionFill)
/// mode.
///
/// **Deprecated** by OANDA.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UnitsAvailable {
    /// Units available under the `Default` position-fill mode.
    pub default: UnitsAvailableDetails,
    /// Units available under the `OpenOnly` position-fill mode.
    pub open_only: UnitsAvailableDetails,
    /// Units available under the `ReduceFirst` position-fill mode.
    pub reduce_first: UnitsAvailableDetails,
    /// Units available under the `ReduceOnly` position-fill mode.
    pub reduce_only: UnitsAvailableDetails,
}

// ---------------------------------------------------------------------------
// HomeConversions
// ---------------------------------------------------------------------------

/// Conversion factors for translating a position's value (denominated in a
/// foreign currency) into the account's home currency.
///
/// Returned alongside prices when `include_home_conversions` is requested.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HomeConversions {
    /// The foreign currency these conversion factors apply to.
    pub currency: Currency,
    /// Factor to multiply a gain (positive P&L) in `currency` by to obtain
    /// the equivalent amount in home currency.
    pub account_gain: DecimalNumber,
    /// Factor to multiply a loss (negative P&L) in `currency` by to obtain
    /// the equivalent amount in home currency.
    pub account_loss: DecimalNumber,
    /// Factor to convert a position's notional value in `currency` into home
    /// currency (used for margin calculations).
    pub position_value: DecimalNumber,
}

// ---------------------------------------------------------------------------
// ClientPrice
// ---------------------------------------------------------------------------

/// A real-time bid/ask price snapshot for a single instrument, as seen by
/// the authenticated client.
///
/// Returned by `GET /v3/accounts/{accountID}/pricing` and streamed by the
/// pricing stream endpoint.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClientPrice {
    /// The instrument this price is for (e.g. `"EUR_USD"`).
    pub instrument: Option<InstrumentName>,
    /// The timestamp at which this price was generated by OANDA.
    #[serde(alias = "time")]
    pub timestamp: Option<DateTime<Utc>>,
    /// The tradeable status of the instrument at the time of this price.
    /// `None` if the status is unknown.
    pub status: Option<PriceStatus>,
    /// `true` if the instrument can currently be traded at this price.
    pub tradeable: Option<bool>,
    /// The bid-side order book levels, ordered from best (highest) to worst.
    pub bids: Vec<PriceBucket>,
    /// The ask-side order book levels, ordered from best (lowest) to worst.
    pub asks: Vec<PriceBucket>,
    /// The bid price used when closing positions or triggering stop-loss
    /// orders during margin closeout. May differ from the best bid.
    pub closeout_bid: PriceValue,
    /// The ask price used when closing positions or triggering stop-loss
    /// orders during margin closeout. May differ from the best ask.
    pub closeout_ask: PriceValue,
    /// Deprecated conversion factors for quote-to-home-currency P&L.
    /// Use the `home_conversions` field in [`PricesResponse`] instead.
    pub quote_home_conversion_factors: Option<QuoteHomeConversionFactors>,
    /// Deprecated breakdown of units available to trade by position-fill mode.
    pub units_available: Option<UnitsAvailable>,
}

// ---------------------------------------------------------------------------
// PricingHeartbeat
// ---------------------------------------------------------------------------

/// A keepalive message sent periodically over the pricing stream to confirm
/// the connection is still active.
#[derive(Debug, Serialize, Deserialize)]
pub struct PricingHeartbeat {
    /// The server timestamp at which the heartbeat was generated.
    pub time: DateTime<Utc>,
}

// ---------------------------------------------------------------------------
// PricingStreamItem
// ---------------------------------------------------------------------------

/// A single item in the pricing stream, which is either a live price update
/// or a heartbeat.
///
/// Deserialised from newline-delimited JSON using the `"type"` field as a tag.
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum PricingStreamItem {
    /// A live price update for an instrument.
    ///
    /// Boxed because `ClientPrice` is much larger than a heartbeat.
    PRICE(Box<ClientPrice>),
    /// A periodic keepalive message confirming the stream is active.
    HEARTBEAT(PricingHeartbeat),
}

// ---------------------------------------------------------------------------
// Response types
// ---------------------------------------------------------------------------

/// Response body for `GET /v3/accounts/{accountID}/pricing`.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PricesResponse {
    /// The current prices for each requested instrument.
    pub prices: Vec<ClientPrice>,
    /// Home-currency conversion factors for each foreign currency involved in
    /// the requested instruments. `None` if not requested.
    pub home_conversions: Option<Vec<HomeConversions>>,
    /// The server timestamp at which the prices were generated. `None` when
    /// not included in the response.
    pub time: Option<DateTime<Utc>>,
}

// ---------------------------------------------------------------------------
// Service
// ---------------------------------------------------------------------------

/// Provides access to the OANDA Pricing endpoints
/// (`/v3/accounts/{id}/pricing/...`).
///
/// Obtain an instance via [`Client::pricing`](crate::client::Client::pricing).
pub struct PricingService<'a> {
    client: &'a Client,
}

impl<'a> PricingService<'a> {
    /// Creates a new `PricingService` bound to the given client.
    pub(crate) fn new(client: &'a Client) -> Self {
        Self { client }
    }

    /// Fetches the current bid/ask prices for one or more instruments.
    ///
    /// Calls `GET /v3/accounts/{accountID}/pricing` with the given
    /// `instruments` list as a comma-separated query parameter.
    ///
    /// Returns [`APIError::InvalidRequest`] if no account ID is configured.
    pub async fn get(&self, instruments: Vec<InstrumentName>) -> Result<PricesResponse, APIError> {
        let mut url = self.client.account_url("pricing")?;
        url.query_pairs_mut()
            .append_pair("instruments", instruments.join(",").as_str());
        let http_req = Request::new(reqwest::Method::GET, url);
        let http_resp = self.client.http_client.execute(http_req).await?;
        handle_response!(
            http_resp,
            success: StatusCode::OK => PricesResponse,
            errors: [ ]
        )
    }
}

#[cfg(test)]
mod tests {
    use crate::client::setup_test_client;

    #[tokio::test]
    #[ignore = "requires OANDA demo credentials; run explicitly with --ignored"]
    async fn test_get_prices() {
        let client = setup_test_client();
        let resp = client
            .pricing()
            .get(vec!["USD_JPY".to_string(), "EUR_USD".to_string()])
            .await
            .unwrap();
        println!("{:#?}", resp);
    }
}
