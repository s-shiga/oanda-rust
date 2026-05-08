use crate::client::Client;
use crate::errors::{APIError, CommonErrorResponse, ErrorResponse};
use crate::instrument::InstrumentName;
use crate::primitives::DecimalNumber;
use crate::transaction::{AccountUnits, ClientExtensions, TradeID, TransactionID};
use crate::{handle_response, request_option_setter};
use reqwest::{Method, Request, StatusCode};
use serde::{Deserialize, Serialize};

/// The net exposure an account holds on a single instrument, aggregating all
/// open trades on both the long and short side.
///
/// A position exists as long as there is at least one open trade on the
/// instrument. Returned by the position endpoints and embedded in full account
/// detail responses.
#[derive(Debug, Serialize, Deserialize)]
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
    #[serde(rename = "marginUsed")]
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
    #[serde(rename = "dividendAdjustment")]
    pub dividend_adjustment: AccountUnits,
    /// Cumulative fees paid for guaranteed stop-loss execution on this
    /// instrument, in home currency units.
    #[serde(rename = "guaranteedExecutionFees")]
    pub guaranteed_execution_fees: AccountUnits,
    /// Aggregated state of all long (buy) trades on this instrument.
    pub long: PositionSide,
    /// Aggregated state of all short (sell) trades on this instrument.
    pub short: PositionSide,
}

/// The aggregated state of all trades on one side (long or short) of a
/// [`Position`].
#[derive(Debug, Serialize, Deserialize)]
pub struct PositionSide {
    /// Net number of units held on this side. Positive for long, negative for
    /// short. Zero when there are no open trades on this side.
    pub units: DecimalNumber,
    /// Volume-weighted average open price across all trades on this side.
    /// `None` when `units` is zero.
    #[serde(rename = "averagePrice")]
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
    #[serde(rename = "dividendAdjustment")]
    pub dividend_adjustment: AccountUnits,
    /// Cumulative guaranteed execution fees on this side, in home currency
    /// units.
    #[serde(rename = "guaranteedExecutionFees")]
    pub guaranteed_execution_fees: AccountUnits,
}

/// The price-dependent (dynamic) state of a position, returned as part of
/// [`AccountChangesState`](crate::account::AccountChangesState).
///
/// Contains only the fields that change as the market moves; all static
/// position data is in [`Position`].
#[derive(Debug, Serialize, Deserialize)]
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
    #[serde(rename = "marginUsed")]
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
pub struct ClosePositionRequest {
    /// Units of the long side to close. `"ALL"` closes all long units,
    /// `"NONE"` leaves the long side open, or a decimal string (e.g. `"5000"`)
    /// partially closes. Omitted when `None`.
    #[serde(rename = "longUnits", skip_serializing_if = "Option::is_none")]
    pub long_units: Option<String>,
    /// Optional client extensions to attach to the market order created to
    /// close the long side. Omitted when `None`.
    #[serde(
        rename = "longClientExtensions",
        skip_serializing_if = "Option::is_none"
    )]
    pub long_client_extensions: Option<ClientExtensions>,
    /// Units of the short side to close. `"ALL"` closes all short units,
    /// `"NONE"` leaves the short side open, or a decimal string partially
    /// closes. Omitted when `None`.
    #[serde(rename = "shortUnits", skip_serializing_if = "Option::is_none")]
    pub short_units: Option<String>,
    /// Optional client extensions to attach to the market order created to
    /// close the short side. Omitted when `None`.
    #[serde(
        rename = "shortClientExtensions",
        skip_serializing_if = "Option::is_none"
    )]
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
///
/// Transaction fields are raw JSON values because the OANDA API returns a
/// polymorphic union of transaction sub-types.
#[derive(Debug, Serialize, Deserialize)]
pub struct ClosePositionResponse {
    /// The market order transaction created to close the long side, if any.
    #[serde(rename = "longOrderCreateTransaction")]
    pub long_order_create_transaction: Option<serde_json::Value>,
    /// The fill transaction for the long-side close order, if it was filled.
    #[serde(rename = "longOrderFillTransaction")]
    pub long_order_fill_transaction: Option<serde_json::Value>,
    /// The cancel transaction for the long-side close order, if it was cancelled.
    #[serde(rename = "longOrderCancelTransaction")]
    pub long_order_cancel_transaction: Option<serde_json::Value>,
    /// The market order transaction created to close the short side, if any.
    #[serde(rename = "shortOrderCreateTransaction")]
    pub short_order_create_transaction: Option<serde_json::Value>,
    /// The fill transaction for the short-side close order, if it was filled.
    #[serde(rename = "shortOrderFillTransaction")]
    pub short_order_fill_transaction: Option<serde_json::Value>,
    /// The cancel transaction for the short-side close order, if it was cancelled.
    #[serde(rename = "shortOrderCancelTransaction")]
    pub short_order_cancel_transaction: Option<serde_json::Value>,
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
    /// # Panics
    ///
    /// Panics if no `account_id` has been set on the client.
    pub async fn list(&self) -> Result<ListPositionsResponse, APIError> {
        let url = self.client.account_url("positions");
        let http_req = Request::new(Method::GET, url);
        let http_resp = self.client.http_client.execute(http_req).await?;
        handle_response!(
            http_resp,
            success: StatusCode::OK => ListPositionsResponse,
            errors: [ ]
        )
    }

    /// Lists all currently open positions on the account.
    ///
    /// Calls `GET /v3/accounts/{accountID}/openPositions`.
    ///
    /// # Panics
    ///
    /// Panics if no `account_id` has been set on the client.
    pub async fn list_open(&self) -> Result<ListPositionsResponse, APIError> {
        let url = self.client.account_url("openPositions");
        let http_req = Request::new(Method::GET, url);
        let http_resp = self.client.http_client.execute(http_req).await?;
        handle_response!(
            http_resp,
            success: StatusCode::OK => ListPositionsResponse,
            errors: [ ]
        )
    }

    /// Returns the details of the position for the given `instrument`.
    ///
    /// Calls `GET /v3/accounts/{accountID}/positions/{instrument}`.
    ///
    /// # Panics
    ///
    /// Panics if no `account_id` has been set on the client.
    pub async fn get_details(
        &self,
        instrument: InstrumentName,
    ) -> Result<GetPositionDetailsResponse, APIError> {
        let url = self.client.account_url(&format!("positions/{}", instrument));
        let http_req = Request::new(Method::GET, url);
        let http_resp = self.client.http_client.execute(http_req).await?;
        handle_response!(
            http_resp,
            success: StatusCode::OK => GetPositionDetailsResponse,
            errors: [ ]
        )
    }

    /// Closes all or part of an open position for the given `instrument`.
    ///
    /// Calls `PUT /v3/accounts/{accountID}/positions/{instrument}/close`.
    ///
    /// Use [`ClosePositionRequest`] to control how many long/short units to
    /// close. Omitting both sides defaults to closing all open units.
    ///
    /// # Panics
    ///
    /// Panics if no `account_id` has been set on the client.
    pub async fn close(
        &self,
        instrument: InstrumentName,
        req: ClosePositionRequest,
    ) -> Result<ClosePositionResponse, APIError> {
        let url = self.client.account_url(&format!("positions/{}/close", instrument));
        let http_resp = self.client.http_client.put(url).json(&req).send().await?;
        handle_response!(
            http_resp,
            success: StatusCode::OK => ClosePositionResponse,
            errors: [ ]
        )
    }
}

#[cfg(test)]
mod tests {
    use crate::client::setup_test_client;

    #[tokio::test]
    async fn test_position_list() {
        let client = setup_test_client();
        let positions = client.position().list().await.unwrap();
        println!("{:#?}", positions);
    }

    #[tokio::test]
    async fn test_position_list_open() {
        let client = setup_test_client();
        let positions = client.position().list_open().await.unwrap();
        println!("{:#?}", positions);
    }

    #[tokio::test]
    async fn test_position_details() {
        let client = setup_test_client();
        let details = client
            .position()
            .get_details(String::from("USD_JPY"))
            .await
            .unwrap();
        println!("{:#?}", details);
    }
}

#[cfg(feature = "write-tests")]
#[cfg(test)]
mod write_tests {
    use crate::client::setup_test_client;
    use crate::position::ClosePositionRequest;

    #[tokio::test]
    async fn test_position_close() {
        use crate::order::{MarketOrderRequest, OrderRequest};
        let client = setup_test_client();
        client
            .order()
            .create(OrderRequest::Market(MarketOrderRequest::new(
                "USD_JPY".to_string(),
                "10000".to_string(),
            )))
            .await
            .unwrap();
        let resp = client
            .position()
            .close(
                "USD_JPY".to_string(),
                ClosePositionRequest::new().long_units("ALL".to_string()),
            )
            .await
            .unwrap();
        println!("{:#?}", resp);
    }
}
