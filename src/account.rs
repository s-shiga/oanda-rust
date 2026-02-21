use crate::client::Client;
use crate::errors::APIError;
use crate::order::{DynamicOrderState, Order};
use crate::position::{CalculatedPositionState, Position};
use crate::primitives::{Currency, DecimalNumber};
use crate::trade::{CalculatedTradeState, TradeSummary};
use crate::transaction::{AccountUnits, TransactionID};
use chrono::{DateTime, Utc};
use reqwest::{Request, StatusCode};
use serde::{Deserialize, Serialize};

pub type AccountID = String;

// ---------------------------------------------------------------------------
// Enums
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum GuaranteedStopLossOrderMode {
    Disabled,
    Allowed,
    Required,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum GuaranteedStopLossOrderMutability {
    Fixed,
    Replaceable,
    Cancelable,
    PriceWidenOnly,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AccountFinancingMode {
    NoFinancing,
    SecondBySecond,
    Daily,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PositionAggregationMode {
    AbsoluteSum,
    MaximalSide,
    NetSum,
}

// ---------------------------------------------------------------------------
// GuaranteedStopLossOrderParameters
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize, Deserialize)]
pub struct GuaranteedStopLossOrderParameters {
    #[serde(rename = "mutabilityMarketOpen")]
    pub mutability_market_open: GuaranteedStopLossOrderMutability,
    #[serde(rename = "mutabilityMarketHalted")]
    pub mutability_market_halted: GuaranteedStopLossOrderMutability,
}

// ---------------------------------------------------------------------------
// Account
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize, Deserialize)]
pub struct Account {
    pub id: AccountID,
    pub alias: Option<String>,
    pub currency: Currency,
    #[serde(rename = "createdByUserID")]
    pub created_by_user_id: i64,
    #[serde(rename = "createdTime")]
    pub created_time: DateTime<Utc>,
    #[serde(rename = "guaranteedStopLossOrderParameters")]
    pub guaranteed_stop_loss_order_parameters: Option<GuaranteedStopLossOrderParameters>,
    #[serde(rename = "guaranteedStopLossOrderMode")]
    pub guaranteed_stop_loss_order_mode: Option<GuaranteedStopLossOrderMode>,
    #[serde(rename = "guaranteedStopLossOrderMutability")]
    pub guaranteed_stop_loss_order_mutability: Option<GuaranteedStopLossOrderMutability>,
    #[serde(rename = "resettablePLTime")]
    pub resettable_pl_time: Option<DateTime<Utc>>,
    #[serde(rename = "marginRate")]
    pub margin_rate: Option<DecimalNumber>,
    #[serde(rename = "openTradeCount")]
    pub open_trade_count: Option<i32>,
    #[serde(rename = "openPositionCount")]
    pub open_position_count: Option<i32>,
    #[serde(rename = "pendingOrderCount")]
    pub pending_order_count: Option<i32>,
    #[serde(rename = "hedgingEnabled")]
    pub hedging_enabled: Option<bool>,
    #[serde(rename = "unrealizedPL")]
    pub unrealized_pl: Option<AccountUnits>,
    #[serde(rename = "NAV")]
    pub nav: Option<AccountUnits>,
    #[serde(rename = "marginUsed")]
    pub margin_used: Option<AccountUnits>,
    #[serde(rename = "marginAvailable")]
    pub margin_available: Option<AccountUnits>,
    #[serde(rename = "positionValue")]
    pub position_value: Option<AccountUnits>,
    #[serde(rename = "marginCloseoutUnrealizedPL")]
    pub margin_closeout_unrealized_pl: Option<AccountUnits>,
    #[serde(rename = "marginCloseoutNAV")]
    pub margin_closeout_nav: Option<AccountUnits>,
    #[serde(rename = "marginCloseoutMarginUsed")]
    pub margin_closeout_margin_used: Option<AccountUnits>,
    #[serde(rename = "marginCloseoutPercent")]
    pub margin_closeout_percent: Option<DecimalNumber>,
    #[serde(rename = "marginCloseoutPositionValue")]
    pub margin_closeout_position_value: Option<DecimalNumber>,
    #[serde(rename = "withdrawalLimit")]
    pub withdrawal_limit: Option<AccountUnits>,
    #[serde(rename = "marginCallMarginUsed")]
    pub margin_call_margin_used: Option<AccountUnits>,
    #[serde(rename = "marginCallPercent")]
    pub margin_call_percent: Option<DecimalNumber>,
    pub balance: Option<AccountUnits>,
    pub pl: Option<AccountUnits>,
    #[serde(rename = "resettablePL")]
    pub resettable_pl: Option<AccountUnits>,
    pub financing: Option<AccountUnits>,
    pub commission: Option<AccountUnits>,
    #[serde(rename = "dividendAdjustment")]
    pub dividend_adjustment: Option<AccountUnits>,
    #[serde(rename = "guaranteedExecutionFees")]
    pub guaranteed_execution_fees: Option<AccountUnits>,
    #[serde(rename = "marginCallEnterTime")]
    pub margin_call_enter_time: Option<DateTime<Utc>>,
    #[serde(rename = "marginCallExtensionCount")]
    pub margin_call_extension_count: Option<i32>,
    #[serde(rename = "lastMarginCallExtensionTime")]
    pub last_margin_call_extension_time: Option<DateTime<Utc>>,
    #[serde(rename = "lastTransactionID")]
    pub last_transaction_id: Option<TransactionID>,
    pub trades: Option<Vec<TradeSummary>>,
    pub positions: Option<Vec<Position>>,
    pub orders: Option<Vec<Order>>,
}

// ---------------------------------------------------------------------------
// AccountSummary (same shape as Account but without open trades/positions/orders)
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize, Deserialize)]
pub struct AccountSummary {
    pub id: AccountID,
    pub alias: Option<String>,
    pub currency: Currency,
    #[serde(rename = "createdByUserID")]
    pub created_by_user_id: i64,
    #[serde(rename = "createdTime")]
    pub created_time: DateTime<Utc>,
    #[serde(rename = "guaranteedStopLossOrderParameters")]
    pub guaranteed_stop_loss_order_parameters: Option<GuaranteedStopLossOrderParameters>,
    #[serde(rename = "guaranteedStopLossOrderMode")]
    pub guaranteed_stop_loss_order_mode: Option<GuaranteedStopLossOrderMode>,
    #[serde(rename = "guaranteedStopLossOrderMutability")]
    pub guaranteed_stop_loss_order_mutability: Option<GuaranteedStopLossOrderMutability>,
    #[serde(rename = "resettablePLTime")]
    pub resettable_pl_time: Option<DateTime<Utc>>,
    #[serde(rename = "marginRate")]
    pub margin_rate: Option<DecimalNumber>,
    #[serde(rename = "openTradeCount")]
    pub open_trade_count: Option<i32>,
    #[serde(rename = "openPositionCount")]
    pub open_position_count: Option<i32>,
    #[serde(rename = "pendingOrderCount")]
    pub pending_order_count: Option<i32>,
    #[serde(rename = "hedgingEnabled")]
    pub hedging_enabled: Option<bool>,
    #[serde(rename = "unrealizedPL")]
    pub unrealized_pl: Option<AccountUnits>,
    #[serde(rename = "NAV")]
    pub nav: Option<AccountUnits>,
    #[serde(rename = "marginUsed")]
    pub margin_used: Option<AccountUnits>,
    #[serde(rename = "marginAvailable")]
    pub margin_available: Option<AccountUnits>,
    #[serde(rename = "positionValue")]
    pub position_value: Option<AccountUnits>,
    #[serde(rename = "marginCloseoutUnrealizedPL")]
    pub margin_closeout_unrealized_pl: Option<AccountUnits>,
    #[serde(rename = "marginCloseoutNAV")]
    pub margin_closeout_nav: Option<AccountUnits>,
    #[serde(rename = "marginCloseoutMarginUsed")]
    pub margin_closeout_margin_used: Option<AccountUnits>,
    #[serde(rename = "marginCloseoutPercent")]
    pub margin_closeout_percent: Option<DecimalNumber>,
    #[serde(rename = "marginCloseoutPositionValue")]
    pub margin_closeout_position_value: Option<DecimalNumber>,
    #[serde(rename = "withdrawalLimit")]
    pub withdrawal_limit: Option<AccountUnits>,
    #[serde(rename = "marginCallMarginUsed")]
    pub margin_call_margin_used: Option<AccountUnits>,
    #[serde(rename = "marginCallPercent")]
    pub margin_call_percent: Option<DecimalNumber>,
    pub balance: Option<AccountUnits>,
    pub pl: Option<AccountUnits>,
    #[serde(rename = "resettablePL")]
    pub resettable_pl: Option<AccountUnits>,
    pub financing: Option<AccountUnits>,
    pub commission: Option<AccountUnits>,
    #[serde(rename = "dividendAdjustment")]
    pub dividend_adjustment: Option<AccountUnits>,
    #[serde(rename = "guaranteedExecutionFees")]
    pub guaranteed_execution_fees: Option<AccountUnits>,
    #[serde(rename = "marginCallEnterTime")]
    pub margin_call_enter_time: Option<DateTime<Utc>>,
    #[serde(rename = "marginCallExtensionCount")]
    pub margin_call_extension_count: Option<i32>,
    #[serde(rename = "lastMarginCallExtensionTime")]
    pub last_margin_call_extension_time: Option<DateTime<Utc>>,
    #[serde(rename = "lastTransactionID")]
    pub last_transaction_id: Option<TransactionID>,
}

// ---------------------------------------------------------------------------
// AccountProperties
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize, Deserialize)]
pub struct AccountProperties {
    pub id: AccountID,
    #[serde(rename = "mt4AccountID")]
    pub mt4_account_id: Option<i32>,
    pub tags: Vec<String>,
}

// ---------------------------------------------------------------------------
// AccountChangesState
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize, Deserialize)]
pub struct AccountChangesState {
    #[serde(rename = "unrealizedPL")]
    pub unrealized_pl: Option<AccountUnits>,
    #[serde(rename = "NAV")]
    pub nav: Option<AccountUnits>,
    #[serde(rename = "marginUsed")]
    pub margin_used: Option<AccountUnits>,
    #[serde(rename = "marginAvailable")]
    pub margin_available: Option<AccountUnits>,
    #[serde(rename = "positionValue")]
    pub position_value: Option<AccountUnits>,
    #[serde(rename = "marginCloseoutUnrealizedPL")]
    pub margin_closeout_unrealized_pl: Option<AccountUnits>,
    #[serde(rename = "marginCloseoutNAV")]
    pub margin_closeout_nav: Option<AccountUnits>,
    #[serde(rename = "marginCloseoutMarginUsed")]
    pub margin_closeout_margin_used: Option<AccountUnits>,
    #[serde(rename = "marginCloseoutPercent")]
    pub margin_closeout_percent: Option<DecimalNumber>,
    #[serde(rename = "marginCloseoutPositionValue")]
    pub margin_closeout_position_value: Option<DecimalNumber>,
    #[serde(rename = "withdrawalLimit")]
    pub withdrawal_limit: Option<AccountUnits>,
    #[serde(rename = "marginCallMarginUsed")]
    pub margin_call_margin_used: Option<AccountUnits>,
    #[serde(rename = "marginCallPercent")]
    pub margin_call_percent: Option<DecimalNumber>,
    pub balance: Option<AccountUnits>,
    pub pl: Option<AccountUnits>,
    #[serde(rename = "resettablePL")]
    pub resettable_pl: Option<AccountUnits>,
    pub financing: Option<AccountUnits>,
    pub commission: Option<AccountUnits>,
    #[serde(rename = "dividendAdjustment")]
    pub dividend_adjustment: Option<AccountUnits>,
    #[serde(rename = "guaranteedExecutionFees")]
    pub guaranteed_execution_fees: Option<AccountUnits>,
    #[serde(rename = "marginCallEnterTime")]
    pub margin_call_enter_time: Option<DateTime<Utc>>,
    #[serde(rename = "marginCallExtensionCount")]
    pub margin_call_extension_count: Option<i32>,
    #[serde(rename = "lastMarginCallExtensionTime")]
    pub last_margin_call_extension_time: Option<DateTime<Utc>>,
    pub orders: Option<Vec<DynamicOrderState>>,
    pub trades: Option<Vec<CalculatedTradeState>>,
    pub positions: Option<Vec<CalculatedPositionState>>,
}

// ---------------------------------------------------------------------------
// AccountChanges
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize, Deserialize)]
pub struct AccountChanges {
    #[serde(rename = "ordersCreated")]
    pub orders_created: Option<Vec<Order>>,
    #[serde(rename = "ordersCancelled")]
    pub orders_cancelled: Option<Vec<Order>>,
    #[serde(rename = "ordersFilled")]
    pub orders_filled: Option<Vec<Order>>,
    #[serde(rename = "ordersTriggered")]
    pub orders_triggered: Option<Vec<Order>>,
    #[serde(rename = "tradesOpened")]
    pub trades_opened: Option<Vec<TradeSummary>>,
    #[serde(rename = "tradesReduced")]
    pub trades_reduced: Option<Vec<TradeSummary>>,
    #[serde(rename = "tradesClosed")]
    pub trades_closed: Option<Vec<TradeSummary>>,
    pub positions: Option<Vec<Position>>,
    pub transactions: Option<Vec<serde_json::Value>>, // Transaction (complex union type)
}

// ---------------------------------------------------------------------------
// UserAttributes
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize, Deserialize)]
pub struct UserAttributes {
    #[serde(rename = "userID")]
    pub user_id: i64,
    pub username: String,
    pub title: String,
    pub name: String,
    pub email: String,
    #[serde(rename = "divisionAbbreviation")]
    pub division_abbreviation: String,
    #[serde(rename = "languageAbbreviation")]
    pub language_abbreviation: String,
    #[serde(rename = "homeCurrency")]
    pub home_currency: Currency,
}

// ---------------------------------------------------------------------------
// Response types
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize, Deserialize)]
pub struct ListAccountsResponse {
    pub accounts: Vec<AccountProperties>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetAccountDetailsResponse {
    pub account: Account,
    #[serde(rename = "lastTransactionID")]
    pub last_transaction_id: TransactionID,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetAccountSummaryResponse {
    pub account: AccountSummary,
    #[serde(rename = "lastTransactionID")]
    pub last_transaction_id: TransactionID,
}

// ---------------------------------------------------------------------------
// Service
// ---------------------------------------------------------------------------

pub struct AccountService<'a> {
    client: &'a Client,
}

impl<'a> AccountService<'a> {
    pub fn new(client: &'a Client) -> Self {
        Self { client }
    }

    pub async fn list(&self) -> Result<ListAccountsResponse, APIError> {
        let url = self.client.base_url.join("/v3/accounts").unwrap();
        let http_req = Request::new(reqwest::Method::GET, url);
        let http_resp = self.client.http_client.execute(http_req).await?;
        match http_resp.status() {
            StatusCode::OK => {
                let resp = http_resp.json::<ListAccountsResponse>().await?;
                Ok(resp)
            }
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
    async fn test_list() {
        let client = setup_test_client();
        let account = client.account().list().await.unwrap();
        println!("{:#?}", account);
    }
}
