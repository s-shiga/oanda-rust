use crate::client::Client;
use crate::errors::APIError;
use crate::http::decode_response;
use crate::instrument::InstrumentName;
use crate::primitives::DecimalNumber;
use crate::request_option_setter;
use crate::transaction::{
    AccountUnits, ClientExtensions, MarketOrderTransaction, OrderCancelTransaction,
    OrderFillTransaction, TradeID, TransactionID,
};
use reqwest::{Method, Request, StatusCode};
use serde::{Deserialize, Serialize};

/// The net exposure an account holds on a single instrument, aggregating all
/// open trades on both the long and short side.
///
/// A position exists as long as there is at least one open trade on the
/// instrument. Returned by the position endpoints and embedded in full account
/// detail responses.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Position {
    /// The instrument this position is on (e.g. `"EUR_USD"`).
    pub instrument: InstrumentName,
    /// Cumulative realised profit/loss across all closed trades on this
    /// instrument, in home currency units.
    pub pl: AccountUnits,
    /// Current unrealised profit/loss based on the live market price,
    /// in home currency units.
    #[serde(rename = "unrealizedPL")]
    pub unrealized_pl: AccountUnits,
    /// Margin currently consumed by the net open position, in home currency
    /// units. `None` when the position is closed.
    pub margin_used: Option<AccountUnits>,
    /// Realised P&L since the last account P&L reset, in home currency units.
    #[serde(rename = "resettablePL")]
    pub resettable_pl: AccountUnits,
    /// Cumulative financing (swap/rollover) charges or credits applied to all
    /// trades on this instrument, in home currency units.
    pub financing: AccountUnits,
    /// Cumulative commission paid on trades for this instrument, in home
    /// currency units.
    pub commission: AccountUnits,
    /// Cumulative dividend adjustment applied to this position (for CFDs that
    /// pay dividends), in home currency units.
    pub dividend_adjustment: AccountUnits,
    /// Cumulative fees paid for guaranteed stop-loss execution on this
    /// instrument, in home currency units.
    pub guaranteed_execution_fees: AccountUnits,
    /// Aggregated state of all long (buy) trades on this instrument.
    pub long: PositionSide,
    /// Aggregated state of all short (sell) trades on this instrument.
    pub short: PositionSide,
}

/// The aggregated state of all trades on one side (long or short) of a
/// [`Position`].
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PositionSide {
    /// Net number of units held on this side. Positive for long, negative for
    /// short. Zero when there are no open trades on this side.
    pub units: DecimalNumber,
    /// Volume-weighted average open price across all trades on this side.
    /// `None` when `units` is zero.
    pub average_price: Option<DecimalNumber>,
    /// IDs of the open trades that make up this side of the position.
    /// `None` when there are no open trades on this side.
    #[serde(rename = "tradeIDs")]
    pub trade_ids: Option<Vec<TradeID>>,
    /// Cumulative realised profit/loss from closed trades on this side, in
    /// home currency units.
    pub pl: AccountUnits,
    /// Current unrealised profit/loss on this side based on the live market
    /// price, in home currency units.
    #[serde(rename = "unrealizedPL")]
    pub unrealized_pl: AccountUnits,
    /// Realised P&L on this side since the last account P&L reset, in home
    /// currency units.
    #[serde(rename = "resettablePL")]
    pub resettable_pl: AccountUnits,
    /// Cumulative financing charges or credits on this side, in home currency
    /// units.
    pub financing: AccountUnits,
    /// Cumulative dividend adjustment on this side, in home currency units.
    pub dividend_adjustment: AccountUnits,
    /// Cumulative guaranteed execution fees on this side, in home currency
    /// units.
    pub guaranteed_execution_fees: AccountUnits,
}

/// The price-dependent (dynamic) state of a position, returned as part of
/// [`AccountChangesState`](crate::account::AccountChangesState).
///
/// Contains only the fields that change as the market moves; all static
/// position data is in [`Position`].
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CalculatedPositionState {
    /// The instrument this position state belongs to.
    pub instrument: InstrumentName,
    /// Net unrealised P&L across both long and short sides, in home currency
    /// units (`long_unrealized_pl + short_unrealized_pl`).
    #[serde(rename = "netUnrealizedPL")]
    pub net_unrealized_pl: AccountUnits,
    /// Unrealised P&L on the long side, in home currency units.
    #[serde(rename = "longUnrealizedPL")]
    pub long_unrealized_pl: AccountUnits,
    /// Unrealised P&L on the short side, in home currency units.
    #[serde(rename = "shortUnrealizedPL")]
    pub short_unrealized_pl: AccountUnits,
    /// Margin currently consumed by the net open position, in home currency
    /// units.
    pub margin_used: AccountUnits,
}

/// Response body for `GET /v3/accounts/{accountID}/positions` and
/// `GET /v3/accounts/{accountID}/openPositions`.
#[derive(Debug, Serialize, Deserialize)]
pub struct ListPositionsResponse {
    /// The list of positions matching the request.
    pub positions: Vec<Position>,
    /// ID of the most recent transaction on the account.
    #[serde(rename = "lastTransactionID")]
    pub last_transaction_id: TransactionID,
}

/// Response body for `GET /v3/accounts/{accountID}/positions/{instrument}`.
#[derive(Debug, Serialize, Deserialize)]
pub struct GetPositionDetailsResponse {
    /// The requested position.
    pub position: Position,
    /// ID of the most recent transaction on the account.
    #[serde(rename = "lastTransactionID")]
    pub last_transaction_id: TransactionID,
}

/// Request body for `PUT /v3/accounts/{accountID}/positions/{instrument}/close`.
///
/// Specify `"ALL"` to close all units on a side, `"NONE"` to leave it open,
/// or a decimal string (e.g. `"5000"`) to partially close. When both fields
/// are omitted the API defaults to closing all units on both sides.
#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClosePositionRequest {
    /// Units of the long side to close. `"ALL"` closes all long units,
    /// `"NONE"` leaves the long side open, or a decimal string (e.g. `"5000"`)
    /// partially closes. Omitted when `None`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub long_units: Option<String>,
    /// Optional client extensions to attach to the market order created to
    /// close the long side. Omitted when `None`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub long_client_extensions: Option<ClientExtensions>,
    /// Units of the short side to close. `"ALL"` closes all short units,
    /// `"NONE"` leaves the short side open, or a decimal string partially
    /// closes. Omitted when `None`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub short_units: Option<String>,
    /// Optional client extensions to attach to the market order created to
    /// close the short side. Omitted when `None`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub short_client_extensions: Option<ClientExtensions>,
}

impl ClosePositionRequest {
    /// Creates a new request with all fields unset.
    ///
    /// The OANDA API interprets omitted `longUnits`/`shortUnits` as closing
    /// all units on each side. Use the builder methods to close only specific
    /// sides or a partial number of units.
    pub fn new() -> Self {
        Self::default()
    }

    request_option_setter!(long_units, String);
    request_option_setter!(long_client_extensions, ClientExtensions);
    request_option_setter!(short_units, String);
    request_option_setter!(short_client_extensions, ClientExtensions);
}

/// Response body for `PUT /v3/accounts/{accountID}/positions/{instrument}/close`
/// (HTTP 200).
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClosePositionResponse {
    /// The market order transaction created to close the long side, if any.
    pub long_order_create_transaction: Option<MarketOrderTransaction>,
    /// The fill transaction for the long-side close order, if it was filled.
    pub long_order_fill_transaction: Option<Box<OrderFillTransaction>>,
    /// The cancel transaction for the long-side close order, if it was cancelled.
    pub long_order_cancel_transaction: Option<OrderCancelTransaction>,
    /// The market order transaction created to close the short side, if any.
    pub short_order_create_transaction: Option<MarketOrderTransaction>,
    /// The fill transaction for the short-side close order, if it was filled.
    pub short_order_fill_transaction: Option<Box<OrderFillTransaction>>,
    /// The cancel transaction for the short-side close order, if it was cancelled.
    pub short_order_cancel_transaction: Option<OrderCancelTransaction>,
    /// IDs of all transactions related to this close request.
    #[serde(rename = "relatedTransactionIDs")]
    pub related_transaction_ids: Vec<TransactionID>,
    /// ID of the most recent transaction on the account after this request.
    #[serde(rename = "lastTransactionID")]
    pub last_transaction_id: TransactionID,
}

/// Provides access to the OANDA Position endpoints
/// (`/v3/accounts/{id}/positions/...`).
///
/// Obtain an instance via [`Client::position`](crate::client::Client::position).
pub struct PositionService<'a> {
    client: &'a Client,
}

impl<'a> PositionService<'a> {
    /// Creates a new `PositionService` bound to the given client.
    pub(crate) fn new(client: &'a Client) -> Self {
        Self { client }
    }

    /// Lists all positions on the account (open and closed).
    ///
    /// Calls `GET /v3/accounts/{accountID}/positions`.
    ///
    /// Returns [`APIError::InvalidRequest`] if no account ID is configured.
    pub async fn list(&self) -> Result<ListPositionsResponse, APIError> {
        let url = self.client.account_url("positions")?;
        let http_req = Request::new(Method::GET, url);
        let http_resp = self.client.http_client.execute(http_req).await?;
        decode_response::<ListPositionsResponse>(http_resp, StatusCode::OK, None).await
    }

    /// Lists all currently open positions on the account.
    ///
    /// Calls `GET /v3/accounts/{accountID}/openPositions`.
    ///
    /// Returns [`APIError::InvalidRequest`] if no account ID is configured.
    pub async fn list_open(&self) -> Result<ListPositionsResponse, APIError> {
        let url = self.client.account_url("openPositions")?;
        let http_req = Request::new(Method::GET, url);
        let http_resp = self.client.http_client.execute(http_req).await?;
        decode_response::<ListPositionsResponse>(http_resp, StatusCode::OK, None).await
    }

    /// Returns the details of the position for the given `instrument`.
    ///
    /// Calls `GET /v3/accounts/{accountID}/positions/{instrument}`.
    ///
    /// Returns [`APIError::InvalidRequest`] if no account ID is configured.
    pub async fn get_details(
        &self,
        instrument: InstrumentName,
    ) -> Result<GetPositionDetailsResponse, APIError> {
        let url = self
            .client
            .account_url(&format!("positions/{}", instrument))?;
        let http_req = Request::new(Method::GET, url);
        let http_resp = self.client.http_client.execute(http_req).await?;
        decode_response::<GetPositionDetailsResponse>(http_resp, StatusCode::OK, None).await
    }

    /// Closes all or part of an open position for the given `instrument`.
    ///
    /// Calls `PUT /v3/accounts/{accountID}/positions/{instrument}/close`.
    ///
    /// Use [`ClosePositionRequest`] to control how many long/short units to
    /// close. Omitting both sides defaults to closing all open units.
    ///
    /// Returns [`APIError::InvalidRequest`] if no account ID is configured.
    pub async fn close(
        &self,
        instrument: InstrumentName,
        req: ClosePositionRequest,
    ) -> Result<ClosePositionResponse, APIError> {
        let url = self
            .client
            .account_url(&format!("positions/{}/close", instrument))?;
        let http_resp = self.client.http_client.put(url).json(&req).send().await?;
        decode_response::<ClosePositionResponse>(http_resp, StatusCode::OK, None).await
    }
}
