use crate::client::request_option_setter;
use crate::errors::{APIError, ErrorResponse};
use crate::http::{decode_reject, decode_response, Connection};
use crate::instrument::InstrumentName;
use crate::order::{
    GuaranteedStopLossOrder, StopLossOrder, StopLossPrice, TakeProfitOrder, TimeInForce,
    TrailingStopLossOrder,
};
use crate::pricing::PriceValue;
use crate::primitives::DecimalNumber;
use crate::transaction::{
    AccountUnits, ClientExtensions, GuaranteedStopLossOrderRejectTransaction,
    GuaranteedStopLossOrderTransaction, MarketOrderRejectTransaction, MarketOrderTransaction,
    OrderCancelRejectTransaction, OrderCancelTransaction, OrderFillTransaction, OrderID,
    StopLossOrderRejectTransaction, StopLossOrderTransaction, TakeProfitOrderRejectTransaction,
    TakeProfitOrderTransaction, TradeClientExtensionsModifyRejectTransaction,
    TradeClientExtensionsModifyTransaction, TradeID, TrailingStopLossOrderRejectTransaction,
    TrailingStopLossOrderTransaction, TransactionID,
};
use chrono::{DateTime, Utc};
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use strum_macros::Display;
use thiserror::Error;
use url::Url;

/// A string that uniquely identifies a trade within an account.
///
/// Can be either an OANDA-assigned [`TradeID`] or a client-provided trade ID
/// prefixed with `"@"` (e.g. `"@my-trade-id"`).
pub type TradeSpecifier = String;

// ---------------------------------------------------------------------------
// Enums
// ---------------------------------------------------------------------------

/// The lifecycle state of a trade.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TradeState {
    /// The trade is currently open and has an active market position.
    Open,
    /// The trade has been fully closed and is no longer active.
    Closed,
    /// The trade has been flagged to close as soon as the instrument becomes
    /// tradeable again (e.g. after a market halt).
    CloseWhenTradeable,
}

/// Extends [`TradeState`] with an `All` variant for use as a filter when
/// listing trades.
#[derive(Debug, Serialize, Deserialize, Display)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[strum(serialize_all = "SCREAMING_SNAKE_CASE")]
pub enum TradeStateFilter {
    /// Return only open trades.
    Open,
    /// Return only closed trades.
    Closed,
    /// Return only trades pending close-when-tradeable.
    CloseWhenTradeable,
    /// Return trades in any state.
    All,
}

/// Categorises the profit/loss direction of a trade.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum TradePL {
    /// The trade has a positive (profitable) P&L.
    Positive,
    /// The trade has a negative (loss) P&L.
    Negative,
    /// The trade has neither a gain nor a loss (P&L is zero).
    Zero,
}

// ---------------------------------------------------------------------------
// Trade
// ---------------------------------------------------------------------------

/// A full representation of an open or closed trade, including its attached
/// stop-loss, take-profit, and trailing stop-loss orders.
///
/// Returned by `GET /v3/accounts/{accountID}/trades/{tradeSpecifier}` and
/// included in full account detail responses.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Trade {
    /// The trade's unique identifier assigned by OANDA.
    pub id: TradeID,
    /// The instrument being traded (e.g. `"EUR_USD"`).
    pub instrument: InstrumentName,
    /// The price at which the trade was opened.
    pub price: PriceValue,
    /// Timestamp at which the trade was opened.
    pub open_time: DateTime<Utc>,
    /// Current lifecycle state of the trade.
    pub state: TradeState,
    /// The number of units traded when the trade was first opened.
    /// Positive = long, negative = short.
    pub initial_units: DecimalNumber,
    /// The margin required to open the trade at its initial size, in home
    /// currency units. `None` when omitted by OANDA.
    pub initial_margin_required: Option<AccountUnits>,
    /// The number of units currently open. Decreases as partial closes occur.
    /// Zero once the trade is fully closed.
    pub current_units: DecimalNumber,
    /// Cumulative realised profit/loss from partial closes of this trade,
    /// in home currency units.
    #[serde(rename = "realizedPL")]
    pub realized_pl: AccountUnits,
    /// Current unrealised profit/loss based on the live market price,
    /// in home currency units.
    #[serde(rename = "unrealizedPL")]
    pub unrealized_pl: AccountUnits,
    /// Margin currently consumed by this trade's open units, in home currency, if reported.
    pub margin_used: Option<AccountUnits>,
    /// The average price at which units have been closed. `None` if no units
    /// have been closed yet.
    pub average_close_price: Option<PriceValue>,
    /// IDs of the transactions that fully or partially closed this trade.
    #[serde(rename = "closingTransactionIDs")]
    pub closing_transaction_ids: Option<Vec<TransactionID>>,
    /// Cumulative financing (swap/rollover) charges or credits applied to
    /// this trade, in home currency units.
    pub financing: AccountUnits,
    /// Cumulative dividend adjustment applied to this trade (for CFDs that
    /// pay dividends), in home currency units. `None` when omitted by OANDA.
    pub dividend_adjustment: Option<AccountUnits>,
    /// Timestamp at which the trade was fully closed. `None` while open.
    pub close_time: Option<DateTime<Utc>>,
    /// Optional client-supplied metadata attached to this trade.
    pub client_extensions: Option<ClientExtensions>,
    /// The take-profit order attached to this trade, if any.
    pub take_profit_order: Option<TakeProfitOrder>,
    /// The stop-loss order attached to this trade, if any.
    pub stop_loss_order: Option<StopLossOrder>,
    /// The guaranteed stop-loss order attached to this trade, if any.
    pub guaranteed_stop_loss_order: Option<GuaranteedStopLossOrder>,
    /// The trailing stop-loss order attached to this trade, if any.
    pub trailing_stop_loss_order: Option<TrailingStopLossOrder>,
}

// ---------------------------------------------------------------------------
// TradeSummary
// ---------------------------------------------------------------------------

/// A condensed representation of a trade that references attached orders by
/// ID instead of embedding the full order objects.
///
/// Returned in account-level list responses (e.g. inside [`Account`](crate::account::Account))
/// where embedding full order details would be redundant.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TradeSummary {
    /// The trade's unique identifier assigned by OANDA.
    pub id: TradeID,
    /// The instrument being traded (e.g. `"EUR_USD"`).
    pub instrument: InstrumentName,
    /// The price at which the trade was opened.
    pub price: PriceValue,
    /// Timestamp at which the trade was opened.
    pub open_time: DateTime<Utc>,
    /// Current lifecycle state of the trade.
    pub state: TradeState,
    /// The number of units traded when the trade was first opened.
    /// Positive = long, negative = short.
    pub initial_units: DecimalNumber,
    /// The margin required to open the trade at its initial size, in home
    /// currency units. `None` when omitted by OANDA.
    pub initial_margin_required: Option<AccountUnits>,
    /// The number of units currently open.
    pub current_units: DecimalNumber,
    /// Cumulative realised profit/loss from partial closes, in home currency units.
    #[serde(rename = "realizedPL")]
    pub realized_pl: AccountUnits,
    /// Current unrealised profit/loss, in home currency units.
    #[serde(rename = "unrealizedPL")]
    pub unrealized_pl: AccountUnits,
    /// Margin currently consumed by this trade's open units, in home currency, if reported.
    pub margin_used: Option<AccountUnits>,
    /// Average price at which units have been closed. `None` if no units have
    /// been closed yet.
    pub average_close_price: Option<PriceValue>,
    /// IDs of the transactions that fully or partially closed this trade.
    #[serde(rename = "closingTransactionIDs")]
    pub closing_transaction_ids: Option<Vec<TransactionID>>,
    /// Cumulative financing charges or credits applied to this trade, in home
    /// currency units.
    pub financing: AccountUnits,
    /// Cumulative dividend adjustment in home currency units, if reported.
    pub dividend_adjustment: Option<AccountUnits>,
    /// Timestamp at which the trade was fully closed. `None` while open.
    pub close_time: Option<DateTime<Utc>>,
    /// Optional client-supplied metadata attached to this trade.
    pub client_extensions: Option<ClientExtensions>,
    /// ID of the take-profit order attached to this trade, if any.
    #[serde(rename = "takeProfitOrderID")]
    pub take_profit_order_id: Option<OrderID>,
    /// ID of the stop-loss order attached to this trade, if any.
    #[serde(rename = "stopLossOrderID")]
    pub stop_loss_order_id: Option<OrderID>,
    /// ID of the guaranteed stop-loss order attached to this trade, if any.
    #[serde(rename = "guaranteedStopLossOrderID")]
    pub guaranteed_stop_loss_order_id: Option<OrderID>,
    /// ID of the trailing stop-loss order attached to this trade, if any.
    #[serde(rename = "trailingStopLossOrderID")]
    pub trailing_stop_loss_order_id: Option<OrderID>,
}

// ---------------------------------------------------------------------------
// CalculatedTradeState
// ---------------------------------------------------------------------------

/// The price-dependent (dynamic) state of an open trade, returned as part of
/// [`AccountChangesState`](crate::account::AccountChangesState).
///
/// Contains only the fields that change as the market moves; all static trade
/// fields are in [`Trade`] or [`TradeSummary`].
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CalculatedTradeState {
    /// The trade's unique identifier.
    pub id: TradeID,
    /// Current unrealised profit/loss based on the live market price,
    /// in home currency units.
    #[serde(rename = "unrealizedPL")]
    pub unrealized_pl: AccountUnits,
    /// Margin currently consumed by this trade's open units, in home currency units.
    pub margin_used: AccountUnits,
}

// ---------------------------------------------------------------------------
// Request types
// ---------------------------------------------------------------------------

/// Filters for one page of `GET /v3/accounts/{accountID}/trades`.
///
/// By default OANDA returns up to 50 open trades. Set `state` to
/// [`TradeStateFilter::All`] to include closed trades, and use `before_id` to
/// retrieve older pages while keeping the same filters.
///
/// ```no_run
/// use oanda_rust::{client::Client, errors::APIError};
/// use oanda_rust::trade::{ListTradesRequest, TradeStateFilter};
///
/// async fn example(client: &Client) -> Result<(), APIError> {
///     let page = client.trade().list(
///         ListTradesRequest::new()
///             .state(TradeStateFilter::All)
///             .count(100)
///             .before_id("6397".into())
///     ).await?;
///     println!("{:?}", page.trades);
///     Ok(())
/// }
/// ```
#[derive(Debug, Default)]
pub struct ListTradesRequest {
    ids: Vec<TradeID>,
    state: Option<TradeStateFilter>,
    instrument: Option<InstrumentName>,
    count: Option<u16>,
    before_id: Option<TradeID>,
}

impl ListTradesRequest {
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a trade ID to the requested selection.
    pub fn ids(mut self, id: TradeID) -> Self {
        self.ids.push(id);
        self
    }

    request_option_setter!(state, TradeStateFilter);
    request_option_setter!(instrument, InstrumentName);
    request_option_setter!(count, u16);
    request_option_setter!(before_id, TradeID);

    pub(crate) fn set_params(&self, url: &mut Url) -> Result<(), APIError> {
        if self.count.is_some_and(|count| !(1..=500).contains(&count)) {
            return Err(APIError::InvalidRequest(
                "count must be between 1 and 500".into(),
            ));
        }
        if !self.ids.is_empty() {
            url.query_pairs_mut()
                .append_pair("ids", &self.ids.join(","));
        }
        if let Some(state) = &self.state {
            url.query_pairs_mut()
                .append_pair("state", &state.to_string());
        }
        if let Some(instrument) = &self.instrument {
            url.query_pairs_mut().append_pair("instrument", instrument);
        }
        if let Some(count) = self.count {
            url.query_pairs_mut()
                .append_pair("count", &count.to_string());
        }
        if let Some(id) = &self.before_id {
            url.query_pairs_mut().append_pair("beforeID", id);
        }
        Ok(())
    }
}

/// Request body for `PUT /v3/accounts/{accountID}/trades/{tradeSpecifier}/close`.
///
/// Omitting `units` closes the entire trade. Supply a decimal string to
/// partially close only that many units.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct CloseTradeRequest {
    /// Number of units to close. `None` closes all open units. A decimal
    /// string (e.g. `"5000"`) partially closes the trade. Omitted when `None`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub units: Option<String>,
}

impl CloseTradeRequest {
    /// Creates a new request that will close all open units of the trade.
    pub fn new() -> CloseTradeRequest {
        Self::default()
    }

    request_option_setter!(units, String);
}

/// Take-profit fields to create or replace on an existing trade.
/// Unset fields are omitted: OANDA supplies defaults on creation and inherits
/// the existing order's values on replacement.
#[derive(Debug, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TakeProfitOrderUpdate {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub price: Option<PriceValue>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_in_force: Option<TimeInForce>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gtd_time: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_extensions: Option<ClientExtensions>,
}

impl TakeProfitOrderUpdate {
    pub fn new() -> Self {
        Self::default()
    }
    request_option_setter!(price, PriceValue);
    request_option_setter!(time_in_force, TimeInForce);
    request_option_setter!(gtd_time, DateTime<Utc>);
    request_option_setter!(client_extensions, ClientExtensions);
}

/// Regular or guaranteed stop-loss fields to create or replace on a trade.
/// Unset fields inherit existing values on replacement. Setting a distance
/// replaces an absolute price, and setting a price replaces a distance.
#[derive(Debug, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StopLossOrderUpdate {
    #[serde(flatten)]
    pub price: Option<StopLossPrice>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_in_force: Option<TimeInForce>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gtd_time: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_extensions: Option<ClientExtensions>,
}

impl StopLossOrderUpdate {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn price(mut self, price: PriceValue) -> Self {
        self.price = Some(StopLossPrice::Price(price));
        self
    }

    pub fn distance(mut self, distance: DecimalNumber) -> Self {
        self.price = Some(StopLossPrice::Distance(distance));
        self
    }

    request_option_setter!(time_in_force, TimeInForce);
    request_option_setter!(gtd_time, DateTime<Utc>);
    request_option_setter!(client_extensions, ClientExtensions);
}

/// Guaranteed stop-loss updates accept the same fields as regular stop losses.
pub type GuaranteedStopLossOrderUpdate = StopLossOrderUpdate;

/// Trailing-stop-loss fields to create or replace on an existing trade.
/// Unset fields inherit existing values on replacement.
#[derive(Debug, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TrailingStopLossOrderUpdate {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub distance: Option<DecimalNumber>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_in_force: Option<TimeInForce>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gtd_time: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_extensions: Option<ClientExtensions>,
}

impl TrailingStopLossOrderUpdate {
    pub fn new() -> Self {
        Self::default()
    }
    request_option_setter!(distance, DecimalNumber);
    request_option_setter!(time_in_force, TimeInForce);
    request_option_setter!(gtd_time, DateTime<Utc>);
    request_option_setter!(client_extensions, ClientExtensions);
}

/// Creates, replaces, or cancels dependent orders on an existing trade.
/// Each field distinguishes three operations: `None` leaves the order alone,
/// `Some(None)` cancels it, and `Some(Some(details))` creates or replaces it.
/// Builder setters accept `None` to cancel and `Some(details)` to set an order.
///
/// ```no_run
/// use oanda_rust::trade::{StopLossOrderUpdate, UpdateTradeOrdersRequest};
/// let request = UpdateTradeOrdersRequest::new()
///     .stop_loss(Some(StopLossOrderUpdate::new().price("1.0800".into())))
///     .take_profit(None); // Cancel the existing take profit.
/// ```
#[derive(Debug, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateTradeOrdersRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub take_profit: Option<Option<TakeProfitOrderUpdate>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop_loss: Option<Option<StopLossOrderUpdate>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trailing_stop_loss: Option<Option<TrailingStopLossOrderUpdate>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub guaranteed_stop_loss: Option<Option<GuaranteedStopLossOrderUpdate>>,
}

impl UpdateTradeOrdersRequest {
    pub fn new() -> Self {
        Self::default()
    }
    request_option_setter!(take_profit, Option<TakeProfitOrderUpdate>);
    request_option_setter!(stop_loss, Option<StopLossOrderUpdate>);
    request_option_setter!(trailing_stop_loss, Option<TrailingStopLossOrderUpdate>);
    request_option_setter!(guaranteed_stop_loss, Option<GuaranteedStopLossOrderUpdate>);
}

// ---------------------------------------------------------------------------
// Response types
// ---------------------------------------------------------------------------

/// Response body for `GET /v3/accounts/{accountID}/trades` and
/// `GET /v3/accounts/{accountID}/openTrades`.
#[derive(Debug, Serialize, Deserialize)]
pub struct ListTradesResponse {
    /// The list of trades matching the request.
    pub trades: Vec<Trade>,
    /// ID of the most recent transaction on the account.
    #[serde(rename = "lastTransactionID")]
    pub last_transaction_id: TransactionID,
}

/// Response body for `GET /v3/accounts/{accountID}/trades/{tradeSpecifier}`.
#[derive(Debug, Serialize, Deserialize)]
pub struct GetTradeDetailsResponse {
    /// The requested trade.
    pub trade: Trade,
    /// ID of the most recent transaction on the account.
    #[serde(rename = "lastTransactionID")]
    pub last_transaction_id: TransactionID,
}

/// Response body for `PUT /v3/accounts/{accountID}/trades/{tradeSpecifier}/close`
/// (HTTP 200).
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CloseTradeResponse {
    /// The market order transaction created to close the trade.
    pub order_create_transaction: Option<MarketOrderTransaction>,
    /// The order-fill transaction that recorded the trade closure.
    pub order_fill_transaction: Option<Box<OrderFillTransaction>>,
    /// The order-cancel transaction, present when the close order itself was
    /// cancelled (e.g. the trade was already closed).
    pub order_cancel_transaction: Option<OrderCancelTransaction>,
    /// IDs of all transactions related to this close request.
    #[serde(rename = "relatedTransactionIDs")]
    pub related_transaction_ids: Option<Vec<TransactionID>>,
    /// ID of the most recent transaction on the account after this request.
    #[serde(rename = "lastTransactionID")]
    pub last_transaction_id: Option<TransactionID>,
}

/// Error response body for a failed
/// `PUT /v3/accounts/{accountID}/trades/{tradeSpecifier}/close` (HTTP 400 or 404).
#[derive(Debug, Error, Serialize, Deserialize)]
#[error(
    "Trade close was rejected{}: {error_message}",
    crate::errors::code_suffix(.error_code.as_deref())
)]
#[serde(rename_all = "camelCase")]
pub struct CloseTradeErrorResponse {
    /// The transaction that recorded why the closing market order was rejected.
    /// `None` when OANDA omits it.
    pub order_reject_transaction: Option<MarketOrderRejectTransaction>,
    /// IDs of all transactions related to this request (HTTP 404 only).
    #[serde(rename = "relatedTransactionIDs")]
    pub related_transaction_ids: Option<Vec<TransactionID>>,
    /// ID of the most recent transaction on the account (HTTP 404 only).
    #[serde(rename = "lastTransactionID")]
    pub last_transaction_id: Option<TransactionID>,
    /// Machine-readable error code. `None` when OANDA omits it.
    pub error_code: Option<String>,
    /// Human-readable description of why the close was rejected.
    pub error_message: String,
}

/// Request body for
/// `PUT /v3/accounts/{accountID}/trades/{tradeSpecifier}/clientExtensions`.
///
/// Wraps a [`ClientExtensions`] value to match the JSON envelope the OANDA API
/// expects (`{"clientExtensions": {...}}`).
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateTradeClientExtensionsRequest {
    /// The new client extensions to apply to the trade.
    pub client_extensions: ClientExtensions,
}

/// Response body for a successful
/// `PUT /v3/accounts/{accountID}/trades/{tradeSpecifier}/clientExtensions`
/// (HTTP 200).
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateTradeClientExtensionsResponse {
    /// The transaction that recorded the client-extensions change on the trade.
    pub trade_client_extensions_modify_transaction: TradeClientExtensionsModifyTransaction,
    /// IDs of all transactions created by this request.
    #[serde(rename = "relatedTransactionIDs")]
    pub related_transaction_ids: Vec<TransactionID>,
    /// ID of the most recent transaction on the account after this request.
    #[serde(rename = "lastTransactionID")]
    pub last_transaction_id: TransactionID,
}

/// Error response body for a failed
/// `PUT /v3/accounts/{accountID}/trades/{tradeSpecifier}/clientExtensions`
/// (HTTP 400 or 404).
///
/// Returned when the update is rejected — for example, if the trade specifier
/// does not match any trade on the account.
#[derive(Debug, Error, Serialize, Deserialize)]
#[error(
    "Trade client extensions update error{}: {error_message}",
    crate::errors::code_suffix(.error_code.as_deref())
)]
#[serde(rename_all = "camelCase")]
pub struct UpdateTradeClientExtensionsErrorResponse {
    /// The reject transaction that recorded why the modification was refused.
    /// `None` when OANDA omits it.
    pub trade_client_extensions_modify_reject_transaction:
        Option<TradeClientExtensionsModifyRejectTransaction>,
    /// ID of the most recent transaction on the account. `None` when OANDA
    /// omits it.
    #[serde(rename = "lastTransactionID")]
    pub last_transaction_id: Option<TransactionID>,
    /// IDs of all transactions related to this (failed) request. `None` when
    /// OANDA omits them.
    #[serde(rename = "relatedTransactionIDs")]
    pub related_transaction_ids: Option<Vec<TransactionID>>,
    /// Machine-readable error code. `None` when OANDA omits it.
    pub error_code: Option<String>,
    /// Human-readable description of the error.
    pub error_message: String,
}

/// Transactions produced by a successful trade dependent-order update.
/// A transaction is absent when its corresponding action did not occur.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateTradeOrdersResponse {
    pub take_profit_order_cancel_transaction: Option<Box<OrderCancelTransaction>>,
    pub take_profit_order_transaction: Option<Box<TakeProfitOrderTransaction>>,
    pub take_profit_order_fill_transaction: Option<Box<OrderFillTransaction>>,
    pub take_profit_order_created_cancel_transaction: Option<Box<OrderCancelTransaction>>,
    pub stop_loss_order_cancel_transaction: Option<Box<OrderCancelTransaction>>,
    pub stop_loss_order_transaction: Option<Box<StopLossOrderTransaction>>,
    pub stop_loss_order_fill_transaction: Option<Box<OrderFillTransaction>>,
    pub stop_loss_order_created_cancel_transaction: Option<Box<OrderCancelTransaction>>,
    pub trailing_stop_loss_order_cancel_transaction: Option<Box<OrderCancelTransaction>>,
    pub trailing_stop_loss_order_transaction: Option<Box<TrailingStopLossOrderTransaction>>,
    pub guaranteed_stop_loss_order_cancel_transaction: Option<Box<OrderCancelTransaction>>,
    pub guaranteed_stop_loss_order_transaction: Option<Box<GuaranteedStopLossOrderTransaction>>,
    #[serde(rename = "relatedTransactionIDs")]
    pub related_transaction_ids: Vec<TransactionID>,
    #[serde(rename = "lastTransactionID")]
    pub last_transaction_id: TransactionID,
}

/// Rejection body for a trade dependent-order update (HTTP 400).
#[derive(Debug, Error, Serialize, Deserialize)]
#[error("Trade dependent orders update error{}: {error_message}", crate::errors::code_suffix(.error_code.as_deref()))]
#[serde(rename_all = "camelCase")]
pub struct UpdateTradeOrdersErrorResponse {
    pub take_profit_order_cancel_reject_transaction: Option<Box<OrderCancelRejectTransaction>>,
    pub take_profit_order_reject_transaction: Option<Box<TakeProfitOrderRejectTransaction>>,
    pub stop_loss_order_cancel_reject_transaction: Option<Box<OrderCancelRejectTransaction>>,
    pub stop_loss_order_reject_transaction: Option<Box<StopLossOrderRejectTransaction>>,
    pub trailing_stop_loss_order_cancel_reject_transaction:
        Option<Box<OrderCancelRejectTransaction>>,
    pub trailing_stop_loss_order_reject_transaction:
        Option<Box<TrailingStopLossOrderRejectTransaction>>,
    pub guaranteed_stop_loss_order_cancel_reject_transaction:
        Option<Box<OrderCancelRejectTransaction>>,
    pub guaranteed_stop_loss_order_reject_transaction:
        Option<Box<GuaranteedStopLossOrderRejectTransaction>>,
    #[serde(rename = "lastTransactionID")]
    pub last_transaction_id: Option<TransactionID>,
    #[serde(rename = "relatedTransactionIDs")]
    pub related_transaction_ids: Option<Vec<TransactionID>>,
    pub error_code: Option<String>,
    pub error_message: String,
}

// ---------------------------------------------------------------------------
// Service
// ---------------------------------------------------------------------------

/// Provides access to the OANDA Trade endpoints (`/v3/accounts/{id}/trades/...`).
///
/// Obtain an instance via [`Client::trade`](crate::client::Client::trade).
pub struct TradeService<'a> {
    connection: &'a Connection,
}

impl<'a> TradeService<'a> {
    /// Creates a new `TradeService` bound to the given client.
    pub(crate) fn new(connection: &'a Connection) -> Self {
        Self { connection }
    }

    /// Creates, replaces, or cancels this trade's dependent orders.
    /// Calls `PUT /v3/accounts/{accountID}/trades/{tradeSpecifier}/orders`.
    /// Omitted orders are unchanged; explicit null orders are cancelled.
    /// HTTP 400 is decoded as [`UpdateTradeOrdersErrorResponse`]. Other HTTP
    /// failures retain their status and request ID in [`APIError::Response`].
    pub async fn update_orders(
        &self,
        specifier: TradeSpecifier,
        req: UpdateTradeOrdersRequest,
    ) -> Result<UpdateTradeOrdersResponse, APIError> {
        let url = self
            .connection
            .account_url(&["trades", &specifier, "orders"])?;
        let response = self
            .connection
            .http_client
            .put(url)
            .json(&req)
            .send()
            .await?;
        decode_response(
            response,
            StatusCode::OK,
            Some(|status, body| match status {
                StatusCode::BAD_REQUEST => {
                    decode_reject(body, ErrorResponse::UpdateTradeOrdersError)
                }
                _ => None,
            }),
        )
        .await
    }

    /// Retrieves one page of trades matching `req`.
    ///
    /// Calls `GET /v3/accounts/{accountID}/trades`. The default request returns
    /// up to 50 open trades. Use [`ListTradesRequest::state`] to include closed
    /// trades and [`ListTradesRequest::before_id`] to retrieve older pages.
    /// This method does not fetch subsequent pages automatically.
    ///
    /// Returns [`APIError::InvalidRequest`] if no account ID is configured or
    /// `count` is outside `1..=500`.
    pub async fn list(&self, req: ListTradesRequest) -> Result<ListTradesResponse, APIError> {
        let mut url = self.connection.account_url(&["trades"])?;
        req.set_params(&mut url)?;
        self.connection.http_client.get_json(url).await
    }

    /// Lists all currently open trades on the account.
    ///
    /// Calls `GET /v3/accounts/{accountID}/openTrades`.
    ///
    /// Returns [`APIError::InvalidRequest`] if no account ID is configured.
    pub async fn list_open(&self) -> Result<ListTradesResponse, APIError> {
        let url = self.connection.account_url(&["openTrades"])?;
        self.connection.http_client.get_json(url).await
    }

    /// Returns the details of the trade identified by `specifier`.
    ///
    /// Calls `GET /v3/accounts/{accountID}/trades/{tradeSpecifier}`.
    ///
    /// Returns [`APIError::InvalidRequest`] if no account ID is configured.
    pub async fn get_details(
        &self,
        specifier: TradeSpecifier,
    ) -> Result<GetTradeDetailsResponse, APIError> {
        let url = self.connection.account_url(&["trades", &specifier])?;
        self.connection.http_client.get_json(url).await
    }

    /// Closes the trade identified by `specifier`, fully or partially.
    ///
    /// Calls `PUT /v3/accounts/{accountID}/trades/{tradeSpecifier}/close`.
    /// [`CloseTradeRequest::new`] closes all open units; set
    /// [`units`](CloseTradeRequest::units) to close only part of the trade.
    ///
    /// # Errors
    ///
    /// Returns [`APIError`] wrapping [`CloseTradeErrorResponse`] on HTTP 400
    /// (close rejected) or 404 (trade not found).
    ///
    /// Returns [`APIError::InvalidRequest`] if no account ID is configured.
    pub async fn close(
        &self,
        specifier: TradeSpecifier,
        req: CloseTradeRequest,
    ) -> Result<CloseTradeResponse, APIError> {
        let url = self
            .connection
            .account_url(&["trades", &specifier, "close"])?;
        let http_resp = self
            .connection
            .http_client
            .put(url)
            .json(&req)
            .send()
            .await?;
        decode_response::<CloseTradeResponse>(
            http_resp,
            StatusCode::OK,
            Some(|status, body| match status {
                StatusCode::BAD_REQUEST | StatusCode::NOT_FOUND => {
                    decode_reject(body, ErrorResponse::CloseTradeError)
                }
                _ => None,
            }),
        )
        .await
    }

    /// Replaces the client extensions on the trade identified by `specifier`.
    ///
    /// Calls `PUT /v3/accounts/{accountID}/trades/{tradeSpecifier}/clientExtensions`.
    ///
    /// Client extensions let you attach an optional client-assigned ID, tag, and
    /// free-text comment to a trade. Passing a new [`ClientExtensions`] value
    /// overwrites any previously stored extensions; fields left as `None` are
    /// cleared on the server.
    ///
    /// # Errors
    ///
    /// Returns [`APIError`] wrapping [`UpdateTradeClientExtensionsErrorResponse`]
    /// on HTTP 400 (bad request) or 404 (trade not found).
    ///
    /// Returns [`APIError::InvalidRequest`] if no account ID is configured.
    pub async fn update_client_extensions(
        &self,
        specifier: TradeSpecifier,
        client_extensions: ClientExtensions,
    ) -> Result<UpdateTradeClientExtensionsResponse, APIError> {
        let url = self
            .connection
            .account_url(&["trades", &specifier, "clientExtensions"])?;
        let req = UpdateTradeClientExtensionsRequest { client_extensions };
        let http_resp = self
            .connection
            .http_client
            .put(url)
            .json(&req)
            .send()
            .await?;
        decode_response::<UpdateTradeClientExtensionsResponse>(
            http_resp,
            StatusCode::OK,
            Some(|status, body| match status {
                StatusCode::BAD_REQUEST | StatusCode::NOT_FOUND => {
                    decode_reject(body, ErrorResponse::UpdateTradeClientExtensionsError)
                }
                _ => None,
            }),
        )
        .await
    }
}
