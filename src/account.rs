use crate::client::Client;
use crate::errors::{APIError, ErrorResponse};
use crate::http::decode_response;
use crate::instrument::Instrument;
use crate::order::{DynamicOrderState, Order};
use crate::position::{CalculatedPositionState, Position};
use crate::primitives::{deserialize_datetime, Currency, DecimalNumber};
use crate::request_option_setter;
use crate::trade::{CalculatedTradeState, TradeSummary};
use crate::transaction::{
    AccountUnits, ClientConfigureRejectTransaction, ClientConfigureTransaction, Transaction,
    TransactionID,
};
use chrono::{DateTime, Utc};
use reqwest::{Request, StatusCode};
use serde::{Deserialize, Serialize};
use thiserror::Error;

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
#[serde(rename_all = "camelCase")]
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
    pub created_time: DateTime<Utc>,
    /// GSLO parameters governing mutability when markets are open vs halted.
    pub guaranteed_stop_loss_order_parameters: Option<GuaranteedStopLossOrderParameters>,
    /// Whether GSLOs are disabled, allowed, or required on this account.
    pub guaranteed_stop_loss_order_mode: Option<GuaranteedStopLossOrderMode>,
    /// The mutability setting for GSLOs on this account (deprecated field).
    pub guaranteed_stop_loss_order_mutability: Option<GuaranteedStopLossOrderMutability>,
    /// Timestamp of the last P&L reset, if one has occurred.
    #[serde(rename = "resettablePLTime", deserialize_with = "deserialize_datetime")]
    pub resettable_pl_time: Option<DateTime<Utc>>,
    /// Margin rate expressed as a decimal (e.g. `0.05` for 5 % margin / 20:1 leverage).
    pub margin_rate: Option<DecimalNumber>,
    /// Number of currently open trades.
    pub open_trade_count: Option<i32>,
    /// Number of currently open positions.
    pub open_position_count: Option<i32>,
    /// Number of pending orders.
    pub pending_order_count: Option<i32>,
    /// Whether hedging (simultaneous long and short positions) is enabled.
    pub hedging_enabled: Option<bool>,
    /// Unrealized profit/loss across all open trades, in home currency units.
    #[serde(rename = "unrealizedPL")]
    pub unrealized_pl: Option<AccountUnits>,
    /// Net asset value: `balance + unrealizedPL`.
    #[serde(rename = "NAV")]
    pub nav: Option<AccountUnits>,
    /// Margin currently consumed by open positions.
    pub margin_used: Option<AccountUnits>,
    /// Margin available to open new positions.
    pub margin_available: Option<AccountUnits>,
    /// The value of all open positions expressed in home currency.
    pub position_value: Option<AccountUnits>,
    /// Unrealized P&L used in the margin closeout calculation.
    #[serde(rename = "marginCloseoutUnrealizedPL")]
    pub margin_closeout_unrealized_pl: Option<AccountUnits>,
    /// NAV used in the margin closeout calculation.
    #[serde(rename = "marginCloseoutNAV")]
    pub margin_closeout_nav: Option<AccountUnits>,
    /// Margin used in the margin closeout calculation.
    pub margin_closeout_margin_used: Option<AccountUnits>,
    /// Ratio of margin closeout margin used to NAV (`marginCloseoutMarginUsed / marginCloseoutNAV`).
    pub margin_closeout_percent: Option<DecimalNumber>,
    /// Value of all open positions in the margin closeout calculation.
    pub margin_closeout_position_value: Option<DecimalNumber>,
    /// Maximum funds that can be withdrawn without margin impact.
    pub withdrawal_limit: Option<AccountUnits>,
    /// Margin used as computed for the margin call trigger.
    pub margin_call_margin_used: Option<AccountUnits>,
    /// Ratio of margin call margin used to NAV.
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
    pub dividend_adjustment: Option<AccountUnits>,
    /// Cumulative fees paid for guaranteed execution.
    pub guaranteed_execution_fees: Option<AccountUnits>,
    /// Timestamp at which the account entered margin call, if applicable.
    pub margin_call_enter_time: Option<DateTime<Utc>>,
    /// Number of times the margin call deadline has been extended.
    pub margin_call_extension_count: Option<i32>,
    /// Timestamp of the most recent margin call deadline extension.
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
#[serde(rename_all = "camelCase")]
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
    pub created_time: DateTime<Utc>,
    /// GSLO parameters governing mutability when markets are open vs halted.
    pub guaranteed_stop_loss_order_parameters: Option<GuaranteedStopLossOrderParameters>,
    /// Whether GSLOs are disabled, allowed, or required on this account.
    pub guaranteed_stop_loss_order_mode: Option<GuaranteedStopLossOrderMode>,
    /// The mutability setting for GSLOs on this account (deprecated field).
    pub guaranteed_stop_loss_order_mutability: Option<GuaranteedStopLossOrderMutability>,
    /// Timestamp of the last P&L reset, if one has occurred.
    #[serde(rename = "resettablePLTime", deserialize_with = "deserialize_datetime")]
    pub resettable_pl_time: Option<DateTime<Utc>>,
    /// Margin rate expressed as a decimal (e.g. `0.05` for 5 % margin / 20:1 leverage).
    pub margin_rate: Option<DecimalNumber>,
    /// Number of currently open trades.
    pub open_trade_count: Option<i32>,
    /// Number of currently open positions.
    pub open_position_count: Option<i32>,
    /// Number of pending orders.
    pub pending_order_count: Option<i32>,
    /// Whether hedging (simultaneous long and short positions) is enabled.
    pub hedging_enabled: Option<bool>,
    /// Unrealized profit/loss across all open trades, in home currency units.
    #[serde(rename = "unrealizedPL")]
    pub unrealized_pl: Option<AccountUnits>,
    /// Net asset value: `balance + unrealizedPL`.
    #[serde(rename = "NAV")]
    pub nav: Option<AccountUnits>,
    /// Margin currently consumed by open positions.
    pub margin_used: Option<AccountUnits>,
    /// Margin available to open new positions.
    pub margin_available: Option<AccountUnits>,
    /// The value of all open positions expressed in home currency.
    pub position_value: Option<AccountUnits>,
    /// Unrealized P&L used in the margin closeout calculation.
    #[serde(rename = "marginCloseoutUnrealizedPL")]
    pub margin_closeout_unrealized_pl: Option<AccountUnits>,
    /// NAV used in the margin closeout calculation.
    #[serde(rename = "marginCloseoutNAV")]
    pub margin_closeout_nav: Option<AccountUnits>,
    /// Margin used in the margin closeout calculation.
    pub margin_closeout_margin_used: Option<AccountUnits>,
    /// Ratio of margin closeout margin used to NAV.
    pub margin_closeout_percent: Option<DecimalNumber>,
    /// Value of all open positions in the margin closeout calculation.
    pub margin_closeout_position_value: Option<DecimalNumber>,
    /// Maximum funds that can be withdrawn without margin impact.
    pub withdrawal_limit: Option<AccountUnits>,
    /// Margin used as computed for the margin call trigger.
    pub margin_call_margin_used: Option<AccountUnits>,
    /// Ratio of margin call margin used to NAV.
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
    pub dividend_adjustment: Option<AccountUnits>,
    /// Cumulative fees paid for guaranteed execution.
    pub guaranteed_execution_fees: Option<AccountUnits>,
    /// Timestamp at which the account entered margin call, if applicable.
    pub margin_call_enter_time: Option<DateTime<Utc>>,
    /// Number of times the margin call deadline has been extended.
    pub margin_call_extension_count: Option<i32>,
    /// Timestamp of the most recent margin call deadline extension.
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
#[serde(rename_all = "camelCase")]
pub struct AccountChanges {
    /// Orders that were created in the period.
    pub orders_created: Option<Vec<Order>>,
    /// Orders that were cancelled in the period.
    pub orders_cancelled: Option<Vec<Order>>,
    /// Orders that were filled (executed) in the period.
    pub orders_filled: Option<Vec<Order>>,
    /// Orders that were triggered in the period.
    pub orders_triggered: Option<Vec<Order>>,
    /// Trades that were newly opened in the period.
    pub trades_opened: Option<Vec<TradeSummary>>,
    /// Trades that were partially closed (reduced) in the period.
    pub trades_reduced: Option<Vec<TradeSummary>>,
    /// Trades that were fully closed in the period.
    pub trades_closed: Option<Vec<TradeSummary>>,
    /// Positions affected by changes in the period.
    pub positions: Option<Vec<Position>>,
    /// Transactions generated in the period.
    pub transactions: Option<Vec<Transaction>>,
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
#[serde(rename_all = "camelCase")]
pub struct AccountChangesState {
    /// Unrealized profit/loss across all open trades.
    #[serde(rename = "unrealizedPL")]
    pub unrealized_pl: Option<AccountUnits>,
    /// Net asset value: `balance + unrealizedPL`.
    #[serde(rename = "NAV")]
    pub nav: Option<AccountUnits>,
    /// Margin currently consumed by open positions.
    pub margin_used: Option<AccountUnits>,
    /// Margin available to open new positions.
    pub margin_available: Option<AccountUnits>,
    /// Value of all open positions in home currency.
    pub position_value: Option<AccountUnits>,
    /// Unrealized P&L used in the margin closeout calculation.
    #[serde(rename = "marginCloseoutUnrealizedPL")]
    pub margin_closeout_unrealized_pl: Option<AccountUnits>,
    /// NAV used in the margin closeout calculation.
    #[serde(rename = "marginCloseoutNAV")]
    pub margin_closeout_nav: Option<AccountUnits>,
    /// Margin used in the margin closeout calculation.
    pub margin_closeout_margin_used: Option<AccountUnits>,
    /// Ratio of margin closeout margin used to NAV.
    pub margin_closeout_percent: Option<DecimalNumber>,
    /// Value of all open positions in the margin closeout calculation.
    pub margin_closeout_position_value: Option<DecimalNumber>,
    /// Maximum funds that can be withdrawn without margin impact.
    pub withdrawal_limit: Option<AccountUnits>,
    /// Margin used as computed for the margin call trigger.
    pub margin_call_margin_used: Option<AccountUnits>,
    /// Ratio of margin call margin used to NAV.
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
    pub dividend_adjustment: Option<AccountUnits>,
    /// Cumulative fees paid for guaranteed execution.
    pub guaranteed_execution_fees: Option<AccountUnits>,
    /// Timestamp at which the account entered margin call, if applicable.
    pub margin_call_enter_time: Option<DateTime<Utc>>,
    /// Number of times the margin call deadline has been extended.
    pub margin_call_extension_count: Option<i32>,
    /// Timestamp of the most recent margin call deadline extension.
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
#[serde(rename_all = "camelCase")]
pub struct GuaranteedStopLossOrderParameters {
    /// Mutability rule that applies while the market is open.
    pub mutability_market_open: GuaranteedStopLossOrderMutability,
    /// Mutability rule that applies while the market is halted.
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
#[serde(rename_all = "camelCase")]
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
    pub division_abbreviation: String,
    /// BCP 47 language tag for the user's preferred language (e.g. `"en"`).
    pub language_abbreviation: String,
    /// The user's home/base currency (e.g. `"USD"`).
    pub home_currency: Currency,
}

// ---------------------------------------------------------------------------
// Request types
// ---------------------------------------------------------------------------

/// Request body for `PATCH /v3/accounts/{accountID}/configuration`.
///
/// At least one field must be set; fields left as `None` are not sent and
/// therefore not changed on the server.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConfigureAccountRequest {
    /// New display name for the account.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub alias: Option<String>,
    /// New margin rate expressed as a decimal (e.g. `"0.05"` for 5 % margin / 20:1 leverage).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub margin_rate: Option<DecimalNumber>,
}

impl Default for ConfigureAccountRequest {
    fn default() -> Self {
        Self::new()
    }
}

impl ConfigureAccountRequest {
    /// Creates a new request with no fields set.
    pub fn new() -> Self {
        ConfigureAccountRequest {
            alias: None,
            margin_rate: None,
        }
    }

    request_option_setter!(alias, String);
    request_option_setter!(margin_rate, DecimalNumber);
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

/// Response body for `GET /v3/accounts/{accountID}/instruments`.
#[derive(Debug, Serialize, Deserialize)]
pub struct GetInstrumentsResponse {
    /// The list of tradeable instruments for the account.
    pub instruments: Vec<Instrument>,
    /// ID of the most recent transaction on the account.
    #[serde(rename = "lastTransactionID")]
    pub last_transaction_id: TransactionID,
}

/// Response body for `PATCH /v3/accounts/{accountID}/configuration` (HTTP 200).
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConfigureAccountResponse {
    /// The transaction that recorded the configuration change.
    pub client_configure_transaction: ClientConfigureTransaction,
    /// ID of the most recent transaction on the account.
    #[serde(rename = "lastTransactionID")]
    pub last_transaction_id: TransactionID,
}

/// Error response body for `PATCH /v3/accounts/{accountID}/configuration`
/// (HTTP 400 or 403).
#[derive(Debug, Error, Serialize, Deserialize)]
#[error("Configure account error: {error_message}")]
#[serde(rename_all = "camelCase")]
pub struct ConfigureAccountErrorResponse {
    /// The reject transaction that recorded the failed configuration attempt.
    pub client_configure_reject_transaction: ClientConfigureRejectTransaction,
    /// ID of the most recent transaction on the account.
    #[serde(rename = "lastTransactionID")]
    pub last_transaction_id: TransactionID,
    /// A machine-readable error code, if provided.
    pub error_code: Option<String>,
    /// A human-readable description of why the request was rejected.
    pub error_message: String,
}

/// Response body for `GET /v3/accounts/{accountID}/changes` (HTTP 200).
#[derive(Debug, Serialize, Deserialize)]
pub struct GetAccountChangesResponse {
    /// All changes to the account since the requested transaction.
    pub changes: AccountChanges,
    /// Current dynamic state of the account (prices, margins, P&L).
    pub state: AccountChangesState,
    /// ID of the most recent transaction on the account.
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
    pub(crate) fn new(client: &'a Client) -> Self {
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
        decode_response::<ListAccountsResponse>(http_resp, StatusCode::OK, None).await
    }

    /// Returns the full details of the specified account, including all open
    /// trades, positions, and pending orders.
    ///
    /// Calls `GET /v3/accounts/{accountID}`.
    pub async fn get_details(
        &self,
        account_id: &AccountID,
    ) -> Result<GetAccountDetailsResponse, APIError> {
        let url = crate::http::account_url(&self.client.base_url, Some(account_id), "")?;
        let http_req = Request::new(reqwest::Method::GET, url);
        let http_resp = self.client.http_client.execute(http_req).await?;
        decode_response::<GetAccountDetailsResponse>(http_resp, StatusCode::OK, None).await
    }

    /// Returns a condensed snapshot of the specified account's state.
    ///
    /// Calls `GET /v3/accounts/{accountID}/summary`. Unlike [`get_details`](Self::get_details),
    /// the response does not include the full lists of open trades, positions, or orders.
    pub async fn get_summary(
        &self,
        account_id: &AccountID,
    ) -> Result<GetAccountSummaryResponse, APIError> {
        let url = crate::http::account_url(&self.client.base_url, Some(account_id), "summary")?;
        let http_req = Request::new(reqwest::Method::GET, url);
        let http_resp = self.client.http_client.execute(http_req).await?;
        decode_response::<GetAccountSummaryResponse>(http_resp, StatusCode::OK, None).await
    }

    /// Returns the list of tradeable instruments for the given account.
    ///
    /// Calls `GET /v3/accounts/{accountID}/instruments`.
    /// Pass `instruments` to filter by a specific set of instrument names;
    /// `None` returns all available instruments.
    pub async fn get_instruments(
        &self,
        account_id: &AccountID,
        instruments: Option<Vec<String>>,
    ) -> Result<GetInstrumentsResponse, APIError> {
        let mut url =
            crate::http::account_url(&self.client.base_url, Some(account_id), "instruments")?;
        if let Some(names) = instruments {
            url.query_pairs_mut()
                .append_pair("instruments", &names.join(","));
        }
        let http_req = Request::new(reqwest::Method::GET, url);
        let http_resp = self.client.http_client.execute(http_req).await?;
        decode_response::<GetInstrumentsResponse>(http_resp, StatusCode::OK, None).await
    }

    /// Updates the account's alias and/or margin rate.
    ///
    /// Calls `PATCH /v3/accounts/{accountID}/configuration`.
    /// Returns a [`ConfigureAccountErrorResponse`] wrapped in [`APIError`]
    /// on HTTP 400 or 403.
    pub async fn configure(
        &self,
        account_id: &AccountID,
        req: ConfigureAccountRequest,
    ) -> Result<ConfigureAccountResponse, APIError> {
        let url =
            crate::http::account_url(&self.client.base_url, Some(account_id), "configuration")?;
        let http_resp = self.client.http_client.patch(url).json(&req).send().await?;
        decode_response::<ConfigureAccountResponse>(
            http_resp,
            StatusCode::OK,
            Some(|status, body| match status {
                StatusCode::BAD_REQUEST | StatusCode::FORBIDDEN => {
                    serde_json::from_slice::<ConfigureAccountErrorResponse>(body)
                        .ok()
                        .map(ErrorResponse::ConfigureAccountError)
                }
                _ => None,
            }),
        )
        .await
    }

    /// Returns all changes to the account since the given transaction ID,
    /// along with the current dynamic account state.
    ///
    /// Calls `GET /v3/accounts/{accountID}/changes?sinceTransactionID={id}`.
    pub async fn get_changes(
        &self,
        account_id: &AccountID,
        since_transaction_id: TransactionID,
    ) -> Result<GetAccountChangesResponse, APIError> {
        let mut url = crate::http::account_url(&self.client.base_url, Some(account_id), "changes")?;
        url.query_pairs_mut()
            .append_pair("sinceTransactionID", &since_transaction_id);
        let http_req = Request::new(reqwest::Method::GET, url);
        let http_resp = self.client.http_client.execute(http_req).await?;
        decode_response::<GetAccountChangesResponse>(http_resp, StatusCode::OK, None).await
    }
}
