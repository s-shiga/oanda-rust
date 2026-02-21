use chrono::{DateTime, Utc};
use crate::client::Client;
use crate::errors::APIError;
use crate::instrument::InstrumentName;
use crate::order::{GuaranteedStopLossOrder, OrderID, StopLossOrder, TakeProfitOrder, TrailingStopLossOrder};
use crate::pricing::PriceValue;
use crate::primitives::DecimalNumber;
use crate::transaction::{AccountUnits, ClientExtensions, TransactionID, TradeID};
use reqwest::{Request, StatusCode};
use serde::{Deserialize, Serialize};

pub type TradeSpecifier = String;

// ---------------------------------------------------------------------------
// Enums
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TradeState {
    Open,
    Closed,
    CloseWhenTradeable,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TradeStateFilter {
    Open,
    Closed,
    CloseWhenTradeable,
    All,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum TradePL {
    Positive,
    Negative,
    Zero,
}

// ---------------------------------------------------------------------------
// Trade
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize, Deserialize)]
pub struct Trade {
    pub id: TradeID,
    pub instrument: InstrumentName,
    pub price: PriceValue,
    #[serde(rename = "openTime")]
    pub open_time: DateTime<Utc>,
    pub state: TradeState,
    #[serde(rename = "initialUnits")]
    pub initial_units: DecimalNumber,
    #[serde(rename = "initialMarginRequired")]
    pub initial_margin_required: AccountUnits,
    #[serde(rename = "currentUnits")]
    pub current_units: DecimalNumber,
    #[serde(rename = "realizedPL")]
    pub realized_pl: AccountUnits,
    #[serde(rename = "unrealizedPL")]
    pub unrealized_pl: AccountUnits,
    #[serde(rename = "marginUsed")]
    pub margin_used: AccountUnits,
    #[serde(rename = "averageClosePrice")]
    pub average_close_price: Option<PriceValue>,
    #[serde(rename = "closingTransactionIDs")]
    pub closing_transaction_ids: Option<Vec<TransactionID>>,
    pub financing: AccountUnits,
    #[serde(rename = "dividendAdjustment")]
    pub dividend_adjustment: AccountUnits,
    #[serde(rename = "closeTime")]
    pub close_time: Option<DateTime<Utc>>,
    #[serde(rename = "clientExtensions")]
    pub client_extensions: Option<ClientExtensions>,
    #[serde(rename = "takeProfitOrder")]
    pub take_profit_order: Option<TakeProfitOrder>,
    #[serde(rename = "stopLossOrder")]
    pub stop_loss_order: Option<StopLossOrder>,
    #[serde(rename = "guaranteedStopLossOrder")]
    pub guaranteed_stop_loss_order: Option<GuaranteedStopLossOrder>,
    #[serde(rename = "trailingStopLossOrder")]
    pub trailing_stop_loss_order: Option<TrailingStopLossOrder>,
}

// ---------------------------------------------------------------------------
// TradeSummary
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize, Deserialize)]
pub struct TradeSummary {
    pub id: TradeID,
    pub instrument: InstrumentName,
    pub price: PriceValue,
    #[serde(rename = "openTime")]
    pub open_time: DateTime<Utc>,
    pub state: TradeState,
    #[serde(rename = "initialUnits")]
    pub initial_units: DecimalNumber,
    #[serde(rename = "initialMarginRequired")]
    pub initial_margin_required: AccountUnits,
    #[serde(rename = "currentUnits")]
    pub current_units: DecimalNumber,
    #[serde(rename = "realizedPL")]
    pub realized_pl: AccountUnits,
    #[serde(rename = "unrealizedPL")]
    pub unrealized_pl: AccountUnits,
    #[serde(rename = "marginUsed")]
    pub margin_used: AccountUnits,
    #[serde(rename = "averageClosePrice")]
    pub average_close_price: Option<PriceValue>,
    #[serde(rename = "closingTransactionIDs")]
    pub closing_transaction_ids: Option<Vec<TransactionID>>,
    pub financing: AccountUnits,
    #[serde(rename = "dividendAdjustment")]
    pub dividend_adjustment: AccountUnits,
    #[serde(rename = "closeTime")]
    pub close_time: Option<DateTime<Utc>>,
    #[serde(rename = "clientExtensions")]
    pub client_extensions: Option<ClientExtensions>,
    #[serde(rename = "takeProfitOrderID")]
    pub take_profit_order_id: Option<OrderID>,
    #[serde(rename = "stopLossOrderID")]
    pub stop_loss_order_id: Option<OrderID>,
    #[serde(rename = "guaranteedStopLossOrderID")]
    pub guaranteed_stop_loss_order_id: Option<OrderID>,
    #[serde(rename = "trailingStopLossOrderID")]
    pub trailing_stop_loss_order_id: Option<OrderID>,
}

// ---------------------------------------------------------------------------
// CalculatedTradeState
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize, Deserialize)]
pub struct CalculatedTradeState {
    pub id: TradeID,
    #[serde(rename = "unrealizedPL")]
    pub unrealized_pl: AccountUnits,
    #[serde(rename = "marginUsed")]
    pub margin_used: AccountUnits,
}

// ---------------------------------------------------------------------------
// Response types
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize, Deserialize)]
pub struct TradesResponse {
    pub trades: Vec<Trade>,
    #[serde(rename = "lastTransactionID")]
    pub last_transaction_id: TransactionID,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TradeResponse {
    pub trade: Trade,
    #[serde(rename = "lastTransactionID")]
    pub last_transaction_id: TransactionID,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CloseTradeResponse {
    #[serde(rename = "orderFillTransaction")]
    pub order_fill_transaction: Option<serde_json::Value>,
    #[serde(rename = "orderCancelTransaction")]
    pub order_cancel_transaction: Option<serde_json::Value>,
    #[serde(rename = "relatedTransactionIDs")]
    pub related_transaction_ids: Option<Vec<TransactionID>>,
    #[serde(rename = "lastTransactionID")]
    pub last_transaction_id: Option<TransactionID>,
}

// ---------------------------------------------------------------------------
// Service
// ---------------------------------------------------------------------------

pub struct TradeService<'a> {
    client: &'a Client,
}

impl<'a> TradeService<'a> {
    pub fn new(client: &'a Client) -> Self {
        Self { client }
    }

    pub async fn list(&self) -> Result<TradesResponse, APIError> {
        let url = self
            .client
            .base_url
            .join(
                format!(
                    "/v3/accounts/{}/trades",
                    self.client.account_id.as_ref().expect("Missing account_id")
                )
                .as_str(),
            )
            .unwrap();
        let http_req = Request::new(reqwest::Method::GET, url);
        let http_resp = self.client.http_client.execute(http_req).await?;
        match http_resp.status() {
            StatusCode::OK => Ok(http_resp.json::<TradesResponse>().await?),
            status => Err(APIError::ApiErrorResponse {
                status,
                message: http_resp.text().await?,
            }),
        }
    }

    pub async fn list_open(&self) -> Result<TradesResponse, APIError> {
        let url = self
            .client
            .base_url
            .join(
                format!(
                    "/v3/accounts/{}/openTrades",
                    self.client.account_id.as_ref().expect("Missing account_id")
                )
                .as_str(),
            )
            .unwrap();
        let http_req = Request::new(reqwest::Method::GET, url);
        let http_resp = self.client.http_client.execute(http_req).await?;
        match http_resp.status() {
            StatusCode::OK => Ok(http_resp.json::<TradesResponse>().await?),
            status => Err(APIError::ApiErrorResponse {
                status,
                message: http_resp.text().await?,
            }),
        }
    }

    pub async fn get(&self, specifier: TradeSpecifier) -> Result<TradeResponse, APIError> {
        let url = self
            .client
            .base_url
            .join(
                format!(
                    "/v3/accounts/{}/trades/{}",
                    self.client.account_id.as_ref().expect("Missing account_id"),
                    specifier
                )
                .as_str(),
            )
            .unwrap();
        let http_req = Request::new(reqwest::Method::GET, url);
        let http_resp = self.client.http_client.execute(http_req).await?;
        match http_resp.status() {
            StatusCode::OK => Ok(http_resp.json::<TradeResponse>().await?),
            status => Err(APIError::ApiErrorResponse {
                status,
                message: http_resp.text().await?,
            }),
        }
    }

    pub async fn close(&self, specifier: TradeSpecifier) -> Result<CloseTradeResponse, APIError> {
        let url = self
            .client
            .base_url
            .join(
                format!(
                    "/v3/accounts/{}/trades/{}/close",
                    self.client.account_id.as_ref().expect("Missing account_id"),
                    specifier
                )
                .as_str(),
            )
            .unwrap();
        let http_resp = self.client.http_client.put(url).send().await?;
        match http_resp.status() {
            StatusCode::OK => Ok(http_resp.json::<CloseTradeResponse>().await?),
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
    async fn test_list_trades() {
        let client = setup_test_client();
        let resp = client.trade().list().await.unwrap();
        println!("{:#?}", resp);
    }

    #[tokio::test]
    async fn test_list_open_trades() {
        let client = setup_test_client();
        let resp = client.trade().list_open().await.unwrap();
        println!("{:#?}", resp);
    }
}
