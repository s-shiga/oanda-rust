use chrono::{DateTime, Utc};
use crate::client::Client;
use crate::errors::APIError;
use crate::instrument::InstrumentName;
use crate::primitives::{Currency, DecimalNumber};
use reqwest::{Request, StatusCode};
use serde::{Deserialize, Serialize};

pub type PriceValue = String;
pub type PricingComponent = String;
pub type CandleSpecification = String; // InstrumentName:CandlestickGranularity:PricingComponent

// ---------------------------------------------------------------------------
// Enums
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize, Deserialize)]
pub enum PriceStatus {
    #[serde(rename = "tradeable")]
    Tradeable,
    #[serde(rename = "non-tradeable")]
    NonTradeable,
    #[serde(rename = "invalid")]
    Invalid,
}

// ---------------------------------------------------------------------------
// PriceBucket
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize, Deserialize)]
pub struct PriceBucket {
    pub price: PriceValue,
    pub liquidity: i64,
}

// ---------------------------------------------------------------------------
// QuoteHomeConversionFactors (deprecated)
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize, Deserialize)]
pub struct QuoteHomeConversionFactors {
    #[serde(rename = "positiveUnits")]
    pub positive_units: DecimalNumber,
    #[serde(rename = "negativeUnits")]
    pub negative_units: DecimalNumber,
}

// ---------------------------------------------------------------------------
// UnitsAvailable / UnitsAvailableDetails (deprecated)
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize, Deserialize)]
pub struct UnitsAvailableDetails {
    pub long: DecimalNumber,
    pub short: DecimalNumber,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UnitsAvailable {
    pub default: UnitsAvailableDetails,
    #[serde(rename = "openOnly")]
    pub open_only: UnitsAvailableDetails,
    #[serde(rename = "reduceFirst")]
    pub reduce_first: UnitsAvailableDetails,
    #[serde(rename = "reduceOnly")]
    pub reduce_only: UnitsAvailableDetails,
}

// ---------------------------------------------------------------------------
// HomeConversions
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize, Deserialize)]
pub struct HomeConversions {
    pub currency: Currency,
    #[serde(rename = "accountGain")]
    pub account_gain: DecimalNumber,
    #[serde(rename = "accountLoss")]
    pub account_loss: DecimalNumber,
    #[serde(rename = "positionValue")]
    pub position_value: DecimalNumber,
}

// ---------------------------------------------------------------------------
// ClientPrice
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize, Deserialize)]
pub struct ClientPrice {
    pub instrument: InstrumentName,
    pub time: DateTime<Utc>,
    pub status: Option<PriceStatus>,
    pub tradeable: bool,
    pub bids: Vec<PriceBucket>,
    pub asks: Vec<PriceBucket>,
    #[serde(rename = "closeoutBid")]
    pub closeout_bid: PriceValue,
    #[serde(rename = "closeoutAsk")]
    pub closeout_ask: PriceValue,
    #[serde(rename = "quoteHomeConversionFactors")]
    pub quote_home_conversion_factors: Option<QuoteHomeConversionFactors>,
    #[serde(rename = "unitsAvailable")]
    pub units_available: Option<UnitsAvailable>,
}

// ---------------------------------------------------------------------------
// PricingHeartbeat
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize, Deserialize)]
pub struct PricingHeartbeat {
    pub time: DateTime<Utc>,
}

// ---------------------------------------------------------------------------
// PricingStreamItem
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum PricingStreamItem {
    PRICE(ClientPrice),
    HEARTBEAT(PricingHeartbeat),
}

// ---------------------------------------------------------------------------
// Response types
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize, Deserialize)]
pub struct PricesResponse {
    pub prices: Vec<ClientPrice>,
    #[serde(rename = "homeConversions")]
    pub home_conversions: Option<Vec<HomeConversions>>,
    pub time: Option<DateTime<Utc>>,
}

// ---------------------------------------------------------------------------
// Service
// ---------------------------------------------------------------------------

pub struct PricingService<'a> {
    client: &'a Client,
}

impl<'a> PricingService<'a> {
    pub fn new(client: &'a Client) -> Self {
        Self { client }
    }

    pub async fn get(&self, instruments: Vec<InstrumentName>) -> Result<PricesResponse, APIError> {
        let mut url = self
            .client
            .base_url
            .join(
                format!(
                    "/v3/accounts/{}/pricing",
                    self.client.account_id.as_ref().expect("Missing account_id")
                )
                .as_str(),
            )
            .unwrap();
        url.query_pairs_mut()
            .append_pair("instruments", instruments.join(",").as_str());
        let http_req = Request::new(reqwest::Method::GET, url);
        let http_resp = self.client.http_client.execute(http_req).await?;
        match http_resp.status() {
            StatusCode::OK => Ok(http_resp.json::<PricesResponse>().await?),
            status => Err(APIError::ApiErrorResponse {
                status,
                message: http_resp.text().await?,
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::client::setup_test_client;

    #[tokio::test]
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
