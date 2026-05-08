use crate::client::Client;
use crate::errors::{APIError, CommonErrorResponse, ErrorResponse};
use crate::instrument::InstrumentName;
use crate::order::{
    GuaranteedStopLossOrder, OrderID, StopLossOrder, TakeProfitOrder, TrailingStopLossOrder,
};
use crate::pricing::PriceValue;
use crate::primitives::DecimalNumber;
use crate::transaction::{
    AccountUnits, ClientExtensions, TradeClientExtensionsModifyRejectTransaction,
    TradeClientExtensionsModifyTransaction, TradeID, TransactionID,
};
use crate::{handle_response, request_option_setter};
use chrono::{DateTime, Utc};
use reqwest::{Request, StatusCode};
use serde::{Deserialize, Serialize};
use thiserror::Error;

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
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
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
pub struct Trade {
    /// The trade's unique identifier assigned by OANDA.
    pub id: TradeID,
    /// The instrument being traded (e.g. `"EUR_USD"`).
    pub instrument: InstrumentName,
    /// The price at which the trade was opened.
    pub price: PriceValue,
    /// Timestamp at which the trade was opened.
    #[serde(rename = "openTime")]
    pub open_time: DateTime<Utc>,
    /// Current lifecycle state of the trade.
    pub state: TradeState,
    /// The number of units traded when the trade was first opened.
    /// Positive = long, negative = short.
    #[serde(rename = "initialUnits")]
    pub initial_units: DecimalNumber,
    /// The margin required to open the trade at its initial size, in home
    /// currency units.
    #[serde(rename = "initialMarginRequired")]
    pub initial_margin_required: AccountUnits,
    /// The number of units currently open. Decreases as partial closes occur.
    /// Zero once the trade is fully closed.
    #[serde(rename = "currentUnits")]
    pub current_units: DecimalNumber,
    /// Cumulative realised profit/loss from partial closes of this trade,
    /// in home currency units.
    #[serde(rename = "realizedPL")]
    pub realized_pl: AccountUnits,
    /// Current unrealised profit/loss based on the live market price,
    /// in home currency units.
    #[serde(rename = "unrealizedPL")]
    pub unrealized_pl: AccountUnits,
    /// Margin currently consumed by this trade's open units, in home currency units.
    #[serde(rename = "marginUsed")]
    pub margin_used: AccountUnits,
    /// The average price at which units have been closed. `None` if no units
    /// have been closed yet.
    #[serde(rename = "averageClosePrice")]
    pub average_close_price: Option<PriceValue>,
    /// IDs of the transactions that fully or partially closed this trade.
    #[serde(rename = "closingTransactionIDs")]
    pub closing_transaction_ids: Option<Vec<TransactionID>>,
    /// Cumulative financing (swap/rollover) charges or credits applied to
    /// this trade, in home currency units.
    pub financing: AccountUnits,
    /// Cumulative dividend adjustment applied to this trade (for CFDs that
    /// pay dividends), in home currency units.
    #[serde(rename = "dividendAdjustment")]
    pub dividend_adjustment: AccountUnits,
    /// Timestamp at which the trade was fully closed. `None` while open.
    #[serde(rename = "closeTime")]
    pub close_time: Option<DateTime<Utc>>,
    /// Optional client-supplied metadata attached to this trade.
    #[serde(rename = "clientExtensions")]
    pub client_extensions: Option<ClientExtensions>,
    /// The take-profit order attached to this trade, if any.
    #[serde(rename = "takeProfitOrder")]
    pub take_profit_order: Option<TakeProfitOrder>,
    /// The stop-loss order attached to this trade, if any.
    #[serde(rename = "stopLossOrder")]
    pub stop_loss_order: Option<StopLossOrder>,
    /// The guaranteed stop-loss order attached to this trade, if any.
    #[serde(rename = "guaranteedStopLossOrder")]
    pub guaranteed_stop_loss_order: Option<GuaranteedStopLossOrder>,
    /// The trailing stop-loss order attached to this trade, if any.
    #[serde(rename = "trailingStopLossOrder")]
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
pub struct TradeSummary {
    /// The trade's unique identifier assigned by OANDA.
    pub id: TradeID,
    /// The instrument being traded (e.g. `"EUR_USD"`).
    pub instrument: InstrumentName,
    /// The price at which the trade was opened.
    pub price: PriceValue,
    /// Timestamp at which the trade was opened.
    #[serde(rename = "openTime")]
    pub open_time: DateTime<Utc>,
    /// Current lifecycle state of the trade.
    pub state: TradeState,
    /// The number of units traded when the trade was first opened.
    /// Positive = long, negative = short.
    #[serde(rename = "initialUnits")]
    pub initial_units: DecimalNumber,
    /// The margin required to open the trade at its initial size, in home
    /// currency units.
    #[serde(rename = "initialMarginRequired")]
    pub initial_margin_required: AccountUnits,
    /// The number of units currently open.
    #[serde(rename = "currentUnits")]
    pub current_units: DecimalNumber,
    /// Cumulative realised profit/loss from partial closes, in home currency units.
    #[serde(rename = "realizedPL")]
    pub realized_pl: AccountUnits,
    /// Current unrealised profit/loss, in home currency units.
    #[serde(rename = "unrealizedPL")]
    pub unrealized_pl: AccountUnits,
    /// Margin currently consumed by this trade's open units, in home currency units.
    #[serde(rename = "marginUsed")]
    pub margin_used: AccountUnits,
    /// Average price at which units have been closed. `None` if no units have
    /// been closed yet.
    #[serde(rename = "averageClosePrice")]
    pub average_close_price: Option<PriceValue>,
    /// IDs of the transactions that fully or partially closed this trade.
    #[serde(rename = "closingTransactionIDs")]
    pub closing_transaction_ids: Option<Vec<TransactionID>>,
    /// Cumulative financing charges or credits applied to this trade, in home
    /// currency units.
    pub financing: AccountUnits,
    /// Cumulative dividend adjustment applied to this trade, in home currency units.
    #[serde(rename = "dividendAdjustment")]
    pub dividend_adjustment: AccountUnits,
    /// Timestamp at which the trade was fully closed. `None` while open.
    #[serde(rename = "closeTime")]
    pub close_time: Option<DateTime<Utc>>,
    /// Optional client-supplied metadata attached to this trade.
    #[serde(rename = "clientExtensions")]
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
pub struct CalculatedTradeState {
    /// The trade's unique identifier.
    pub id: TradeID,
    /// Current unrealised profit/loss based on the live market price,
    /// in home currency units.
    #[serde(rename = "unrealizedPL")]
    pub unrealized_pl: AccountUnits,
    /// Margin currently consumed by this trade's open units, in home currency units.
    #[serde(rename = "marginUsed")]
    pub margin_used: AccountUnits,
}

// ---------------------------------------------------------------------------
// Request types
// ---------------------------------------------------------------------------

/// Request body for `PUT /v3/accounts/{accountID}/trades/{tradeSpecifier}/close`.
///
/// Omitting `units` closes the entire trade. Supply a decimal string to
/// partially close only that many units.
#[derive(Debug, Serialize, Deserialize)]
pub struct CloseTradeRequest {
    /// Number of units to close. `None` closes all open units. A decimal
    /// string (e.g. `"5000"`) partially closes the trade. Omitted when `None`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub units: Option<String>,
}

impl CloseTradeRequest {
    /// Creates a new request that will close all open units of the trade.
    pub fn new() -> CloseTradeRequest {
        CloseTradeRequest { units: None }
    }

    request_option_setter!(units, String);
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
///
/// Transaction fields are raw JSON values because the OANDA API returns a
/// polymorphic union of transaction sub-types.
#[derive(Debug, Serialize, Deserialize)]
pub struct CloseTradeResponse {
    /// The order-fill transaction that recorded the trade closure.
    #[serde(rename = "orderFillTransaction")]
    pub order_fill_transaction: Option<serde_json::Value>,
    /// The order-cancel transaction, present when the close order itself was
    /// cancelled (e.g. the trade was already closed).
    #[serde(rename = "orderCancelTransaction")]
    pub order_cancel_transaction: Option<serde_json::Value>,
    /// IDs of all transactions related to this close request.
    #[serde(rename = "relatedTransactionIDs")]
    pub related_transaction_ids: Option<Vec<TransactionID>>,
    /// ID of the most recent transaction on the account after this request.
    #[serde(rename = "lastTransactionID")]
    pub last_transaction_id: Option<TransactionID>,
}

/// Request body for
/// `PUT /v3/accounts/{accountID}/trades/{tradeSpecifier}/clientExtensions`.
///
/// Wraps a [`ClientExtensions`] value to match the JSON envelope the OANDA API
/// expects (`{"clientExtensions": {...}}`).
#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateClientExtensionsRequest {
    /// The new client extensions to apply to the trade.
    #[serde(rename = "clientExtensions")]
    pub client_extensions: ClientExtensions,
}

/// Response body for a successful
/// `PUT /v3/accounts/{accountID}/trades/{tradeSpecifier}/clientExtensions`
/// (HTTP 200).
#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateTradeClientExtensionsResponse {
    /// The transaction that recorded the client-extensions change on the trade.
    #[serde(rename = "tradeClientExtensionsModifyTransaction")]
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
#[error("Trade client extensions update error {error_code}: {error_message}")]
pub struct UpdateTradeClientExtensionsErrorResponse {
    /// The reject transaction that recorded why the modification was refused.
    #[serde(rename = "TradeClientExtensionsModifyRejectTransaction")]
    pub trade_client_extensions_modify_reject_transaction:
        TradeClientExtensionsModifyRejectTransaction,
    /// ID of the most recent transaction on the account.
    #[serde(rename = "lastTransactionID")]
    pub last_transaction_id: TransactionID,
    /// IDs of all transactions related to this (failed) request.
    #[serde(rename = "relatedTransactionIDs")]
    pub related_transaction_ids: Vec<TransactionID>,
    /// Machine-readable error code returned by the OANDA API.
    #[serde(rename = "errorCode")]
    pub error_code: String,
    /// Human-readable description of the error.
    #[serde(rename = "errorMessage")]
    pub error_message: String,
}

// ---------------------------------------------------------------------------
// Service
// ---------------------------------------------------------------------------

/// Provides access to the OANDA Trade endpoints (`/v3/accounts/{id}/trades/...`).
///
/// Obtain an instance via [`Client::trade`](crate::client::Client::trade).
pub struct TradeService<'a> {
    client: &'a Client,
}

impl<'a> TradeService<'a> {
    /// Creates a new `TradeService` bound to the given client.
    pub fn new(client: &'a Client) -> Self {
        Self { client }
    }

    /// Lists all trades on the account (open and closed).
    ///
    /// Calls `GET /v3/accounts/{accountID}/trades`.
    ///
    /// # Panics
    ///
    /// Panics if no `account_id` has been set on the client.
    pub async fn list(&self) -> Result<ListTradesResponse, APIError> {
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
        handle_response!(
            http_resp,
            success: StatusCode::OK => ListTradesResponse,
            errors: [ ]
        )
    }

    /// Lists all currently open trades on the account.
    ///
    /// Calls `GET /v3/accounts/{accountID}/openTrades`.
    ///
    /// # Panics
    ///
    /// Panics if no `account_id` has been set on the client.
    pub async fn list_open(&self) -> Result<ListTradesResponse, APIError> {
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
        handle_response!(
            http_resp,
            success: StatusCode::OK => ListTradesResponse,
            errors: [ ]
        )
    }

    /// Returns the details of the trade identified by `specifier`.
    ///
    /// Calls `GET /v3/accounts/{accountID}/trades/{tradeSpecifier}`.
    ///
    /// # Panics
    ///
    /// Panics if no `account_id` has been set on the client.
    pub async fn get_details(
        &self,
        specifier: TradeSpecifier,
    ) -> Result<GetTradeDetailsResponse, APIError> {
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
        handle_response!(
            http_resp,
            success: StatusCode::OK => GetTradeDetailsResponse,
            errors: [ ]
        )
    }

    /// Fully closes the trade identified by `specifier`.
    ///
    /// Calls `PUT /v3/accounts/{accountID}/trades/{tradeSpecifier}/close`.
    /// To partially close a trade (reduce units), use the OANDA API directly
    /// with a units parameter — partial-close is not yet exposed here.
    ///
    /// # Panics
    ///
    /// Panics if no `account_id` has been set on the client.
    pub async fn close(
        &self,
        specifier: TradeSpecifier,
        req: CloseTradeRequest,
    ) -> Result<CloseTradeResponse, APIError> {
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
        let http_resp = self.client.http_client.put(url).json(&req).send().await?;
        handle_response!(
            http_resp,
            success: StatusCode::OK => CloseTradeResponse,
            errors: [ ]
        )
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
    /// # Panics
    ///
    /// Panics if no `account_id` has been set on the client.
    pub async fn update_client_extensions(
        &self,
        specifier: TradeSpecifier,
        client_extensions: ClientExtensions,
    ) -> Result<UpdateTradeClientExtensionsResponse, APIError> {
        let url = self
            .client
            .base_url
            .join(
                format!(
                    "/v3/accounts/{}/trades/{}/clientExtensions",
                    self.client.account_id.as_ref().expect("Missing account_id"),
                    specifier
                )
                .as_str(),
            )
            .unwrap();
        let req = UpdateClientExtensionsRequest { client_extensions };
        let http_resp = self.client.http_client.put(url).json(&req).send().await?;
        handle_response!(
            http_resp,
            success: StatusCode::OK => UpdateTradeClientExtensionsResponse,
            errors: [
                StatusCode::BAD_REQUEST => (UpdateTradeClientExtensionsErrorResponse, UpdateTradeClientExtensionsError),
                StatusCode::NOT_FOUND => (UpdateTradeClientExtensionsErrorResponse, UpdateTradeClientExtensionsError),
            ]
        )
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
}

#[cfg(feature = "write-tests")]
#[cfg(test)]
mod write_tests {
    use crate::client::setup_test_client;
    use crate::order::{MarketOrderRequest, OrderRequest};
    use crate::trade::CloseTradeRequest;
    use crate::transaction::{ClientExtensions, TransactionID};

    async fn create_market_order(client: &crate::client::Client) -> TransactionID {
        let req = MarketOrderRequest::new("USD_JPY".to_string(), "10000".to_string());
        let resp = client
            .order()
            .create(OrderRequest::Market(req))
            .await
            .unwrap();
        println!("{:#?}", resp);
        resp.order_fill_transaction.unwrap().id
    }

    #[tokio::test]
    async fn test_trades() {
        let client = setup_test_client();

        let id = create_market_order(&client).await;

        let client_id = "test_trade".to_string();
        let client_extensions = ClientExtensions::new().id(client_id.clone());
        let resp = client
            .trade()
            .update_client_extensions(id, client_extensions)
            .await
            .unwrap();
        println!("{:#?}", resp);

        let resp = client.trade().list_open().await.unwrap();
        println!("{:#?}", resp);

        let resp = client
            .trade()
            .get_details(format!("@{}", client_id))
            .await
            .unwrap();
        println!("{:#?}", resp);

        let resp = client
            .trade()
            .close(format!("@{}", client_id), CloseTradeRequest::new())
            .await
            .unwrap();
        println!("{:#?}", resp);
    }
}
