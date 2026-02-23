use crate::client::Client;
use crate::errors::APIError;
use crate::order::{DynamicOrderState, Order};
use crate::position::{CalculatedPositionState, Position};
use crate::primitives::{Currency, DecimalNumber, deserialize_datetime};
use crate::trade::{CalculatedTradeState, TradeSummary};
use crate::transaction::{AccountUnits, TransactionID};
use chrono::{DateTime, Utc};
use reqwest::{Request, StatusCode};
use serde::{Deserialize, Serialize};

/// A unique identifier for an OANDA account (e.g. `"101-001-1234567-001"`).
pub type AccountID = String;

// ---------------------------------------------------------------------------
// Account
// ---------------------------------------------------------------------------

/// Full representation of an OANDA account, including all open trades,
/// positions, and pending orders.
///
/// Returned by the `GET /v3/accounts/{accountID}` endpoint. Fields that are
/// not always populated by OANDA are modelled as `Option`.
#[derive(Debug, Serialize, Deserialize)]
pub struct Account {
    /// The account's unique identifier.
    pub id: AccountID,
    /// An optional human-readable label for the account.
    pub alias: Option<String>,
    /// The home currency of the account (e.g. `"USD"`).
    pub currency: Currency,
    /// The numeric ID of the user who created the account.
    #[serde(rename = "createdByUserID")]
    pub created_by_user_id: i64,
    /// Timestamp at which the account was created.
    #[serde(rename = "createdTime")]
    pub created_time: DateTime<Utc>,
    /// GSLO parameters governing mutability when markets are open vs halted.
    #[serde(rename = "guaranteedStopLossOrderParameters")]
    pub guaranteed_stop_loss_order_parameters: Option<GuaranteedStopLossOrderParameters>,
    /// Whether GSLOs are disabled, allowed, or required on this account.
    #[serde(rename = "guaranteedStopLossOrderMode")]
    pub guaranteed_stop_loss_order_mode: Option<GuaranteedStopLossOrderMode>,
    /// The mutability setting for GSLOs on this account (deprecated field).
    #[serde(rename = "guaranteedStopLossOrderMutability")]
    pub guaranteed_stop_loss_order_mutability: Option<GuaranteedStopLossOrderMutability>,
    /// Timestamp of the last P&L reset, if one has occurred.
    #[serde(rename = "resettablePLTime", deserialize_with = "deserialize_datetime")]
    pub resettable_pl_time: Option<DateTime<Utc>>,
    /// Margin rate expressed as a decimal (e.g. `0.05` for 5 % margin / 20:1 leverage).
    #[serde(rename = "marginRate")]
    pub margin_rate: Option<DecimalNumber>,
    /// Number of currently open trades.
    #[serde(rename = "openTradeCount")]
    pub open_trade_count: Option<i32>,
    /// Number of currently open positions.
    #[serde(rename = "openPositionCount")]
    pub open_position_count: Option<i32>,
    /// Number of pending orders.
    #[serde(rename = "pendingOrderCount")]
    pub pending_order_count: Option<i32>,
    /// Whether hedging (simultaneous long and short positions) is enabled.
    #[serde(rename = "hedgingEnabled")]
    pub hedging_enabled: Option<bool>,
    /// Unrealized profit/loss across all open trades, in home currency units.
    #[serde(rename = "unrealizedPL")]
    pub unrealized_pl: Option<AccountUnits>,
    /// Net asset value: `balance + unrealizedPL`.
    #[serde(rename = "NAV")]
    pub nav: Option<AccountUnits>,
    /// Margin currently consumed by open positions.
    #[serde(rename = "marginUsed")]
    pub margin_used: Option<AccountUnits>,
    /// Margin available to open new positions.
    #[serde(rename = "marginAvailable")]
    pub margin_available: Option<AccountUnits>,
    /// The value of all open positions expressed in home currency.
    #[serde(rename = "positionValue")]
    pub position_value: Option<AccountUnits>,
    /// Unrealized P&L used in the margin closeout calculation.
    #[serde(rename = "marginCloseoutUnrealizedPL")]
    pub margin_closeout_unrealized_pl: Option<AccountUnits>,
    /// NAV used in the margin closeout calculation.
    #[serde(rename = "marginCloseoutNAV")]
    pub margin_closeout_nav: Option<AccountUnits>,
    /// Margin used in the margin closeout calculation.
    #[serde(rename = "marginCloseoutMarginUsed")]
    pub margin_closeout_margin_used: Option<AccountUnits>,
    /// Ratio of margin closeout margin used to NAV (`marginCloseoutMarginUsed / marginCloseoutNAV`).
    #[serde(rename = "marginCloseoutPercent")]
    pub margin_closeout_percent: Option<DecimalNumber>,
    /// Value of all open positions in the margin closeout calculation.
    #[serde(rename = "marginCloseoutPositionValue")]
    pub margin_closeout_position_value: Option<DecimalNumber>,
    /// Maximum funds that can be withdrawn without margin impact.
    #[serde(rename = "withdrawalLimit")]
    pub withdrawal_limit: Option<AccountUnits>,
    /// Margin used as computed for the margin call trigger.
    #[serde(rename = "marginCallMarginUsed")]
    pub margin_call_margin_used: Option<AccountUnits>,
    /// Ratio of margin call margin used to NAV.
    #[serde(rename = "marginCallPercent")]
    pub margin_call_percent: Option<DecimalNumber>,
    /// Current cash balance of the account.
    pub balance: Option<AccountUnits>,
    /// Cumulative realized profit/loss.
    pub pl: Option<AccountUnits>,
    /// Realized P&L since the last reset.
    #[serde(rename = "resettablePL")]
    pub resettable_pl: Option<AccountUnits>,
    /// Cumulative financing paid/received.
    pub financing: Option<AccountUnits>,
    /// Cumulative commission paid.
    pub commission: Option<AccountUnits>,
    /// Cumulative dividend adjustment received.
    #[serde(rename = "dividendAdjustment")]
    pub dividend_adjustment: Option<AccountUnits>,
    /// Cumulative fees paid for guaranteed execution.
    #[serde(rename = "guaranteedExecutionFees")]
    pub guaranteed_execution_fees: Option<AccountUnits>,
    /// Timestamp at which the account entered margin call, if applicable.
    #[serde(rename = "marginCallEnterTime")]
    pub margin_call_enter_time: Option<DateTime<Utc>>,
    /// Number of times the margin call deadline has been extended.
    #[serde(rename = "marginCallExtensionCount")]
    pub margin_call_extension_count: Option<i32>,
    /// Timestamp of the most recent margin call deadline extension.
    #[serde(rename = "lastMarginCallExtensionTime")]
    pub last_margin_call_extension_time: Option<DateTime<Utc>>,
    /// ID of the most recent transaction applied to this account.
    #[serde(rename = "lastTransactionID")]
    pub last_transaction_id: Option<TransactionID>,
    /// All currently open trades.
    pub trades: Option<Vec<TradeSummary>>,
    /// All currently open positions.
    pub positions: Option<Vec<Position>>,
    /// All pending orders.
    pub orders: Option<Vec<Order>>,
}

// ---------------------------------------------------------------------------
// AccountProperties
// ---------------------------------------------------------------------------

/// Metadata about an account returned when listing all accounts accessible
/// by the authenticated user (`GET /v3/accounts`).
#[derive(Debug, Serialize, Deserialize)]
pub struct AccountProperties {
    /// The account's unique identifier.
    pub id: AccountID,
    /// The MT4 account number linked to this account, if any.
    #[serde(rename = "mt4AccountID")]
    pub mt4_account_id: Option<i32>,
    /// Arbitrary tags associated with the account.
    pub tags: Vec<String>,
}

// ---------------------------------------------------------------------------
// AccountSummary (same shape as Account but without open trades/positions/orders)
// ---------------------------------------------------------------------------

/// A condensed snapshot of an account's state, omitting the full lists of
/// open trades, positions, and orders.
///
/// Returned by the `GET /v3/accounts/{accountID}/summary` endpoint.
#[derive(Debug, Serialize, Deserialize)]
pub struct AccountSummary {
    /// The account's unique identifier.
    pub id: AccountID,
    /// An optional human-readable label for the account.
    pub alias: Option<String>,
    /// The home currency of the account (e.g. `"USD"`).
    pub currency: Currency,
    /// The numeric ID of the user who created the account.
    #[serde(rename = "createdByUserID")]
    pub created_by_user_id: i64,
    /// Timestamp at which the account was created.
    #[serde(rename = "createdTime")]
    pub created_time: DateTime<Utc>,
    /// GSLO parameters governing mutability when markets are open vs halted.
    #[serde(rename = "guaranteedStopLossOrderParameters")]
    pub guaranteed_stop_loss_order_parameters: Option<GuaranteedStopLossOrderParameters>,
    /// Whether GSLOs are disabled, allowed, or required on this account.
    #[serde(rename = "guaranteedStopLossOrderMode")]
    pub guaranteed_stop_loss_order_mode: Option<GuaranteedStopLossOrderMode>,
    /// The mutability setting for GSLOs on this account (deprecated field).
    #[serde(rename = "guaranteedStopLossOrderMutability")]
    pub guaranteed_stop_loss_order_mutability: Option<GuaranteedStopLossOrderMutability>,
    /// Timestamp of the last P&L reset, if one has occurred.
    #[serde(rename = "resettablePLTime", deserialize_with = "deserialize_datetime")]
    pub resettable_pl_time: Option<DateTime<Utc>>,
    /// Margin rate expressed as a decimal (e.g. `0.05` for 5 % margin / 20:1 leverage).
    #[serde(rename = "marginRate")]
    pub margin_rate: Option<DecimalNumber>,
    /// Number of currently open trades.
    #[serde(rename = "openTradeCount")]
    pub open_trade_count: Option<i32>,
    /// Number of currently open positions.
    #[serde(rename = "openPositionCount")]
    pub open_position_count: Option<i32>,
    /// Number of pending orders.
    #[serde(rename = "pendingOrderCount")]
    pub pending_order_count: Option<i32>,
    /// Whether hedging (simultaneous long and short positions) is enabled.
    #[serde(rename = "hedgingEnabled")]
    pub hedging_enabled: Option<bool>,
    /// Unrealized profit/loss across all open trades, in home currency units.
    #[serde(rename = "unrealizedPL")]
    pub unrealized_pl: Option<AccountUnits>,
    /// Net asset value: `balance + unrealizedPL`.
    #[serde(rename = "NAV")]
    pub nav: Option<AccountUnits>,
    /// Margin currently consumed by open positions.
    #[serde(rename = "marginUsed")]
    pub margin_used: Option<AccountUnits>,
    /// Margin available to open new positions.
    #[serde(rename = "marginAvailable")]
    pub margin_available: Option<AccountUnits>,
    /// The value of all open positions expressed in home currency.
    #[serde(rename = "positionValue")]
    pub position_value: Option<AccountUnits>,
    /// Unrealized P&L used in the margin closeout calculation.
    #[serde(rename = "marginCloseoutUnrealizedPL")]
    pub margin_closeout_unrealized_pl: Option<AccountUnits>,
    /// NAV used in the margin closeout calculation.
    #[serde(rename = "marginCloseoutNAV")]
    pub margin_closeout_nav: Option<AccountUnits>,
    /// Margin used in the margin closeout calculation.
    #[serde(rename = "marginCloseoutMarginUsed")]
    pub margin_closeout_margin_used: Option<AccountUnits>,
    /// Ratio of margin closeout margin used to NAV.
    #[serde(rename = "marginCloseoutPercent")]
    pub margin_closeout_percent: Option<DecimalNumber>,
    /// Value of all open positions in the margin closeout calculation.
    #[serde(rename = "marginCloseoutPositionValue")]
    pub margin_closeout_position_value: Option<DecimalNumber>,
    /// Maximum funds that can be withdrawn without margin impact.
    #[serde(rename = "withdrawalLimit")]
    pub withdrawal_limit: Option<AccountUnits>,
    /// Margin used as computed for the margin call trigger.
    #[serde(rename = "marginCallMarginUsed")]
    pub margin_call_margin_used: Option<AccountUnits>,
    /// Ratio of margin call margin used to NAV.
    #[serde(rename = "marginCallPercent")]
    pub margin_call_percent: Option<DecimalNumber>,
    /// Current cash balance of the account.
    pub balance: Option<AccountUnits>,
    /// Cumulative realized profit/loss.
    pub pl: Option<AccountUnits>,
    /// Realized P&L since the last reset.
    #[serde(rename = "resettablePL")]
    pub resettable_pl: Option<AccountUnits>,
    /// Cumulative financing paid/received.
    pub financing: Option<AccountUnits>,
    /// Cumulative commission paid.
    pub commission: Option<AccountUnits>,
    /// Cumulative dividend adjustment received.
    #[serde(rename = "dividendAdjustment")]
    pub dividend_adjustment: Option<AccountUnits>,
    /// Cumulative fees paid for guaranteed execution.
    #[serde(rename = "guaranteedExecutionFees")]
    pub guaranteed_execution_fees: Option<AccountUnits>,
    /// Timestamp at which the account entered margin call, if applicable.
    #[serde(rename = "marginCallEnterTime")]
    pub margin_call_enter_time: Option<DateTime<Utc>>,
    /// Number of times the margin call deadline has been extended.
    #[serde(rename = "marginCallExtensionCount")]
    pub margin_call_extension_count: Option<i32>,
    /// Timestamp of the most recent margin call deadline extension.
    #[serde(rename = "lastMarginCallExtensionTime")]
    pub last_margin_call_extension_time: Option<DateTime<Utc>>,
    /// ID of the most recent transaction applied to this account.
    #[serde(rename = "lastTransactionID")]
    pub last_transaction_id: Option<TransactionID>,
}

// ---------------------------------------------------------------------------
// AccountChanges
// ---------------------------------------------------------------------------

/// Describes every change applied to an account since a given transaction ID,
/// returned by `GET /v3/accounts/{accountID}/changes`.
#[derive(Debug, Serialize, Deserialize)]
pub struct AccountChanges {
    /// Orders that were created in the period.
    #[serde(rename = "ordersCreated")]
    pub orders_created: Option<Vec<Order>>,
    /// Orders that were cancelled in the period.
    #[serde(rename = "ordersCancelled")]
    pub orders_cancelled: Option<Vec<Order>>,
    /// Orders that were filled (executed) in the period.
    #[serde(rename = "ordersFilled")]
    pub orders_filled: Option<Vec<Order>>,
    /// Orders that were triggered in the period.
    #[serde(rename = "ordersTriggered")]
    pub orders_triggered: Option<Vec<Order>>,
    /// Trades that were newly opened in the period.
    #[serde(rename = "tradesOpened")]
    pub trades_opened: Option<Vec<TradeSummary>>,
    /// Trades that were partially closed (reduced) in the period.
    #[serde(rename = "tradesReduced")]
    pub trades_reduced: Option<Vec<TradeSummary>>,
    /// Trades that were fully closed in the period.
    #[serde(rename = "tradesClosed")]
    pub trades_closed: Option<Vec<TradeSummary>>,
    /// Positions affected by changes in the period.
    pub positions: Option<Vec<Position>>,
    /// Transactions generated in the period.
    ///
    /// Stored as raw JSON values because the OANDA API returns a polymorphic
    /// union of many transaction sub-types.
    pub transactions: Option<Vec<serde_json::Value>>,
}

// ---------------------------------------------------------------------------
// AccountChangesState
// ---------------------------------------------------------------------------

/// A polling-optimised snapshot of the dynamic (frequently changing) fields
/// of an account, returned alongside [`AccountChanges`] by the
/// `GET /v3/accounts/{accountID}/changes` endpoint.
///
/// Contains the same financial fields as [`Account`] but omits static
/// identity fields such as `id`, `currency`, and `createdTime`.
#[derive(Debug, Serialize, Deserialize)]
pub struct AccountChangesState {
    /// Unrealized profit/loss across all open trades.
    #[serde(rename = "unrealizedPL")]
    pub unrealized_pl: Option<AccountUnits>,
    /// Net asset value: `balance + unrealizedPL`.
    #[serde(rename = "NAV")]
    pub nav: Option<AccountUnits>,
    /// Margin currently consumed by open positions.
    #[serde(rename = "marginUsed")]
    pub margin_used: Option<AccountUnits>,
    /// Margin available to open new positions.
    #[serde(rename = "marginAvailable")]
    pub margin_available: Option<AccountUnits>,
    /// Value of all open positions in home currency.
    #[serde(rename = "positionValue")]
    pub position_value: Option<AccountUnits>,
    /// Unrealized P&L used in the margin closeout calculation.
    #[serde(rename = "marginCloseoutUnrealizedPL")]
    pub margin_closeout_unrealized_pl: Option<AccountUnits>,
    /// NAV used in the margin closeout calculation.
    #[serde(rename = "marginCloseoutNAV")]
    pub margin_closeout_nav: Option<AccountUnits>,
    /// Margin used in the margin closeout calculation.
    #[serde(rename = "marginCloseoutMarginUsed")]
    pub margin_closeout_margin_used: Option<AccountUnits>,
    /// Ratio of margin closeout margin used to NAV.
    #[serde(rename = "marginCloseoutPercent")]
    pub margin_closeout_percent: Option<DecimalNumber>,
    /// Value of all open positions in the margin closeout calculation.
    #[serde(rename = "marginCloseoutPositionValue")]
    pub margin_closeout_position_value: Option<DecimalNumber>,
    /// Maximum funds that can be withdrawn without margin impact.
    #[serde(rename = "withdrawalLimit")]
    pub withdrawal_limit: Option<AccountUnits>,
    /// Margin used as computed for the margin call trigger.
    #[serde(rename = "marginCallMarginUsed")]
    pub margin_call_margin_used: Option<AccountUnits>,
    /// Ratio of margin call margin used to NAV.
    #[serde(rename = "marginCallPercent")]
    pub margin_call_percent: Option<DecimalNumber>,
    /// Current cash balance of the account.
    pub balance: Option<AccountUnits>,
    /// Cumulative realized profit/loss.
    pub pl: Option<AccountUnits>,
    /// Realized P&L since the last reset.
    #[serde(rename = "resettablePL")]
    pub resettable_pl: Option<AccountUnits>,
    /// Cumulative financing paid/received.
    pub financing: Option<AccountUnits>,
    /// Cumulative commission paid.
    pub commission: Option<AccountUnits>,
    /// Cumulative dividend adjustment received.
    #[serde(rename = "dividendAdjustment")]
    pub dividend_adjustment: Option<AccountUnits>,
    /// Cumulative fees paid for guaranteed execution.
    #[serde(rename = "guaranteedExecutionFees")]
    pub guaranteed_execution_fees: Option<AccountUnits>,
    /// Timestamp at which the account entered margin call, if applicable.
    #[serde(rename = "marginCallEnterTime")]
    pub margin_call_enter_time: Option<DateTime<Utc>>,
    /// Number of times the margin call deadline has been extended.
    #[serde(rename = "marginCallExtensionCount")]
    pub margin_call_extension_count: Option<i32>,
    /// Timestamp of the most recent margin call deadline extension.
    #[serde(rename = "lastMarginCallExtensionTime")]
    pub last_margin_call_extension_time: Option<DateTime<Utc>>,
    /// Dynamic state of each pending order (price-dependent fields).
    pub orders: Option<Vec<DynamicOrderState>>,
    /// Dynamic state of each open trade (unrealized P&L, etc.).
    pub trades: Option<Vec<CalculatedTradeState>>,
    /// Dynamic state of each open position.
    pub positions: Option<Vec<CalculatedPositionState>>,
}

// ---------------------------------------------------------------------------
// GuaranteedStopLossOrderParameters
// ---------------------------------------------------------------------------

/// Configures mutability rules for Guaranteed Stop Loss Orders, separately
/// for when the market is open versus halted.
#[derive(Debug, Serialize, Deserialize)]
pub struct GuaranteedStopLossOrderParameters {
    /// Mutability rule that applies while the market is open.
    #[serde(rename = "mutabilityMarketOpen")]
    pub mutability_market_open: GuaranteedStopLossOrderMutability,
    /// Mutability rule that applies while the market is halted.
    #[serde(rename = "mutabilityMarketHalted")]
    pub mutability_market_halted: GuaranteedStopLossOrderMutability,
}

// ---------------------------------------------------------------------------
// Enums
// ---------------------------------------------------------------------------

/// Controls whether Guaranteed Stop Loss Orders (GSLOs) are available on an account.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum GuaranteedStopLossOrderMode {
    /// GSLOs are not available for this account.
    Disabled,
    /// GSLOs are available but not required.
    Allowed,
    /// Every trade on this account must have a GSLO attached.
    Required,
}

/// Describes how a Guaranteed Stop Loss Order may be changed after it is created.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum GuaranteedStopLossOrderMutability {
    /// The GSLO cannot be modified once placed.
    Fixed,
    /// The GSLO can be replaced with a new one.
    Replaceable,
    /// The GSLO can be cancelled.
    Cancelable,
    /// Only changes that widen the stop price are permitted.
    PriceWidenOnly,
}

/// Determines how financing (swap/rollover) is applied to open positions.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AccountFinancingMode {
    /// No financing charges or credits are applied.
    NoFinancing,
    /// Financing is calculated and applied every second.
    SecondBySecond,
    /// Financing is calculated and applied once per day.
    Daily,
}

/// Determines how the margin requirement for multiple positions on the same
/// instrument is aggregated.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PositionAggregationMode {
    /// Margin is the sum of the absolute values of all position sides.
    AbsoluteSum,
    /// Margin is determined by whichever side (long or short) is larger.
    MaximalSide,
    /// Margin is based on the net (long minus short) position size.
    NetSum,
}

// ---------------------------------------------------------------------------
// UserAttributes
// ---------------------------------------------------------------------------

/// Profile information about the OANDA user who owns an account,
/// returned by `GET /v3/users/{userSpecifier}`.
#[derive(Debug, Serialize, Deserialize)]
pub struct UserAttributes {
    /// OANDA's numeric user identifier.
    #[serde(rename = "userID")]
    pub user_id: i64,
    /// The user's login username.
    pub username: String,
    /// The user's title (e.g. `"Mr"`, `"Ms"`).
    pub title: String,
    /// The user's full name.
    pub name: String,
    /// The user's email address.
    pub email: String,
    /// Abbreviation of the OANDA division the account belongs to (e.g. `"001"`).
    #[serde(rename = "divisionAbbreviation")]
    pub division_abbreviation: String,
    /// BCP 47 language tag for the user's preferred language (e.g. `"en"`).
    #[serde(rename = "languageAbbreviation")]
    pub language_abbreviation: String,
    /// The user's home/base currency (e.g. `"USD"`).
    #[serde(rename = "homeCurrency")]
    pub home_currency: Currency,
}

// ---------------------------------------------------------------------------
// Response types
// ---------------------------------------------------------------------------

/// Response body for `GET /v3/accounts`.
#[derive(Debug, Serialize, Deserialize)]
pub struct ListAccountsResponse {
    /// The list of accounts accessible to the authenticated user.
    pub accounts: Vec<AccountProperties>,
}

/// Response body for `GET /v3/accounts/{accountID}`.
#[derive(Debug, Serialize, Deserialize)]
pub struct GetAccountDetailsResponse {
    /// The full account state, including all open trades, positions, and orders.
    pub account: Account,
    /// The ID of the most recent transaction on the account.
    #[serde(rename = "lastTransactionID")]
    pub last_transaction_id: TransactionID,
}

/// Response body for `GET /v3/accounts/{accountID}/summary`.
#[derive(Debug, Serialize, Deserialize)]
pub struct GetAccountSummaryResponse {
    /// A condensed snapshot of the account's state (no trade/position/order lists).
    pub account: AccountSummary,
    /// The ID of the most recent transaction on the account.
    #[serde(rename = "lastTransactionID")]
    pub last_transaction_id: TransactionID,
}

// ---------------------------------------------------------------------------
// Service
// ---------------------------------------------------------------------------

/// Provides access to the OANDA Account endpoints (`/v3/accounts/...`).
///
/// Obtain an instance via [`Client::account`](crate::client::Client::account).
pub struct AccountService<'a> {
    client: &'a Client,
}

impl<'a> AccountService<'a> {
    /// Creates a new `AccountService` bound to the given client.
    pub fn new(client: &'a Client) -> Self {
        Self { client }
    }

    /// Lists all accounts accessible to the authenticated user.
    ///
    /// Calls `GET /v3/accounts` and returns the parsed response on success,
    /// or an [`APIError`] if the server returns a non-200 status.
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

    pub async fn get_details(&self, account_id: &AccountID) -> Result<GetAccountDetailsResponse, APIError> {
        let url = self
            .client
            .base_url
            .join(format!("/v3/accounts/{}/", account_id).as_str())
            .unwrap();
        let http_req = Request::new(reqwest::Method::GET, url);
        let http_resp = self.client.http_client.execute(http_req).await?;
        match http_resp.status() {
            StatusCode::OK => {
                let resp = http_resp.json().await?;
                Ok(resp)
            }
            status => Err(APIError::ApiErrorResponse {
                status,
                message: http_resp.text().await?,
            }),
        }
    }

    pub async fn get_summary(&self, account_id: &AccountID) -> Result<GetAccountSummaryResponse, APIError> {
        let url = self
            .client
            .base_url
            .join(format!("/v3/accounts/{}/summary", account_id).as_str())
            .unwrap();
        let http_req = Request::new(reqwest::Method::GET, url);
        let http_resp = self.client.http_client.execute(http_req).await?;
        match http_resp.status() {
            StatusCode::OK => {
                let resp = http_resp.json().await?;
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

    #[tokio::test]
    async fn test_get_details() {
        let client = setup_test_client();
        let account_details = client
            .account()
            .get_details(&client.account_id.as_ref().unwrap())
            .await
            .unwrap();
        println!("{:#?}", account_details);
    }

    #[tokio::test]
    async fn test_get_summary() {
        let client = setup_test_client();
        let account_summary = client
            .account()
            .get_summary(&client.account_id.as_ref().unwrap())
            .await
            .unwrap();
        println!("{:#?}", account_summary);
    }
}
