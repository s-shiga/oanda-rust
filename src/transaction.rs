use crate::account::AccountID;
use crate::client::Client;
use crate::errors::{APIError, CommonErrorResponse, ErrorResponse};
use crate::instrument::InstrumentName;
use crate::order::{OrderPositionFill, OrderTriggerCondition, TimeInForce};
use crate::pricing::{ClientPrice, PriceValue};
use crate::primitives::{Currency, DecimalNumber, HomeConversionFactors};
use crate::{handle_response, request_option_setter};
use chrono::{DateTime, Utc};
use reqwest::{Request, StatusCode};
use serde::{Deserialize, Serialize};
use strum_macros::Display;
use url::Url;

/// A unique identifier for a transaction, assigned by OANDA (e.g. `"1234"`).
pub type TransactionID = String;
/// A client-assigned identifier for an order or trade (e.g. `"my-order-001"`).
pub type ClientID = String;
/// A client-assigned tag for grouping orders or trades.
pub type ClientTag = String;
/// A free-text comment attached to an order or trade by the client.
pub type ClientComment = String;
/// An identifier for the HTTP request that created a transaction, echoed back by OANDA.
pub type RequestID = String;
/// A monetary amount expressed in the account's home currency, serialised as a decimal string.
pub type AccountUnits = String;
/// A unique identifier for a trade, assigned by OANDA (e.g. `"42"`).
pub type TradeID = String;
/// A unique identifier for an order, assigned by OANDA (e.g. `"7"`).
pub type OrderID = String;

// ---------------------------------------------------------------------------
// Transaction enum (tagged union)
// ---------------------------------------------------------------------------

/// A tagged union of every transaction type that the OANDA API can return.
///
/// Deserialized from the `"type"` field in the JSON payload. Use this when
/// handling generic transaction streams or history endpoints where the exact
/// sub-type is not known in advance.
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Transaction {
    #[serde(rename = "ORDER_FILL")]
    OrderFillTransaction(Box<OrderFillTransaction>),
    #[serde(rename = "ORDER_CANCEL")]
    OrderCancelTransaction(OrderCancelTransaction),
    #[serde(rename = "ORDER_CANCEL_REJECT")]
    OrderCancelRejectTransaction(OrderCancelRejectTransaction),
    #[serde(rename = "ORDER_CLIENT_EXTENSIONS_MODIFY")]
    OrderClientExtensionsModifyTransaction(OrderClientExtensionsModifyTransaction),
    #[serde(rename = "ORDER_CLIENT_EXTENSIONS_MODIFY_REJECT")]
    OrderClientExtensionsModifyRejectTransaction(OrderClientExtensionsModifyRejectTransaction),
    #[serde(rename = "CREATE")]
    CreateTransaction(CreateTransaction),
    #[serde(rename = "CLOSE")]
    CloseTransaction(CloseTransaction),
    #[serde(rename = "REOPEN")]
    ReopenTransaction(ReopenTransaction),
    #[serde(rename = "CLIENT_CONFIGURE")]
    ClientConfigureTransaction(ClientConfigureTransaction),
    #[serde(rename = "CLIENT_CONFIGURE_REJECT")]
    ClientConfigureRejectTransaction(ClientConfigureRejectTransaction),
    #[serde(rename = "TRANSFER_FUNDS")]
    TransferFundsTransaction(TransferFundsTransaction),
    #[serde(rename = "TRANSFER_FUNDS_REJECT")]
    TransferFundsRejectTransaction(TransferFundsRejectTransaction),
    #[serde(rename = "MARKET_ORDER")]
    MarketOrderTransaction(MarketOrderTransaction),
    #[serde(rename = "MARKET_ORDER_REJECT")]
    MarketOrderRejectTransaction(MarketOrderRejectTransaction),
    #[serde(rename = "FIXED_PRICE_ORDER")]
    FixedPriceOrderTransaction(FixedPriceOrderTransaction),
    #[serde(rename = "LIMIT_ORDER")]
    LimitOrderTransaction(LimitOrderTransaction),
    #[serde(rename = "LIMIT_ORDER_REJECT")]
    LimitOrderRejectTransaction(LimitOrderRejectTransaction),
    #[serde(rename = "STOP_ORDER")]
    StopOrderTransaction(StopOrderTransaction),
    #[serde(rename = "STOP_ORDER_REJECT")]
    StopOrderRejectTransaction(StopOrderRejectTransaction),
    #[serde(rename = "MARKET_IF_TOUCHED_ORDER")]
    MarketIfTouchedOrderTransaction(MarketIfTouchedOrderTransaction),
    #[serde(rename = "MARKET_IF_TOUCHED_ORDER_REJECT")]
    MarketIfTouchedOrderRejectTransaction(MarketIfTouchedOrderRejectTransaction),
    #[serde(rename = "TAKE_PROFIT_ORDER")]
    TakeProfitOrderTransaction(TakeProfitOrderTransaction),
    #[serde(rename = "TAKE_PROFIT_ORDER_REJECT")]
    TakeProfitOrderRejectTransaction(TakeProfitOrderRejectTransaction),
    #[serde(rename = "STOP_LOSS_ORDER")]
    StopLossOrderTransaction(StopLossOrderTransaction),
    #[serde(rename = "STOP_LOSS_ORDER_REJECT")]
    StopLossOrderRejectTransaction(StopLossOrderRejectTransaction),
    #[serde(rename = "GUARANTEED_STOP_LOSS_ORDER")]
    GuaranteedStopLossOrderTransaction(GuaranteedStopLossOrderTransaction),
    #[serde(rename = "GUARANTEED_STOP_LOSS_ORDER_REJECT")]
    GuaranteedStopLossOrderRejectTransaction(GuaranteedStopLossOrderRejectTransaction),
    #[serde(rename = "TRAILING_STOP_LOSS_ORDER")]
    TrailingStopLossOrderTransaction(TrailingStopLossOrderTransaction),
    #[serde(rename = "TRAILING_STOP_LOSS_ORDER_REJECT")]
    TrailingStopLossOrderRejectTransaction(TrailingStopLossOrderRejectTransaction),
    #[serde(rename = "TRADE_CLIENT_EXTENSIONS_MODIFY")]
    TradeClientExtensionsModifyTransaction(TradeClientExtensionsModifyTransaction),
    #[serde(rename = "TRADE_CLIENT_EXTENSIONS_MODIFY_REJECT")]
    TradeClientExtensionsModifyRejectTransaction(TradeClientExtensionsModifyRejectTransaction),
    #[serde(rename = "MARGIN_CALL_ENTER")]
    MarginCallEnterTransaction(MarginCallEnterTransaction),
    #[serde(rename = "MARGIN_CALL_EXTEND")]
    MarginCallExtendTransaction(MarginCallExtendTransaction),
    #[serde(rename = "MARGIN_CALL_EXIT")]
    MarginCallExitTransaction(MarginCallExitTransaction),
    #[serde(rename = "DELAYED_TRADE_CLOSURE")]
    DelayedTradeClosureTransaction(DelayedTradeClosureTransaction),
    #[serde(rename = "DAILY_FINANCING")]
    DailyFinancingTransaction(DailyFinancingTransaction),
    #[serde(rename = "DIVIDEND_ADJUSTMENT")]
    DividendAdjustmentTransaction(DividendAdjustmentTransaction),
    #[serde(rename = "RESET_RESETTABLE_PL")]
    ResetResettablePLTransaction(ResetResettablePLTransaction),
}

/// A tagged union covering only the transaction types that create a new order.
///
/// Used in order-creation responses where OANDA always returns one of these
/// sub-types (never a cancel, fill, etc.).
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum OrderCreateTransaction {
    #[serde(rename = "MARKET_ORDER")]
    MarketOrderTransaction(MarketOrderTransaction),
    #[serde(rename = "FIXED_PRICE_ORDER")]
    FixedPriceOrderTransaction(FixedPriceOrderTransaction),
    #[serde(rename = "LIMIT_ORDER")]
    LimitOrderTransaction(LimitOrderTransaction),
    #[serde(rename = "STOP_ORDER")]
    StopOrderTransaction(StopOrderTransaction),
    #[serde(rename = "MARKET_IF_TOUCHED_ORDER")]
    MarketIfTouchedOrderTransaction(MarketIfTouchedOrderTransaction),
    #[serde(rename = "TAKE_PROFIT_ORDER")]
    TakeProfitOrderTransaction(TakeProfitOrderTransaction),
    #[serde(rename = "STOP_LOSS_ORDER")]
    StopLossOrderTransaction(StopLossOrderTransaction),
    #[serde(rename = "GUARANTEED_STOP_LOSS_ORDER")]
    GuaranteedStopLossOrderTransaction(GuaranteedStopLossOrderTransaction),
    #[serde(rename = "TRAILING_STOP_LOSS_ORDER")]
    TrailingStopLossOrderTransaction(TrailingStopLossOrderTransaction),
}

impl OrderCreateTransaction {
    /// Returns the [`TransactionID`] of the underlying order-creation transaction,
    /// regardless of which order type it is.
    pub fn get_id(&self) -> TransactionID {
        match self {
            OrderCreateTransaction::MarketOrderTransaction(transaction) => transaction.id.clone(),
            OrderCreateTransaction::FixedPriceOrderTransaction(transaction) => {
                transaction.id.clone()
            }
            OrderCreateTransaction::LimitOrderTransaction(transaction) => transaction.id.clone(),
            OrderCreateTransaction::StopOrderTransaction(transaction) => transaction.id.clone(),
            OrderCreateTransaction::MarketIfTouchedOrderTransaction(transaction) => {
                transaction.id.clone()
            }
            OrderCreateTransaction::TakeProfitOrderTransaction(transaction) => {
                transaction.id.clone()
            }
            OrderCreateTransaction::StopLossOrderTransaction(transaction) => transaction.id.clone(),
            OrderCreateTransaction::GuaranteedStopLossOrderTransaction(transaction) => {
                transaction.id.clone()
            }
            OrderCreateTransaction::TrailingStopLossOrderTransaction(transaction) => {
                transaction.id.clone()
            }
        }
    }
}

/// A tagged union of all transaction types that record a rejected order-creation attempt.
///
/// Returned in the error body when an order request is refused by OANDA
/// (e.g. insufficient margin, invalid parameters). Each variant carries a
/// `reject_reason` explaining why the order was not accepted.
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum OrderCreateRejectTransaction {
    #[serde(rename = "MARKET_ORDER_REJECT")]
    MarketOrderRejectTransaction(MarketOrderRejectTransaction),
    #[serde(rename = "LIMIT_ORDER_REJECT")]
    LimitOrderRejectTransaction(LimitOrderRejectTransaction),
    #[serde(rename = "STOP_ORDER_REJECT")]
    StopOrderRejectTransaction(StopOrderRejectTransaction),
    #[serde(rename = "MARKET_IF_TOUCHED_ORDER_REJECT")]
    MarketIfTouchedOrderRejectTransaction(MarketIfTouchedOrderRejectTransaction),
    #[serde(rename = "TAKE_PROFIT_ORDER_REJECT")]
    TakeProfitOrderRejectTransaction(TakeProfitOrderRejectTransaction),
    #[serde(rename = "STOP_LOSS_ORDER_REJECT")]
    StopLossOrderRejectTransaction(StopLossOrderRejectTransaction),
    #[serde(rename = "GUARANTEED_STOP_LOSS_ORDER_REJECT")]
    GuaranteedStopLossOrderRejectTransaction(GuaranteedStopLossOrderRejectTransaction),
    #[serde(rename = "TRAILING_STOP_LOSS_ORDER_REJECT")]
    TrailingStopLossOrderRejectTransaction(TrailingStopLossOrderRejectTransaction),
}

// ---------------------------------------------------------------------------
// Transaction Structs
// ---------------------------------------------------------------------------

/// Transaction recorded when a new OANDA account is created.
///
/// Contains the initial settings for the account such as the home currency,
/// division, and site identifiers assigned at creation time.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateTransaction {
    /// Unique transaction ID assigned by OANDA.
    pub id: TransactionID,
    /// Timestamp at which the transaction was created.
    pub time: DateTime<Utc>,
    /// OANDA internal user ID that initiated the transaction.
    #[serde(rename = "userID")]
    pub user_id: i64,
    /// The account this transaction belongs to.
    #[serde(rename = "accountID")]
    pub account_id: AccountID,
    /// ID of the batch this transaction is part of (same as `id` for single-transaction requests).
    #[serde(rename = "batchID")]
    pub batch_id: TransactionID,
    /// ID of the originating HTTP request, if provided by the client.
    #[serde(rename = "requestID")]
    pub request_id: Option<RequestID>,
    /// OANDA division that owns the account.
    #[serde(rename = "divisionID")]
    pub division_id: i64,
    /// OANDA site that the account was created on.
    #[serde(rename = "siteID")]
    pub site_id: i64,
    /// OANDA user ID of the account owner.
    #[serde(rename = "accountUserID")]
    pub account_user_id: i64,
    /// OANDA account number (numeric form of the account ID).
    pub account_number: i64,
    /// Home currency of the account (e.g. `"USD"`).
    pub home_currency: Currency,
}

/// Transaction recorded when an OANDA account is closed.
#[derive(Debug, Serialize, Deserialize)]
pub struct CloseTransaction {
    pub id: TransactionID,
    pub time: DateTime<Utc>,
    #[serde(rename = "userID")]
    pub user_id: i64,
    #[serde(rename = "accountID")]
    pub account_id: AccountID,
    #[serde(rename = "batchID")]
    pub batch_id: TransactionID,
    #[serde(rename = "requestID")]
    pub request_id: Option<RequestID>,
}

/// Transaction recorded when a previously closed account is reopened.
#[derive(Debug, Serialize, Deserialize)]
pub struct ReopenTransaction {
    pub id: TransactionID,
    pub time: DateTime<Utc>,
    #[serde(rename = "userID")]
    pub user_id: i64,
    #[serde(rename = "accountID")]
    pub account_id: AccountID,
    #[serde(rename = "batchID")]
    pub batch_id: TransactionID,
    #[serde(rename = "requestID")]
    pub request_id: Option<RequestID>,
}

/// Transaction recorded when an account's configuration is changed by the client
/// (e.g. updating the alias or margin rate).
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClientConfigureTransaction {
    pub id: TransactionID,
    pub time: DateTime<Utc>,
    #[serde(rename = "userID")]
    pub user_id: i64,
    #[serde(rename = "accountID")]
    pub account_id: AccountID,
    #[serde(rename = "batchID")]
    pub batch_id: TransactionID,
    #[serde(rename = "requestID")]
    pub request_id: Option<RequestID>,
    /// New display name set for the account, if changed.
    pub alias: Option<String>,
    /// New margin rate applied to the account, if changed.
    pub margin_rate: Option<DecimalNumber>,
}

/// Transaction recorded when an account configuration change is rejected.
/// Transaction recorded when an account configuration change is rejected.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClientConfigureRejectTransaction {
    pub id: TransactionID,
    pub time: DateTime<Utc>,
    #[serde(rename = "userID")]
    pub user_id: i64,
    #[serde(rename = "accountID")]
    pub account_id: AccountID,
    #[serde(rename = "batchID")]
    pub batch_id: TransactionID,
    #[serde(rename = "requestID")]
    pub request_id: Option<RequestID>,
    pub alias: Option<String>,
    pub margin_rate: Option<DecimalNumber>,
    /// Reason why the configuration change was rejected.
    pub reject_reason: TransactionRejectReason,
}

/// Transaction recorded when funds are deposited into or withdrawn from an account.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransferFundsTransaction {
    pub id: TransactionID,
    pub time: DateTime<Utc>,
    #[serde(rename = "userID")]
    pub user_id: i64,
    #[serde(rename = "accountID")]
    pub account_id: AccountID,
    #[serde(rename = "batchID")]
    pub batch_id: TransactionID,
    #[serde(rename = "requestID")]
    pub request_id: Option<RequestID>,
    /// Amount transferred, in home currency units. Positive = deposit, negative = withdrawal.
    pub amount: AccountUnits,
    /// Why the transfer occurred.
    pub funding_reason: FundingReason,
    /// Optional free-text comment attached to the transfer.
    pub comment: Option<String>,
    /// Account balance after the transfer, in home currency units.
    pub account_balance: AccountUnits,
}

/// Transaction recorded when a funds transfer request is rejected.
/// Transaction recorded when a funds transfer request is rejected.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransferFundsRejectTransaction {
    pub id: TransactionID,
    pub time: DateTime<Utc>,
    #[serde(rename = "userID")]
    pub user_id: i64,
    #[serde(rename = "accountID")]
    pub account_id: AccountID,
    #[serde(rename = "batchID")]
    pub batch_id: TransactionID,
    #[serde(rename = "requestID")]
    pub request_id: Option<RequestID>,
    pub amount: AccountUnits,
    pub funding_reason: FundingReason,
    pub comment: Option<String>,
    /// Reason why the transfer was rejected.
    pub reject_reason: TransactionRejectReason,
}

/// Transaction recorded when a market order is accepted and submitted to the market.
///
/// A market order fills immediately at the current market price. This transaction
/// is created before the corresponding [`OrderFillTransaction`].
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MarketOrderTransaction {
    pub id: TransactionID,
    pub time: DateTime<Utc>,
    #[serde(rename = "userID")]
    pub user_id: i64,
    #[serde(rename = "accountID")]
    pub account_id: AccountID,
    #[serde(rename = "batchID")]
    pub batch_id: TransactionID,
    #[serde(rename = "requestID")]
    pub request_id: Option<RequestID>,
    pub instrument: InstrumentName,
    pub units: DecimalNumber,
    pub time_in_force: TimeInForce,
    pub price_bound: Option<PriceValue>,
    pub position_fill: OrderPositionFill,
    pub trade_close: Option<MarketOrderTradeClose>,
    pub long_position_closeout: Option<MarketOrderPositionCloseout>,
    pub short_position_closeout: Option<MarketOrderPositionCloseout>,
    pub margin_closeout: Option<MarketOrderMarginCloseout>,
    pub delayed_trade_close: Option<MarketOrderDelayedTradeClose>,
    pub reason: MarketOrderReason,
    pub client_extensions: Option<ClientExtensions>,
    pub take_profit_on_fill: Option<TakeProfitDetails>,
    pub stop_loss_on_fill: Option<StopLossDetails>,
    pub trailing_stop_loss_on_fill: Option<TrailingStopLossDetails>,
    pub guaranteed_stop_loss_on_fill: Option<GuaranteedStopLossDetails>,
    pub trade_client_extensions: Option<ClientExtensions>,
}

/// Transaction recorded when a market order request is rejected.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MarketOrderRejectTransaction {
    pub id: TransactionID,
    pub time: DateTime<Utc>,
    #[serde(rename = "userID")]
    pub user_id: i64,
    #[serde(rename = "accountID")]
    pub account_id: AccountID,
    #[serde(rename = "batchID")]
    pub batch_id: TransactionID,
    #[serde(rename = "requestID")]
    pub request_id: Option<RequestID>,
    pub instrument: InstrumentName,
    pub units: Option<DecimalNumber>,
    pub time_in_force: TimeInForce,
    pub price_bound: Option<PriceValue>,
    pub position_fill: OrderPositionFill,
    pub trade_close: Option<MarketOrderTradeClose>,
    pub long_position_closeout: Option<MarketOrderPositionCloseout>,
    pub short_position_closeout: Option<MarketOrderPositionCloseout>,
    pub margin_closeout: Option<MarketOrderMarginCloseout>,
    pub delayed_trade_close: Option<MarketOrderDelayedTradeClose>,
    pub reason: MarketOrderReason,
    pub client_extensions: Option<ClientExtensions>,
    pub take_profit_on_fill: Option<TakeProfitDetails>,
    pub stop_loss_on_fill: Option<StopLossDetails>,
    pub trailing_stop_loss_on_fill: Option<TrailingStopLossDetails>,
    pub guaranteed_stop_loss_on_fill: Option<GuaranteedStopLossDetails>,
    pub trade_client_extensions: Option<ClientExtensions>,
    pub reject_reason: Option<TransactionRejectReason>,
}

/// Transaction recorded when a fixed-price order is created.
///
/// Fixed-price orders are created by OANDA internally (e.g. during account
/// migrations) and are not directly placeable by clients.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FixedPriceOrderTransaction {
    pub id: TransactionID,
    pub time: DateTime<Utc>,
    #[serde(rename = "userID")]
    pub user_id: i64,
    #[serde(rename = "accountID")]
    pub account_id: AccountID,
    #[serde(rename = "batchID")]
    pub batch_id: TransactionID,
    #[serde(rename = "requestID")]
    pub request_id: Option<RequestID>,
    pub instrument: InstrumentName,
    pub units: DecimalNumber,
    pub price: PriceValue,
    pub position_fill: OrderPositionFill,
    pub trade_state: Option<String>,
    pub reason: Option<FixedPriceOrderReason>,
    pub client_extensions: Option<ClientExtensions>,
    pub take_profit_on_fill: Option<TakeProfitDetails>,
    pub stop_loss_on_fill: Option<StopLossDetails>,
    pub trailing_stop_loss_on_fill: Option<TrailingStopLossDetails>,
    pub guaranteed_stop_loss_on_fill: Option<GuaranteedStopLossDetails>,
    pub trade_client_extensions: Option<ClientExtensions>,
}

/// Transaction recorded when a limit order is created or replaces an existing order.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LimitOrderTransaction {
    pub id: TransactionID,
    pub time: DateTime<Utc>,
    #[serde(rename = "userID")]
    pub user_id: i64,
    #[serde(rename = "accountID")]
    pub account_id: AccountID,
    #[serde(rename = "batchID")]
    pub batch_id: TransactionID,
    #[serde(rename = "requestID")]
    pub request_id: Option<RequestID>,
    pub instrument: InstrumentName,
    pub units: DecimalNumber,
    pub price: PriceValue,
    pub time_in_force: TimeInForce,
    pub gtd_time: Option<DateTime<Utc>>,
    pub position_fill: OrderPositionFill,
    pub trigger_condition: OrderTriggerCondition,
    pub reason: LimitOrderReason,
    pub client_extensions: Option<ClientExtensions>,
    pub take_profit_on_fill: Option<TakeProfitDetails>,
    pub stop_loss_on_fill: Option<StopLossDetails>,
    pub trailing_stop_loss_on_fill: Option<TrailingStopLossDetails>,
    pub guaranteed_stop_loss_on_fill: Option<GuaranteedStopLossDetails>,
    pub trade_client_extensions: Option<ClientExtensions>,
    #[serde(rename = "replacesOrderID")]
    pub replaces_order_id: Option<OrderID>,
    #[serde(rename = "cancellingTransactionID")]
    pub cancelling_transaction_id: Option<TransactionID>,
}

/// Transaction recorded when a limit order request is rejected.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LimitOrderRejectTransaction {
    pub id: TransactionID,
    pub time: DateTime<Utc>,
    #[serde(rename = "userID")]
    pub user_id: i64,
    #[serde(rename = "accountID")]
    pub account_id: AccountID,
    #[serde(rename = "batchID")]
    pub batch_id: TransactionID,
    #[serde(rename = "requestID")]
    pub request_id: Option<RequestID>,
    pub instrument: InstrumentName,
    pub units: DecimalNumber,
    pub price: PriceValue,
    pub time_in_force: TimeInForce,
    pub gtd_time: Option<DateTime<Utc>>,
    pub position_fill: OrderPositionFill,
    pub trigger_condition: OrderTriggerCondition,
    pub reason: LimitOrderReason,
    pub client_extensions: Option<ClientExtensions>,
    pub take_profit_on_fill: Option<TakeProfitDetails>,
    pub stop_loss_on_fill: Option<StopLossDetails>,
    pub trailing_stop_loss_on_fill: Option<TrailingStopLossDetails>,
    pub guaranteed_stop_loss_on_fill: Option<GuaranteedStopLossDetails>,
    pub trade_client_extensions: Option<ClientExtensions>,
    #[serde(rename = "replacesOrderID")]
    pub replaces_order_id: Option<OrderID>,
    #[serde(rename = "cancellingTransactionID")]
    pub cancelling_transaction_id: Option<TransactionID>,
    #[serde(rename = "intendedReplacesOrderID")]
    pub intended_replaces_order_id: Option<OrderID>,
    pub reject_reason: TransactionRejectReason,
}

/// Transaction recorded when a stop order is created or replaces an existing order.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StopOrderTransaction {
    pub id: TransactionID,
    pub time: DateTime<Utc>,
    #[serde(rename = "userID")]
    pub user_id: i64,
    #[serde(rename = "accountID")]
    pub account_id: AccountID,
    #[serde(rename = "batchID")]
    pub batch_id: TransactionID,
    #[serde(rename = "requestID")]
    pub request_id: Option<RequestID>,
    pub instrument: InstrumentName,
    pub units: DecimalNumber,
    pub price: PriceValue,
    pub price_bound: Option<PriceValue>,
    pub time_in_force: TimeInForce,
    pub gtd_time: Option<DateTime<Utc>>,
    pub position_fill: OrderPositionFill,
    pub trigger_condition: OrderTriggerCondition,
    pub reason: StopOrderReason,
    pub client_extensions: Option<ClientExtensions>,
    pub take_profit_on_fill: Option<TakeProfitDetails>,
    pub stop_loss_on_fill: Option<StopLossDetails>,
    pub trailing_stop_loss_on_fill: Option<TrailingStopLossDetails>,
    pub guaranteed_stop_loss_on_fill: Option<GuaranteedStopLossDetails>,
    pub trade_client_extensions: Option<ClientExtensions>,
    #[serde(rename = "replacesOrderID")]
    pub replaces_order_id: Option<OrderID>,
    #[serde(rename = "cancellingTransactionID")]
    pub cancelling_transaction_id: Option<TransactionID>,
}

/// Transaction recorded when a stop order request is rejected.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StopOrderRejectTransaction {
    pub id: TransactionID,
    pub time: DateTime<Utc>,
    #[serde(rename = "userID")]
    pub user_id: i64,
    #[serde(rename = "accountID")]
    pub account_id: AccountID,
    #[serde(rename = "batchID")]
    pub batch_id: TransactionID,
    #[serde(rename = "requestID")]
    pub request_id: Option<RequestID>,
    pub instrument: InstrumentName,
    pub units: DecimalNumber,
    pub price: PriceValue,
    pub price_bound: Option<PriceValue>,
    pub time_in_force: TimeInForce,
    pub gtd_time: Option<DateTime<Utc>>,
    pub position_fill: OrderPositionFill,
    pub trigger_condition: OrderTriggerCondition,
    pub reason: StopOrderReason,
    pub client_extensions: Option<ClientExtensions>,
    pub take_profit_on_fill: Option<TakeProfitDetails>,
    pub stop_loss_on_fill: Option<StopLossDetails>,
    pub trailing_stop_loss_on_fill: Option<TrailingStopLossDetails>,
    pub guaranteed_stop_loss_on_fill: Option<GuaranteedStopLossDetails>,
    pub trade_client_extensions: Option<ClientExtensions>,
    #[serde(rename = "replacesOrderID")]
    pub replaces_order_id: Option<OrderID>,
    #[serde(rename = "cancellingTransactionID")]
    pub cancelling_transaction_id: Option<TransactionID>,
    #[serde(rename = "intendedReplacesOrderID")]
    pub intended_replaces_order_id: Option<OrderID>,
    pub reject_reason: TransactionRejectReason,
}

/// Transaction recorded when a market-if-touched (MIT) order is created or replaces an existing order.
///
/// A MIT order becomes a market order once the instrument price crosses the specified trigger price.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MarketIfTouchedOrderTransaction {
    pub id: TransactionID,
    pub time: DateTime<Utc>,
    #[serde(rename = "userID")]
    pub user_id: i64,
    #[serde(rename = "accountID")]
    pub account_id: AccountID,
    #[serde(rename = "batchID")]
    pub batch_id: TransactionID,
    #[serde(rename = "requestID")]
    pub request_id: Option<RequestID>,
    pub instrument: InstrumentName,
    pub units: DecimalNumber,
    pub price: PriceValue,
    pub price_bound: Option<PriceValue>,
    pub time_in_force: TimeInForce,
    pub gtd_time: Option<DateTime<Utc>>,
    pub position_fill: OrderPositionFill,
    pub trigger_condition: OrderTriggerCondition,
    pub reason: MarketIfTouchedOrderReason,
    pub client_extensions: Option<ClientExtensions>,
    pub take_profit_on_fill: Option<TakeProfitDetails>,
    pub stop_loss_on_fill: Option<StopLossDetails>,
    pub trailing_stop_loss_on_fill: Option<TrailingStopLossDetails>,
    pub guaranteed_stop_loss_on_fill: Option<GuaranteedStopLossDetails>,
    pub trade_client_extensions: Option<ClientExtensions>,
    #[serde(rename = "replacesOrderID")]
    pub replaces_order_id: Option<OrderID>,
    #[serde(rename = "cancellingTransactionID")]
    pub cancelling_transaction_id: Option<TransactionID>,
}

/// Transaction recorded when a market-if-touched order request is rejected.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MarketIfTouchedOrderRejectTransaction {
    pub id: TransactionID,
    pub time: DateTime<Utc>,
    #[serde(rename = "userID")]
    pub user_id: i64,
    #[serde(rename = "accountID")]
    pub account_id: AccountID,
    #[serde(rename = "batchID")]
    pub batch_id: TransactionID,
    #[serde(rename = "requestID")]
    pub request_id: Option<RequestID>,
    pub instrument: InstrumentName,
    pub units: DecimalNumber,
    pub price: PriceValue,
    pub price_bound: Option<PriceValue>,
    pub time_in_force: TimeInForce,
    pub gtd_time: Option<DateTime<Utc>>,
    pub position_fill: OrderPositionFill,
    pub trigger_condition: OrderTriggerCondition,
    pub reason: MarketIfTouchedOrderReason,
    pub client_extensions: Option<ClientExtensions>,
    pub take_profit_on_fill: Option<TakeProfitDetails>,
    pub stop_loss_on_fill: Option<StopLossDetails>,
    pub trailing_stop_loss_on_fill: Option<TrailingStopLossDetails>,
    pub guaranteed_stop_loss_on_fill: Option<GuaranteedStopLossDetails>,
    pub trade_client_extensions: Option<ClientExtensions>,
    #[serde(rename = "replacesOrderID")]
    pub replaces_order_id: Option<OrderID>,
    #[serde(rename = "cancellingTransactionID")]
    pub cancelling_transaction_id: Option<TransactionID>,
    #[serde(rename = "intendedReplacesOrderID")]
    pub intended_replaces_order_id: Option<OrderID>,
    pub reject_reason: TransactionRejectReason,
}

/// Transaction recorded when a take-profit order is attached to or created for a trade.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TakeProfitOrderTransaction {
    pub id: TransactionID,
    pub time: DateTime<Utc>,
    #[serde(rename = "userID")]
    pub user_id: i64,
    #[serde(rename = "accountID")]
    pub account_id: AccountID,
    #[serde(rename = "batchID")]
    pub batch_id: TransactionID,
    #[serde(rename = "requestID")]
    pub request_id: Option<RequestID>,
    #[serde(rename = "tradeID")]
    pub trade_id: TradeID,
    #[serde(rename = "clientTradeID")]
    pub client_trade_id: Option<ClientID>,
    pub price: PriceValue,
    pub time_in_force: TimeInForce,
    pub gtd_time: Option<DateTime<Utc>>,
    pub trigger_condition: OrderTriggerCondition,
    pub reason: TakeProfitOrderReason,
    pub client_extensions: Option<ClientExtensions>,
    #[serde(rename = "orderFillTransactionID")]
    pub order_fill_transaction_id: Option<TransactionID>,
    #[serde(rename = "replacesOrderID")]
    pub replaces_order_id: Option<OrderID>,
    #[serde(rename = "cancellingTransactionID")]
    pub cancelling_transaction_id: Option<TransactionID>,
}

/// Transaction recorded when a take-profit order request is rejected.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TakeProfitOrderRejectTransaction {
    pub id: TransactionID,
    pub time: DateTime<Utc>,
    #[serde(rename = "userID")]
    pub user_id: i64,
    #[serde(rename = "accountID")]
    pub account_id: AccountID,
    #[serde(rename = "batchID")]
    pub batch_id: TransactionID,
    #[serde(rename = "requestID")]
    pub request_id: Option<RequestID>,
    #[serde(rename = "tradeID")]
    pub trade_id: TradeID,
    #[serde(rename = "clientTradeID")]
    pub client_trade_id: Option<ClientID>,
    pub price: PriceValue,
    pub time_in_force: TimeInForce,
    pub gtd_time: Option<DateTime<Utc>>,
    pub trigger_condition: OrderTriggerCondition,
    pub reason: TakeProfitOrderReason,
    pub client_extensions: Option<ClientExtensions>,
    #[serde(rename = "orderFillTransactionID")]
    pub order_fill_transaction_id: Option<TransactionID>,
    #[serde(rename = "replacesOrderID")]
    pub replaces_order_id: Option<OrderID>,
    #[serde(rename = "cancellingTransactionID")]
    pub cancelling_transaction_id: Option<TransactionID>,
    #[serde(rename = "intendedReplacesOrderID")]
    pub intended_replaces_order_id: Option<OrderID>,
    pub reject_reason: TransactionRejectReason,
}

/// Transaction recorded when a stop-loss order is attached to or created for a trade.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StopLossOrderTransaction {
    pub id: TransactionID,
    pub time: DateTime<Utc>,
    #[serde(rename = "userID")]
    pub user_id: i64,
    #[serde(rename = "accountID")]
    pub account_id: AccountID,
    #[serde(rename = "batchID")]
    pub batch_id: TransactionID,
    #[serde(rename = "requestID")]
    pub request_id: Option<RequestID>,
    #[serde(rename = "tradeID")]
    pub trade_id: TradeID,
    #[serde(rename = "clientTradeID")]
    pub client_trade_id: Option<ClientID>,
    pub price: PriceValue,
    pub distance: Option<DecimalNumber>,
    pub time_in_force: TimeInForce,
    pub gtd_time: Option<DateTime<Utc>>,
    pub trigger_condition: OrderTriggerCondition,
    pub reason: StopLossOrderReason,
    pub client_extensions: Option<ClientExtensions>,
    #[serde(rename = "orderFillTransactionID")]
    pub order_fill_transaction_id: Option<TransactionID>,
    #[serde(rename = "replacesOrderID")]
    pub replaces_order_id: Option<OrderID>,
    #[serde(rename = "cancellingTransactionID")]
    pub cancelling_transaction_id: Option<TransactionID>,
}

/// Transaction recorded when a stop-loss order request is rejected.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StopLossOrderRejectTransaction {
    pub id: TransactionID,
    pub time: DateTime<Utc>,
    #[serde(rename = "userID")]
    pub user_id: i64,
    #[serde(rename = "accountID")]
    pub account_id: AccountID,
    #[serde(rename = "batchID")]
    pub batch_id: TransactionID,
    #[serde(rename = "requestID")]
    pub request_id: Option<RequestID>,
    #[serde(rename = "tradeID")]
    pub trade_id: TradeID,
    #[serde(rename = "clientTradeID")]
    pub client_trade_id: Option<ClientID>,
    pub price: PriceValue,
    pub distance: Option<DecimalNumber>,
    pub time_in_force: TimeInForce,
    pub gtd_time: Option<DateTime<Utc>>,
    pub trigger_condition: OrderTriggerCondition,
    pub reason: StopLossOrderReason,
    pub client_extensions: Option<ClientExtensions>,
    #[serde(rename = "orderFillTransactionID")]
    pub order_fill_transaction_id: Option<TransactionID>,
    #[serde(rename = "replacesOrderID")]
    pub replaces_order_id: Option<OrderID>,
    #[serde(rename = "cancellingTransactionID")]
    pub cancelling_transaction_id: Option<TransactionID>,
    #[serde(rename = "intendedReplacesOrderID")]
    pub intended_replaces_order_id: Option<OrderID>,
    pub reject_reason: TransactionRejectReason,
}

/// Transaction recorded when a guaranteed stop-loss order (GSLO) is attached to or created for a trade.
///
/// Unlike a regular stop-loss, a GSLO guarantees the fill price at the specified level,
/// regardless of market gaps, in exchange for a premium.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GuaranteedStopLossOrderTransaction {
    pub id: TransactionID,
    pub time: DateTime<Utc>,
    #[serde(rename = "userID")]
    pub user_id: i64,
    #[serde(rename = "accountID")]
    pub account_id: AccountID,
    #[serde(rename = "batchID")]
    pub batch_id: TransactionID,
    #[serde(rename = "requestID")]
    pub request_id: Option<RequestID>,
    #[serde(rename = "tradeID")]
    pub trade_id: TradeID,
    #[serde(rename = "clientTradeID")]
    pub client_trade_id: Option<ClientID>,
    pub price: PriceValue,
    pub distance: Option<DecimalNumber>,
    pub time_in_force: TimeInForce,
    pub gtd_time: Option<DateTime<Utc>>,
    pub trigger_condition: OrderTriggerCondition,
    pub reason: GuaranteedStopLossOrderReason,
    pub client_extensions: Option<ClientExtensions>,
    #[serde(rename = "orderFillTransactionID")]
    pub order_fill_transaction_id: Option<TransactionID>,
    #[serde(rename = "replacesOrderID")]
    pub replaces_order_id: Option<OrderID>,
    #[serde(rename = "cancellingTransactionID")]
    pub cancelling_transaction_id: Option<TransactionID>,
}

/// Transaction recorded when a guaranteed stop-loss order request is rejected.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GuaranteedStopLossOrderRejectTransaction {
    pub id: TransactionID,
    pub time: DateTime<Utc>,
    #[serde(rename = "userID")]
    pub user_id: i64,
    #[serde(rename = "accountID")]
    pub account_id: AccountID,
    #[serde(rename = "batchID")]
    pub batch_id: TransactionID,
    #[serde(rename = "requestID")]
    pub request_id: Option<RequestID>,
    #[serde(rename = "tradeID")]
    pub trade_id: TradeID,
    #[serde(rename = "clientTradeID")]
    pub client_trade_id: Option<ClientID>,
    pub price: PriceValue,
    pub distance: Option<DecimalNumber>,
    pub time_in_force: TimeInForce,
    pub gtd_time: Option<DateTime<Utc>>,
    pub trigger_condition: OrderTriggerCondition,
    pub reason: GuaranteedStopLossOrderReason,
    pub client_extensions: Option<ClientExtensions>,
    #[serde(rename = "orderFillTransactionID")]
    pub order_fill_transaction_id: Option<TransactionID>,
    #[serde(rename = "replacesOrderID")]
    pub replaces_order_id: Option<OrderID>,
    #[serde(rename = "cancellingTransactionID")]
    pub cancelling_transaction_id: Option<TransactionID>,
    #[serde(rename = "intendedReplacesOrderID")]
    pub intended_replaces_order_id: Option<OrderID>,
    pub reject_reason: TransactionRejectReason,
}

/// Transaction recorded when a trailing stop-loss order is attached to or created for a trade.
///
/// The order trails the market price by the specified `distance`, locking in profit
/// as the price moves in the trade's favour.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TrailingStopLossOrderTransaction {
    pub id: TransactionID,
    pub time: DateTime<Utc>,
    #[serde(rename = "userID")]
    pub user_id: i64,
    #[serde(rename = "accountID")]
    pub account_id: AccountID,
    #[serde(rename = "batchID")]
    pub batch_id: TransactionID,
    #[serde(rename = "requestID")]
    pub request_id: Option<RequestID>,
    #[serde(rename = "tradeID")]
    pub trade_id: TradeID,
    #[serde(rename = "clientTradeID")]
    pub client_trade_id: Option<ClientID>,
    pub distance: DecimalNumber,
    pub time_in_force: TimeInForce,
    pub gtd_time: Option<DateTime<Utc>>,
    pub trigger_condition: OrderTriggerCondition,
    pub reason: TrailingStopLossOrderReason,
    pub client_extensions: Option<ClientExtensions>,
    #[serde(rename = "orderFillTransactionID")]
    pub order_fill_transaction_id: Option<TransactionID>,
    #[serde(rename = "replacesOrderID")]
    pub replaces_order_id: Option<OrderID>,
    #[serde(rename = "cancellingTransactionID")]
    pub cancelling_transaction_id: Option<TransactionID>,
}

/// Transaction recorded when a trailing stop-loss order request is rejected.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TrailingStopLossOrderRejectTransaction {
    pub id: TransactionID,
    pub time: DateTime<Utc>,
    #[serde(rename = "userID")]
    pub user_id: i64,
    #[serde(rename = "accountID")]
    pub account_id: AccountID,
    #[serde(rename = "batchID")]
    pub batch_id: TransactionID,
    #[serde(rename = "requestID")]
    pub request_id: Option<RequestID>,
    #[serde(rename = "tradeID")]
    pub trade_id: TradeID,
    #[serde(rename = "clientTradeID")]
    pub client_trade_id: Option<ClientID>,
    pub distance: DecimalNumber,
    pub time_in_force: TimeInForce,
    pub gtd_time: Option<DateTime<Utc>>,
    pub trigger_condition: OrderTriggerCondition,
    pub reason: TrailingStopLossOrderReason,
    pub client_extensions: Option<ClientExtensions>,
    #[serde(rename = "orderFillTransactionID")]
    pub order_fill_transaction_id: Option<TransactionID>,
    #[serde(rename = "replacesOrderID")]
    pub replaces_order_id: Option<OrderID>,
    #[serde(rename = "cancellingTransactionID")]
    pub cancelling_transaction_id: Option<TransactionID>,
    #[serde(rename = "intendedReplacesOrderID")]
    pub intended_replaces_order_id: Option<OrderID>,
    pub reject_reason: TransactionRejectReason,
}

/// Transaction recorded when an order is filled and a trade is opened, closed, or reduced.
///
/// This is the primary transaction for all trade activity. Fields such as `trade_opened`,
/// `trades_closed`, and `trade_reduced` describe what changed as a result of the fill.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OrderFillTransaction {
    pub id: TransactionID,
    pub time: DateTime<Utc>,
    #[serde(rename = "userID")]
    pub user_id: i64,
    #[serde(rename = "accountID")]
    pub account_id: AccountID,
    #[serde(rename = "batchID")]
    pub batch_id: TransactionID,
    #[serde(rename = "requestID")]
    pub request_id: Option<RequestID>,
    #[serde(rename = "orderID")]
    pub order_id: OrderID,
    #[serde(rename = "clientOrderID")]
    pub client_order_id: Option<ClientID>,
    pub instrument: InstrumentName,
    pub units: DecimalNumber,
    pub home_conversion_factors: HomeConversionFactors,
    #[serde(rename = "fullVWAP")]
    pub full_vwap: PriceValue,
    pub full_price: ClientPrice,
    pub reason: OrderFillReason,
    pub pl: AccountUnits,
    pub financing: AccountUnits,
    pub base_financing: AccountUnits,
    pub quote_financing: Option<AccountUnits>,
    pub commission: AccountUnits,
    pub guaranteed_execution_fee: AccountUnits,
    pub quote_guaranteed_execution_fee: AccountUnits,
    pub half_spread_cost: AccountUnits,
    pub account_balance: AccountUnits,
    pub trade_opened: Option<TradeOpen>,
    pub trades_closed: Option<Vec<TradeReduce>>,
    pub trade_reduced: Option<TradeReduce>,
}

/// Transaction recorded when a pending order is cancelled.
#[derive(Debug, Serialize, Deserialize)]
pub struct OrderCancelTransaction {
    pub id: TransactionID,
    pub time: DateTime<Utc>,
    #[serde(rename = "userID")]
    pub user_id: i64,
    #[serde(rename = "accountID")]
    pub account_id: AccountID,
    #[serde(rename = "batchID")]
    pub batch_id: TransactionID,
    #[serde(rename = "requestID")]
    pub request_id: Option<RequestID>,
    #[serde(rename = "orderID")]
    pub order_id: OrderID,
    #[serde(rename = "clientOrderID")]
    pub client_order_id: Option<ClientID>,
    pub reason: OrderCancelReason,
    #[serde(rename = "replacedByOrderID")]
    pub replaced_by_order_id: Option<OrderID>,
}

/// Transaction recorded when an order cancellation request is rejected.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OrderCancelRejectTransaction {
    pub id: TransactionID,
    pub time: DateTime<Utc>,
    #[serde(rename = "userID")]
    pub user_id: i64,
    #[serde(rename = "accountID")]
    pub account_id: AccountID,
    #[serde(rename = "batchID")]
    pub batch_id: TransactionID,
    #[serde(rename = "requestID")]
    pub request_id: Option<RequestID>,
    #[serde(rename = "orderID")]
    pub order_id: OrderID,
    #[serde(rename = "clientOrderID")]
    pub client_order_id: Option<ClientID>,
    pub reject_reason: TransactionRejectReason,
}

/// Transaction recorded when the client extensions on an order are successfully updated.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OrderClientExtensionsModifyTransaction {
    pub id: TransactionID,
    pub time: DateTime<Utc>,
    #[serde(rename = "userID")]
    pub user_id: i64,
    #[serde(rename = "accountID")]
    pub account_id: AccountID,
    #[serde(rename = "batchID")]
    pub batch_id: TransactionID,
    #[serde(rename = "requestID")]
    pub request_id: Option<RequestID>,
    #[serde(rename = "orderID")]
    pub order_id: OrderID,
    #[serde(rename = "clientOrderID")]
    pub client_order_id: Option<ClientID>,
    pub client_extensions_modify: Option<ClientExtensions>,
    pub trade_client_extensions_modify: Option<ClientExtensions>,
}

/// Transaction recorded when an order client-extensions modification request is rejected.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OrderClientExtensionsModifyRejectTransaction {
    pub id: TransactionID,
    pub time: DateTime<Utc>,
    #[serde(rename = "userID")]
    pub user_id: i64,
    #[serde(rename = "accountID")]
    pub account_id: AccountID,
    #[serde(rename = "batchID")]
    pub batch_id: TransactionID,
    #[serde(rename = "requestID")]
    pub request_id: Option<RequestID>,
    #[serde(rename = "orderID")]
    pub order_id: OrderID,
    #[serde(rename = "clientOrderID")]
    pub client_order_id: Option<ClientID>,
    pub client_extensions_modify: Option<ClientExtensions>,
    pub trade_client_extensions_modify: Option<ClientExtensions>,
    pub reject_reason: TransactionRejectReason,
}

/// Transaction recorded when the client extensions on a trade are successfully updated.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TradeClientExtensionsModifyTransaction {
    pub id: TransactionID,
    pub time: DateTime<Utc>,
    #[serde(rename = "userID")]
    pub user_id: i64,
    #[serde(rename = "accountID")]
    pub account_id: AccountID,
    #[serde(rename = "batchID")]
    pub batch_id: TransactionID,
    #[serde(rename = "requestID")]
    pub request_id: Option<RequestID>,
    #[serde(rename = "tradeID")]
    pub trade_id: TradeID,
    #[serde(rename = "clientTradeID")]
    pub client_trade_id: Option<ClientID>,
    pub trade_client_extensions_modify: Option<ClientExtensions>,
}

/// Transaction recorded when a trade client-extensions modification request is rejected.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TradeClientExtensionsModifyRejectTransaction {
    pub id: TransactionID,
    pub time: DateTime<Utc>,
    #[serde(rename = "userID")]
    pub user_id: i64,
    #[serde(rename = "accountID")]
    pub account_id: AccountID,
    #[serde(rename = "batchID")]
    pub batch_id: TransactionID,
    #[serde(rename = "requestID")]
    pub request_id: Option<RequestID>,
    #[serde(rename = "tradeID")]
    pub trade_id: TradeID,
    #[serde(rename = "clientTradeID")]
    pub client_trade_id: Option<ClientID>,
    pub trade_client_extensions_modify: Option<ClientExtensions>,
    pub reject_reason: TransactionRejectReason,
}

/// Transaction recorded when the account enters a margin call state.
///
/// OANDA will begin closing positions if the account margin level continues to fall.
#[derive(Debug, Serialize, Deserialize)]
pub struct MarginCallEnterTransaction {
    pub id: TransactionID,
    pub time: DateTime<Utc>,
    #[serde(rename = "userID")]
    pub user_id: i64,
    #[serde(rename = "accountID")]
    pub account_id: AccountID,
    #[serde(rename = "batchID")]
    pub batch_id: TransactionID,
    #[serde(rename = "requestID")]
    pub request_id: Option<RequestID>,
}

/// Transaction recorded each time the margin call duration is extended.
///
/// OANDA allows a grace period before forced liquidation; each extension is recorded here.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MarginCallExtendTransaction {
    pub id: TransactionID,
    pub time: DateTime<Utc>,
    #[serde(rename = "userID")]
    pub user_id: i64,
    #[serde(rename = "accountID")]
    pub account_id: AccountID,
    #[serde(rename = "batchID")]
    pub batch_id: TransactionID,
    #[serde(rename = "requestID")]
    pub request_id: Option<RequestID>,
    /// The number of times this margin call has been extended.
    pub extension_number: Option<i64>,
}

/// Transaction recorded when the account exits the margin call state
/// (either by depositing funds or by reducing exposure).
#[derive(Debug, Serialize, Deserialize)]
pub struct MarginCallExitTransaction {
    pub id: TransactionID,
    pub time: DateTime<Utc>,
    #[serde(rename = "userID")]
    pub user_id: i64,
    #[serde(rename = "accountID")]
    pub account_id: AccountID,
    #[serde(rename = "batchID")]
    pub batch_id: TransactionID,
    #[serde(rename = "requestID")]
    pub request_id: Option<RequestID>,
}

/// Transaction recorded when one or more trades are queued for closure at the next
/// tradeable price (e.g. after a market halt or margin closeout during non-trading hours).
#[derive(Debug, Serialize, Deserialize)]
pub struct DelayedTradeClosureTransaction {
    pub id: TransactionID,
    pub time: DateTime<Utc>,
    #[serde(rename = "userID")]
    pub user_id: i64,
    #[serde(rename = "accountID")]
    pub account_id: AccountID,
    #[serde(rename = "batchID")]
    pub batch_id: TransactionID,
    #[serde(rename = "requestID")]
    pub request_id: Option<RequestID>,
    pub reason: MarketOrderReason,
    #[serde(rename = "tradeIDs")]
    pub trade_ids: Vec<TradeID>,
}

/// Transaction recorded at the end of each trading day when financing (swap/rollover)
/// charges or credits are applied to all open positions.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DailyFinancingTransaction {
    pub id: TransactionID,
    pub time: DateTime<Utc>,
    #[serde(rename = "userID")]
    pub user_id: i64,
    #[serde(rename = "accountID")]
    pub account_id: AccountID,
    #[serde(rename = "batchID")]
    pub batch_id: TransactionID,
    #[serde(rename = "requestID")]
    pub request_id: Option<RequestID>,
    pub financing: AccountUnits,
    pub account_balance: AccountUnits,
    pub position_financings: Vec<PositionFinancing>,
}

/// Transaction recorded when a dividend adjustment is applied to open CFD positions
/// in an instrument that has paid a dividend.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DividendAdjustmentTransaction {
    pub id: TransactionID,
    pub time: DateTime<Utc>,
    #[serde(rename = "userID")]
    pub user_id: i64,
    #[serde(rename = "accountID")]
    pub account_id: AccountID,
    #[serde(rename = "batchID")]
    pub batch_id: TransactionID,
    #[serde(rename = "requestID")]
    pub request_id: Option<RequestID>,
    pub instrument: InstrumentName,
    pub dividend_adjustment: AccountUnits,
    pub quote_dividend_adjustment: AccountUnits,
    pub home_conversion_factors: HomeConversionFactors,
    pub account_balance: AccountUnits,
    pub open_trade_dividend_adjustments: Vec<OpenTradeDividendAdjustment>,
}

/// Transaction recorded when the account's resettable P&L is reset to zero.
///
/// Clients can request a P&L reset to restart P&L tracking from the current balance.
#[derive(Debug, Serialize, Deserialize)]
pub struct ResetResettablePLTransaction {
    pub id: TransactionID,
    pub time: DateTime<Utc>,
    #[serde(rename = "userID")]
    pub user_id: i64,
    #[serde(rename = "accountID")]
    pub account_id: AccountID,
    #[serde(rename = "batchID")]
    pub batch_id: TransactionID,
    #[serde(rename = "requestID")]
    pub request_id: Option<RequestID>,
}

/// All transaction types that can appear on an OANDA account.
///
/// Used as a filter value when listing transactions via [`ListTransactionsRequest`].
#[derive(Debug, Serialize, Deserialize, Display)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[strum(serialize_all = "SCREAMING_SNAKE_CASE")]
pub enum TransactionType {
    Create,
    Close,
    Reopen,
    ClientConfigure,
    ClientConfigureReject,
    TransferFunds,
    TransferFundsReject,
    MarketOrder,
    MarketOrderReject,
    FixedPriceOrder,
    LimitOrder,
    LimitOrderReject,
    StopOrder,
    StopOrderReject,
    MarketIfTouchedOrder,
    MarketIfTouchedOrderReject,
    TakeProfitOrder,
    TakeProfitOrderReject,
    StopLossOrder,
    StopLossOrderReject,
    GuaranteedStopLossOrder,
    GuaranteedStopLossOrderReject,
    TrailingStopLossOrder,
    TrailingStopLossOrderReject,
    OrderFill,
    OrderCancel,
    OrderCancelReject,
    OrderClientExtensionsModify,
    OrderClientExtensionsModifyReject,
    TradeClientExtensionsModify,
    TradeClientExtensionsModifyReject,
    MarginCallEnter,
    MarginCallExtend,
    MarginCallExit,
    DelayedTradeClosure,
    DailyFinancing,
    DividendAdjustment,
    ResetResettablePL,
}

/// Why a funds transfer occurred on an account.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum FundingReason {
    /// The client explicitly deposited or withdrew funds.
    ClientFunding,
    /// Funds were moved between accounts under the same owner.
    AccountTransfer,
    /// Funds were transferred as part of a division migration.
    DivisionMigration,
    /// Funds were transferred as part of a site migration.
    SiteMigration,
    /// An administrative adjustment was applied to the account balance.
    Adjustment,
}

/// Why a market order was created.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum MarketOrderReason {
    /// Submitted directly by the client.
    ClientOrder,
    /// Created automatically to close a specific trade.
    TradeClose,
    /// Created automatically to close out a position.
    PositionCloseout,
    /// Created automatically by OANDA to reduce exposure during a margin call.
    MarginCloseout,
    /// Created to close a trade that was deferred due to the market being closed.
    DelayedTradeClose,
}

/// Why a fixed-price order was created by OANDA internally.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum FixedPriceOrderReason {
    /// Created to migrate open trades during a platform account migration.
    PlatformAccountMigration,
    /// Created to close a trade during a division account migration.
    TradeCloseDivisionAccountMigration,
    /// Created to close a trade as part of an administrative action.
    TradeCloseAdministrativeAction,
}

/// Why a limit order was created.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum LimitOrderReason {
    /// Submitted directly by the client.
    ClientOrder,
    /// Created to replace a previously cancelled order.
    Replacement,
}

/// Why a stop order was created.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum StopOrderReason {
    /// Submitted directly by the client.
    ClientOrder,
    /// Created to replace a previously cancelled order.
    Replacement,
}

/// Why a market-if-touched order was created.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum MarketIfTouchedOrderReason {
    /// Submitted directly by the client.
    ClientOrder,
    /// Created to replace a previously cancelled order.
    Replacement,
}

/// Why a take-profit order was created.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TakeProfitOrderReason {
    /// Submitted directly by the client.
    ClientOrder,
    /// Created to replace an existing take-profit on the same trade.
    Replacement,
    /// Created automatically when a trade was opened with `take_profit_on_fill` set.
    OnFill,
}

/// Why a stop-loss order was created.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum StopLossOrderReason {
    /// Submitted directly by the client.
    ClientOrder,
    /// Created to replace an existing stop-loss on the same trade.
    Replacement,
    /// Created automatically when a trade was opened with `stop_loss_on_fill` set.
    OnFill,
}

/// Why a guaranteed stop-loss order was created.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum GuaranteedStopLossOrderReason {
    /// Submitted directly by the client.
    ClientOrder,
    /// Created to replace an existing guaranteed stop-loss on the same trade.
    Replacement,
    /// Created automatically when a trade was opened with `guaranteed_stop_loss_on_fill` set.
    OnFill,
}

/// Why a trailing stop-loss order was created.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TrailingStopLossOrderReason {
    /// Submitted directly by the client.
    ClientOrder,
    /// Created to replace an existing trailing stop-loss on the same trade.
    Replacement,
    /// Created automatically when a trade was opened with `trailing_stop_loss_on_fill` set.
    OnFill,
}

/// Why an order was filled (what type of order triggered the fill).
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum OrderFillReason {
    LimitOrder,
    StopOrder,
    MarketIfTouchedOrder,
    TakeProfitOrder,
    StopLossOrder,
    GuaranteedStopLossOrder,
    TrailingStopLossOrder,
    MarketOrder,
    MarketOrderTradeClose,
    MarketOrderPositionCloseout,
    MarketOrderMarginCloseout,
    MarketOrderDelayedTradeClose,
    FixedPriceOrder,
    FixedPriceOrderPlatformAccountMigration,
    FixedPriceOrderDivisionAccountMigration,
    FixedPriceOrderAdministrativeAction,
}

/// Why a pending order was cancelled.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum OrderCancelReason {
    InternalServerError,
    AccountLocked,
    AccountNewPositionsLocked,
    AccountOrderCreationLocked,
    AccountOrderFillLocked,
    ClientRequest,
    Migration,
    MarketHalted,
    LinkedTradeClosed,
    TimeInForceExpired,
    InsufficientMargin,
    FifoViolation,
    BoundsViolation,
    ClientRequestReplaced,
    InsufficientLiquidity,
    TakeProfitOnFillGtdTimestampInPast,
    TakeProfitOnFillLoss,
    LosingTakeProfit,
    StopLossOnFillGtdTimestampInPast,
    StopLossOnFillLoss,
    StopLossOnFillPriceDistanceMaximumExceeded,
    StopLossOnFillRequired,
    StopLossOnFillGuaranteedRequired,
    StopLossOnFillGuaranteedNotAllowed,
    StopLossOnFillGuaranteedMinimumDistanceNotMet,
    StopLossOnFillGuaranteedLevelRestrictionExceeded,
    StopLossOnFillGuaranteedHedgingNotAllowed,
    StopLossOnFillTimeInForceInvalid,
    StopLossOnFillTriggerConditionInvalid,
    TakeProfitOnFillPriceDistanceMaximumExceeded,
    TrailingStopLossOnFillGtdTimestampInPast,
    ClientTradeIdAlreadyExists,
    PositionCloseoutFailed,
    OpenTradesAllowedExceeded,
    PendingOrdersAllowedExceeded,
    TakeProfitOnFillClientOrderIdAlreadyExists,
    StopLossOnFillClientOrderIdAlreadyExists,
    TrailingStopLossOnFillClientOrderIdAlreadyExists,
    PositionSizeExceeded,
    HedgingGsloViolation,
    AccountPositionValueLimitExceeded,
    InstrumentBidReduceOnly,
    InstrumentAskReduceOnly,
    InstrumentBidHalted,
    InstrumentAskHalted,
    StopLossOnFillGuaranteedBidHalted,
    StopLossOnFillGuaranteedAskHalted,
}

/// How financing (swap/rollover) is applied to open positions on an account.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AccountFinancingMode {
    /// No financing is applied (typically for spread-bet accounts).
    NoFinancing,
    /// Financing accrues every second and is settled daily.
    SecondBySecond,
    /// Financing is computed and applied once per day.
    Daily,
}

/// A coarser filter for querying transactions by category.
///
/// Used with [`GetTransactionsByIDRangeRequest`] and [`GetTransactionsBySinceIDRequest`]
/// to narrow results to a subset of transaction types. Variants like [`Order`](Self::Order)
/// and [`Funding`](Self::Funding) aggregate multiple [`TransactionType`] values.
#[derive(Debug, Serialize, Deserialize, Display)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[strum(serialize_all = "SCREAMING_SNAKE_CASE")]
pub enum TransactionFilter {
    Order,
    Funding,
    Admin,
    Create,
    Close,
    Reopen,
    ClientConfigure,
    ClientConfigureReject,
    TransferFunds,
    TransferFundsReject,
    MarketOrder,
    MarketOrderReject,
    LimitOrder,
    LimitOrderReject,
    StopOrder,
    StopOrderReject,
    MarketIfTouchedOrder,
    MarketIfTouchedOrderReject,
    TakeProfitOrder,
    TakeProfitOrderReject,
    StopLossOrder,
    StopLossOrderReject,
    TrailingStopLossOrder,
    TrailingStopLossOrderReject,
    OneCancelsAllOrder,
    OneCancelsAllOrderReject,
    OneCancelsAllOrderTriggered,
    OrderFill,
    OrderCancel,
    OrderCancelReject,
    OrderClientExtensionsModify,
    OrderClientExtensionsModifyReject,
    TradeClientExtensionsModify,
    TradeClientExtensionsModifyReject,
    MarginCallEnter,
    MarginCallExtend,
    MarginCallExit,
    DelayedTradeClosure,
    DailyFinancing,
    ResetResettablePL,
}

/// Machine-readable reason why a transaction was rejected by the OANDA API.
///
/// Returned in the `reject_reason` field of every `*RejectTransaction` struct.
/// The variant names are self-describing; consult the OANDA v20 REST API docs for
/// the precise conditions under which each variant is returned.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TransactionRejectReason {
    InternalServerError,
    InstrumentPriceUnknown,
    AccountNotActive,
    AccountLocked,
    AccountOrderCreationLocked,
    AccountConfigurationLocked,
    AccountDepositLocked,
    AccountWithdrawalLocked,
    AccountOrderCancelLocked,
    InstrumentNotTradeable,
    PendingOrdersAllowedExceeded,
    OrderIdUnspecified,
    OrderDoesntExist,
    OrderIdentifierInconsistency,
    TradeIdUnspecified,
    TradeDoesntExist,
    TradeIdentifierInconsistency,
    InsufficientMargin,
    InstrumentMissing,
    InstrumentUnknown,
    UnitsMissing,
    UnitsInvalid,
    UnitsPrecisionExceeded,
    UnitsLimitExceeded,
    UnitsMinimumNotMet,
    PriceMissing,
    PriceInvalid,
    PricePrecisionExceeded,
    PriceDistanceMissing,
    PriceDistanceInvalid,
    PriceDistancePrecisionExceeded,
    PriceDistanceMaximumExceeded,
    PriceDistanceMinimumNotMet,
    TimeInForceMissing,
    TimeInForceInvalid,
    TimeInForceGtdTimestampMissing,
    TimeInForceGtdTimestampInPast,
    PriceBoundInvalid,
    PriceBoundPrecisionExceeded,
    OrdersOnFillDuplicateClientOrderIds,
    TradeOnFillClientExtensionsNotSupported,
    ClientOrderIdInvalid,
    ClientOrderIdAlreadyExists,
    ClientOrderTagInvalid,
    ClientOrderCommentInvalid,
    ClientTradeIdInvalid,
    ClientTradeIdAlreadyExists,
    ClientTradeTagInvalid,
    ClientTradeCommentInvalid,
    OrderFillPositionActionMissing,
    OrderFillPositionActionInvalid,
    TriggerConditionMissing,
    TriggerConditionInvalid,
    OrderPartialFillOptionMissing,
    OrderPartialFillOptionInvalid,
    InvalidReissueImmediatePartialFill,
    TakeProfitOrderAlreadyExists,
    TakeProfitOnFillPriceMissing,
    TakeProfitOnFillPriceInvalid,
    TakeProfitOnFillPricePrecisionExceeded,
    TakeProfitOnFillTimeInForceMissing,
    TakeProfitOnFillTimeInForceInvalid,
    TakeProfitOnFillGtdTimestampMissing,
    TakeProfitOnFillGtdTimestampInPast,
    TakeProfitOnFillClientOrderIdInvalid,
    TakeProfitOnFillClientOrderTagInvalid,
    TakeProfitOnFillClientOrderCommentInvalid,
    TakeProfitOnFillTriggerConditionMissing,
    TakeProfitOnFillTriggerConditionInvalid,
    StopLossOrderAlreadyExists,
    StopLossOrderGuaranteedRequired,
    StopLossOrderGuaranteedPriceWithinSpread,
    StopLossOrderGuaranteedNotAllowed,
    StopLossOrderGuaranteedHaltedCreateViolation,
    StopLossOrderGuaranteedHaltedTightenViolation,
    StopLossOrderGuaranteedHedgingNotAllowed,
    StopLossOrderGuaranteedMinimumDistanceNotMet,
    StopLossOrderNotCancelable,
    StopLossOrderNotReplaceable,
    StopLossOrderGuaranteedLevelRestrictionExceeded,
    StopLossOrderPriceAndDistanceBothSpecified,
    StopLossOrderPriceAndDistanceBothMissing,
    StopLossOnFillRequiredForPendingOrder,
    StopLossOnFillGuaranteedNotAllowed,
    StopLossOnFillGuaranteedRequired,
    StopLossOnFillPriceMissing,
    StopLossOnFillPriceInvalid,
    StopLossOnFillPricePrecisionExceeded,
    StopLossOnFillGuaranteedMinimumDistanceNotMet,
    StopLossOnFillGuaranteedLevelRestrictionExceeded,
    StopLossOnFillDistanceInvalid,
    StopLossOnFillPriceDistanceMaximumExceeded,
    StopLossOnFillDistancePrecisionExceeded,
    StopLossOnFillPriceAndDistanceBothSpecified,
    StopLossOnFillPriceAndDistanceBothMissing,
    StopLossOnFillTimeInForceMissing,
    StopLossOnFillTimeInForceInvalid,
    StopLossOnFillGtdTimestampMissing,
    StopLossOnFillGtdTimestampInPast,
    StopLossOnFillClientOrderIdInvalid,
    StopLossOnFillClientOrderTagInvalid,
    StopLossOnFillClientOrderCommentInvalid,
    StopLossOnFillTriggerConditionMissing,
    StopLossOnFillTriggerConditionInvalid,
    TrailingStopLossOrderAlreadyExists,
    TrailingStopLossOnFillPriceDistanceMissing,
    TrailingStopLossOnFillPriceDistanceInvalid,
    TrailingStopLossOnFillPriceDistancePrecisionExceeded,
    TrailingStopLossOnFillPriceDistanceMaximumExceeded,
    TrailingStopLossOnFillPriceDistanceMinimumNotMet,
    TrailingStopLossOnFillTimeInForceMissing,
    TrailingStopLossOnFillTimeInForceInvalid,
    TrailingStopLossOnFillGtdTimestampMissing,
    TrailingStopLossOnFillGtdTimestampInPast,
    TrailingStopLossOnFillClientOrderIdInvalid,
    TrailingStopLossOnFillClientOrderTagInvalid,
    TrailingStopLossOnFillClientOrderCommentInvalid,
    TrailingStopLossOrdersNotSupported,
    TrailingStopLossOnFillTriggerConditionMissing,
    TrailingStopLossOnFillTriggerConditionInvalid,
    CloseTradeTypeMissing,
    CloseTradePartialUnitsMissing,
    CloseTradeUnitsExceedTradeSize,
    CloseoutPositionDoesntExist,
    CloseoutPositionIncompleteSpecification,
    CloseoutPositionUnitsExceedPositionSize,
    CloseoutPositionReject,
    CloseoutPositionPartialUnitsMissing,
    MarkupGroupIdInvalid,
    PositionAggregationModeInvalid,
    AdminConfigureDataMissing,
    MarginRateInvalid,
    MarginRateWouldTriggerCloseout,
    AliasInvalid,
    ClientConfigureDataMissing,
    MarginRateWouldTriggerMarginCall,
    AmountInvalid,
    InsufficientFunds,
    AmountMissing,
    FundingReasonMissing,
    ClientExtensionsDataMissing,
    ReplacingOrderInvalid,
    ReplacingTradeIdInvalid,
}

/// Details of the trade that a market order was intended to close.
///
/// Present in [`MarketOrderTransaction`] when the order was created with
/// `reason = TradeClose`.
#[derive(Debug, Serialize, Deserialize)]
pub struct MarketOrderTradeClose {
    /// The trade being closed.
    #[serde(rename = "tradeID")]
    pub trade_id: TradeID,
    /// Number of units of the trade being closed (`"ALL"` for a full close).
    pub units: String,
}

/// Details of a margin closeout that triggered a market order.
///
/// Present in [`MarketOrderTransaction`] when the order was created with
/// `reason = MarginCloseout`.
#[derive(Debug, Serialize, Deserialize)]
pub struct MarketOrderMarginCloseout {
    /// The specific rule that triggered the margin closeout.
    pub reason: MarketOrderMarginCloseoutReason,
}

/// Why a margin closeout market order was generated.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum MarketOrderMarginCloseoutReason {
    /// The account's margin level fell below the margin check threshold.
    MarginCheckViolation,
    /// A regulatory margin call was triggered.
    RegulatoryMarginCallViolation,
    /// A regulatory margin check violation occurred.
    RegulatoryMarginCheckViolation,
}

/// Details of a trade that is being closed on a delayed basis (e.g. after a market halt).
///
/// Present in [`MarketOrderTransaction`] when the order was created with
/// `reason = DelayedTradeClose`.
#[derive(Debug, Serialize, Deserialize)]
pub struct MarketOrderDelayedTradeClose {
    /// The trade scheduled for delayed closure.
    #[serde(rename = "tradeID")]
    pub trade_id: TradeID,
    /// ID of the [`DelayedTradeClosureTransaction`] that originally queued this closure.
    #[serde(rename = "sourceTransactionID")]
    pub source_transaction_id: TransactionID,
}

/// Details of a position that a market order was intended to close out.
///
/// Present in [`MarketOrderTransaction`] when the order was created with
/// `reason = PositionCloseout`.
#[derive(Debug, Serialize, Deserialize)]
pub struct MarketOrderPositionCloseout {
    /// The instrument whose position is being closed.
    pub instrument: InstrumentName,
    /// Number of units being closed (`"ALL"` for a full closeout).
    pub units: String,
}

/// Parameters for a take-profit order to be created when a trade opens.
///
/// Embedded in order-creation request types via the `take_profit_on_fill` field.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TakeProfitDetails {
    /// Price at which the take-profit triggers.
    pub price: PriceValue,
    /// How long the order remains active (`GTC`, `GTD`, or `GFD`).
    pub time_in_force: TimeInForce,
    /// Expiry timestamp when `time_in_force` is `GTD`.
    pub gtd_time: Option<DateTime<Utc>>,
    /// Optional client extensions to attach to the created take-profit order.
    pub client_extensions: Option<ClientExtensions>,
}

/// Parameters for a stop-loss order to be created when a trade opens.
///
/// Embedded in order-creation request types via the `stop_loss_on_fill` field.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StopLossDetails {
    /// Absolute price level at which the stop-loss triggers. Mutually exclusive with `distance`.
    pub price: PriceValue,
    /// Distance in price units from the trade price at which the stop-loss triggers.
    /// Mutually exclusive with `price`.
    pub distance: Option<DecimalNumber>,
    /// How long the order remains active.
    pub time_in_force: TimeInForce,
    /// Expiry timestamp when `time_in_force` is `GTD`.
    pub gtd_time: Option<DateTime<Utc>>,
    /// Optional client extensions to attach to the created stop-loss order.
    pub client_extensions: Option<ClientExtensions>,
}

/// Parameters for a trailing stop-loss order to be created when a trade opens.
///
/// Embedded in order-creation request types via the `trailing_stop_loss_on_fill` field.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TrailingStopLossDetails {
    /// Distance in price units that the trailing stop follows behind the best price.
    pub distance: DecimalNumber,
    /// How long the order remains active.
    pub time_in_force: TimeInForce,
    /// Expiry timestamp when `time_in_force` is `GTD`.
    pub gtd_time: Option<DateTime<Utc>>,
    /// Optional client extensions to attach to the created trailing stop-loss order.
    pub client_extensions: Option<ClientExtensions>,
}

/// Parameters for a guaranteed stop-loss order to be created when a trade opens.
///
/// Embedded in order-creation request types via the `guaranteed_stop_loss_on_fill` field.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GuaranteedStopLossDetails {
    /// Absolute price level at which the guaranteed stop triggers. Mutually exclusive with `distance`.
    pub price: PriceValue,
    /// Distance in price units from the trade price. Mutually exclusive with `price`.
    pub distance: Option<DecimalNumber>,
    /// How long the order remains active.
    pub time_in_force: TimeInForce,
    /// Expiry timestamp when `time_in_force` is `GTD`.
    pub gtd_time: Option<DateTime<Utc>>,
    /// Optional client extensions to attach to the created guaranteed stop-loss order.
    pub client_extensions: Option<ClientExtensions>,
}

/// Details of a trade that was opened as a result of an order fill.
///
/// Present in [`OrderFillTransaction::trade_opened`] when the fill created a new trade.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TradeOpen {
    /// The newly opened trade's ID.
    #[serde(rename = "tradeID")]
    pub trade_id: TradeID,
    /// Number of units opened. Positive = long, negative = short.
    pub units: DecimalNumber,
    /// The price at which the trade was opened.
    pub price: PriceValue,
    /// Fee charged for guaranteed execution, in home currency units.
    pub guaranteed_execution_fee: AccountUnits,
    /// Optional client extensions attached to the trade at open time.
    pub client_extensions: Option<ClientExtensions>,
    /// Half of the bid-ask spread cost at the time of opening, in home currency units.
    pub half_spread_cost: AccountUnits,
    /// Margin required to hold the newly opened units, in home currency units.
    pub initial_margin_required: AccountUnits,
}

/// Details of a trade that was fully or partially closed as a result of an order fill.
///
/// Present in [`OrderFillTransaction::trades_closed`] (full close) or
/// [`OrderFillTransaction::trade_reduced`] (partial close).
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TradeReduce {
    /// The trade that was closed or reduced.
    #[serde(rename = "tradeID")]
    pub trade_id: TradeID,
    /// Number of units closed. Always positive.
    pub units: DecimalNumber,
    /// Price at which the units were closed. `None` for administrative closures.
    pub price: Option<PriceValue>,
    /// Realised profit/loss from this closure, in home currency units.
    #[serde(rename = "realizedPL")]
    pub realized_pl: AccountUnits,
    /// Financing applied to the closed units, in home currency units.
    pub financing: AccountUnits,
    /// Guaranteed execution fee applicable to this closure, if any.
    pub guaranteed_execution_fee: Option<AccountUnits>,
    /// Half spread cost for this closure, in home currency units.
    pub half_spread_cost: Option<AccountUnits>,
}

/// Financing applied to a single open trade as part of a [`DailyFinancingTransaction`].
#[derive(Debug, Serialize, Deserialize)]
pub struct OpenTradeFinancing {
    /// The trade to which financing was applied.
    #[serde(rename = "tradeID")]
    pub trade_id: TradeID,
    /// Financing amount applied, in home currency units. Negative = charge, positive = credit.
    pub financing: AccountUnits,
}

/// Financing applied to all open trades in a single instrument position,
/// as part of a [`DailyFinancingTransaction`].
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PositionFinancing {
    /// The instrument whose position received financing.
    pub instrument: InstrumentName,
    /// Total financing for this instrument, in home currency units.
    pub financing: AccountUnits,
    /// Per-trade breakdown of the financing applied.
    pub open_trade_financings: Option<Vec<OpenTradeFinancing>>,
}

/// Dividend adjustment applied to a single open trade as part of a
/// [`DividendAdjustmentTransaction`].
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OpenTradeDividendAdjustment {
    /// The trade to which the dividend adjustment was applied.
    #[serde(rename = "tradeID")]
    pub trade_id: TradeID,
    /// Dividend adjustment amount, in home currency units.
    pub dividend_adjustment: AccountUnits,
}

/// A keepalive message emitted on the transaction stream when no transactions
/// have occurred recently.
///
/// Consumed as part of [`TransactionStreamItem::HEARTBEAT`].
#[derive(Debug, Serialize, Deserialize)]
pub struct TransactionHeartbeat {
    /// Timestamp of the heartbeat.
    pub time: DateTime<Utc>,
    /// ID of the most recent transaction at the time this heartbeat was sent.
    #[serde(rename = "lastTransactionID")]
    pub last_transaction_id: TransactionID,
}

// ---------------------------------------------------------------------------
// Client Extensions
// ---------------------------------------------------------------------------

/// Optional client-supplied metadata that can be attached to orders and trades.
///
/// All three fields are optional and are omitted from serialization when `None`,
/// so that a `PUT` with a partially populated `ClientExtensions` only updates the
/// provided fields and clears any field explicitly set to `None` on the server.
///
/// # Example
///
/// ```rust,ignore
/// let ext = ClientExtensions::new()
///     .id("my-trade-001".to_string())
///     .tag("strategy-A".to_string());
/// ```
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct ClientExtensions {
    /// Client-assigned identifier for the order or trade.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<ClientID>,
    /// Client-assigned tag for grouping related orders or trades.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tag: Option<ClientTag>,
    /// Free-text comment.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comment: Option<ClientComment>,
}

impl ClientExtensions {
    /// Creates an empty `ClientExtensions` with all fields set to `None`.
    pub fn new() -> Self {
        Self::default()
    }

    request_option_setter!(id, ClientID);
    request_option_setter!(tag, ClientTag);
    request_option_setter!(comment, ClientComment);
}

// ---------------------------------------------------------------------------
// Request/Response types
// ---------------------------------------------------------------------------

/// Request parameters for `GET /v3/accounts/{accountID}/transactions`.
///
/// All fields are optional. Call builder methods to set filters before passing
/// to [`TransactionService::list`].
#[derive(Default)]
pub struct ListTransactionsRequest {
    /// Return only transactions at or after this timestamp.
    pub from: Option<DateTime<Utc>>,
    /// Return only transactions at or before this timestamp.
    pub to: Option<DateTime<Utc>>,
    /// Maximum number of transactions per page (server default applies when `None`).
    pub page_size: Option<u16>,
    /// Restrict results to these transaction types. Empty = all types.
    pub transaction_type: Vec<TransactionType>,
}

impl ListTransactionsRequest {
    /// Creates a new request with no filters applied.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the earliest timestamp for transactions to include.
    pub fn from_time(mut self, from: DateTime<Utc>) -> Self {
        self.from = Some(from);
        self
    }

    pub fn to(mut self, to: DateTime<Utc>) -> Self {
        self.to = Some(to);
        self
    }

    pub fn page_size(mut self, page_size: u16) -> Self {
        self.page_size = Some(page_size);
        self
    }

    pub fn transaction_type(mut self, transaction_type: TransactionType) -> Self {
        self.transaction_type.push(transaction_type);
        self
    }

    pub(crate) fn set_params(&self, url: &mut Url) {
        if let Some(from) = self.from {
            url.query_pairs_mut().append_pair("from", &from.to_string());
        }
        if let Some(to) = self.to {
            url.query_pairs_mut().append_pair("to", &to.to_string());
        }
        if let Some(page_size) = self.page_size {
            url.query_pairs_mut()
                .append_pair("pageSize", page_size.to_string().as_str());
        }
        if !self.transaction_type.is_empty() {
            url.query_pairs_mut().append_pair(
                "type",
                self.transaction_type
                    .iter()
                    .map(|t| t.to_string())
                    .collect::<Vec<String>>()
                    .join(",")
                    .as_str(),
            );
        }
    }
}

/// Response body for `GET /v3/accounts/{accountID}/transactions` (HTTP 200).
///
/// The actual transactions are not embedded here; instead, `pages` contains
/// URLs for fetching each page of transactions individually.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListTransactionsResponse {
    /// Total number of transactions matching the query.
    pub count: i64,
    /// Start of the time range covered by this response.
    pub from: DateTime<Utc>,
    /// End of the time range covered by this response.
    pub to: DateTime<Utc>,
    /// Maximum number of transactions per page used for this response.
    pub page_size: i64,
    /// URLs for each page of results. Fetch each URL to retrieve the transactions.
    pub pages: Vec<String>,
    /// ID of the most recent transaction on the account.
    #[serde(rename = "lastTransactionID")]
    pub last_transaction_id: TransactionID,
}

/// Response body for `GET /v3/accounts/{accountID}/transactions/{transactionID}` (HTTP 200).
#[derive(Debug, Serialize, Deserialize)]
pub struct GetTransactionDetailsResponse {
    /// The requested transaction.
    pub transaction: Transaction,
    /// ID of the most recent transaction on the account.
    #[serde(rename = "lastTransactionID")]
    pub last_transaction_id: TransactionID,
}

/// Request parameters for
/// `GET /v3/accounts/{accountID}/transactions/idrange`.
///
/// Returns transactions with IDs in the inclusive range `[from, to]`.
pub struct GetTransactionsByIDRangeRequest {
    from: TransactionID,
    to: TransactionID,
    filter: Vec<TransactionFilter>,
}

impl GetTransactionsByIDRangeRequest {
    /// Creates a new request for the inclusive ID range `[from, to]`.
    pub fn new(from: TransactionID, to: TransactionID) -> Self {
        GetTransactionsByIDRangeRequest {
            from,
            to,
            filter: Vec::new(),
        }
    }

    /// Restricts the results to transactions matching `filter`.
    /// Call multiple times to include several filter categories.
    pub fn filter(mut self, filter: TransactionFilter) -> Self {
        self.filter.push(filter);
        self
    }

    pub(crate) fn set_params(&self, url: &mut Url) {
        url.query_pairs_mut()
            .append_pair("from", &self.from.to_string())
            .append_pair("to", &self.to.to_string());
        if !self.filter.is_empty() {
            url.query_pairs_mut().append_pair(
                "type",
                self.filter
                    .iter()
                    .map(|f| f.to_string())
                    .collect::<Vec<String>>()
                    .join(",")
                    .as_str(),
            );
        }
    }
}

/// Response body for `GET /v3/accounts/{accountID}/transactions/idrange` and
/// `GET /v3/accounts/{accountID}/transactions/sinceid` (HTTP 200).
#[derive(Debug, Serialize, Deserialize)]
pub struct GetTransactionsResponse {
    /// The transactions matching the request.
    transactions: Vec<Transaction>,
    /// ID of the most recent transaction on the account.
    #[serde(rename = "lastTransactionID")]
    pub last_transaction_id: TransactionID,
}

/// Request parameters for
/// `GET /v3/accounts/{accountID}/transactions/sinceid`.
///
/// Returns all transactions with IDs greater than `id`.
pub struct GetTransactionsBySinceIDRequest {
    id: TransactionID,
    filter: Vec<TransactionFilter>,
}

impl GetTransactionsBySinceIDRequest {
    /// Creates a new request that returns transactions since (exclusive of) `id`.
    pub fn new(id: TransactionID) -> Self {
        GetTransactionsBySinceIDRequest {
            id,
            filter: Vec::new(),
        }
    }

    /// Restricts the results to transactions matching `filter`.
    /// Call multiple times to include several filter categories.
    pub fn filter(mut self, filter: TransactionFilter) -> Self {
        self.filter.push(filter);
        self
    }

    pub(crate) fn set_params(&self, url: &mut Url) {
        url.query_pairs_mut()
            .append_pair("id", &self.id.to_string());
        if !self.filter.is_empty() {
            url.query_pairs_mut().append_pair(
                "type",
                &self
                    .filter
                    .iter()
                    .map(|f| f.to_string())
                    .collect::<Vec<String>>()
                    .join(","),
            );
        }
    }
}

/// An item from the OANDA transaction stream.
///
/// The stream emits either a [`TransactionHeartbeat`] (when idle) or a full
/// [`Transaction`] (for every account event). Deserialised via the `"type"` tag.
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum TransactionStreamItem {
    HEARTBEAT(TransactionHeartbeat),
    /// Boxed because `Transaction` is much larger than a heartbeat.
    #[serde(untagged)]
    Transaction(Box<Transaction>),
}

/// Provides access to the OANDA Transaction endpoints
/// (`/v3/accounts/{id}/transactions/...`).
///
/// Obtain an instance via [`Client::transaction`](crate::client::Client::transaction).
pub struct TransactionService<'a> {
    client: &'a Client,
}

impl<'a> TransactionService<'a> {
    pub(crate) fn new(client: &'a Client) -> Self {
        TransactionService { client }
    }

    /// Lists transactions on the account, optionally filtered by time range or type.
    ///
    /// Calls `GET /v3/accounts/{accountID}/transactions`.
    ///
    /// Returns page URLs rather than inline transactions; follow each URL in
    /// [`ListTransactionsResponse::pages`] to retrieve the actual transaction data.
    ///
    /// # Panics
    ///
    /// Panics if no `account_id` has been set on the client.
    pub async fn list(
        &self,
        req: ListTransactionsRequest,
    ) -> Result<ListTransactionsResponse, APIError> {
        let mut url = self.client.account_url("transactions");
        req.set_params(&mut url);
        let http_req = Request::new(reqwest::Method::GET, url);
        let http_resp = self.client.http_client.execute(http_req).await?;
        handle_response!(
            http_resp,
            success: StatusCode::OK => ListTransactionsResponse,
            errors: [ ]
        )
    }

    /// Returns the details of the transaction identified by `id`.
    ///
    /// Calls `GET /v3/accounts/{accountID}/transactions/{transactionID}`.
    ///
    /// # Panics
    ///
    /// Panics if no `account_id` has been set on the client.
    pub async fn get_details(
        &self,
        id: TransactionID,
    ) -> Result<GetTransactionDetailsResponse, APIError> {
        let url = self.client.account_url(&format!("transactions/{}", id));
        let http_req = Request::new(reqwest::Method::GET, url);
        let http_resp = self.client.http_client.execute(http_req).await?;
        handle_response!(
            http_resp,
            success: StatusCode::OK => GetTransactionDetailsResponse,
            errors: [ ]
        )
    }

    /// Returns all transactions with IDs in the inclusive range specified by `req`.
    ///
    /// Calls `GET /v3/accounts/{accountID}/transactions/idrange`.
    ///
    /// # Panics
    ///
    /// Panics if no `account_id` has been set on the client.
    pub async fn get_by_id_range(
        &self,
        req: GetTransactionsByIDRangeRequest,
    ) -> Result<GetTransactionsResponse, APIError> {
        let mut url = self.client.account_url("transactions/idrange");
        req.set_params(&mut url);
        let http_req = Request::new(reqwest::Method::GET, url);
        let http_resp = self.client.http_client.execute(http_req).await?;
        handle_response!(
            http_resp,
            success: StatusCode::OK => GetTransactionsResponse,
            errors: [ ]
        )
    }

    /// Returns all transactions with IDs greater than the one specified in `req`.
    ///
    /// Calls `GET /v3/accounts/{accountID}/transactions/sinceid`.
    ///
    /// # Panics
    ///
    /// Panics if no `account_id` has been set on the client.
    pub async fn get_by_since_id(
        &self,
        req: GetTransactionsBySinceIDRequest,
    ) -> Result<GetTransactionsResponse, APIError> {
        let mut url = self.client.account_url("transactions/sinceid");
        req.set_params(&mut url);
        let http_req = Request::new(reqwest::Method::GET, url);
        let http_resp = self.client.http_client.execute(http_req).await?;
        handle_response!(
            http_resp,
            success: StatusCode::OK => GetTransactionsResponse,
            errors: [ ]
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::client::setup_test_client;

    #[test]
    fn regression_transaction_filter_query_parameters() {
        use std::collections::BTreeMap;
        let mut url = Url::parse("https://example.com/transactions").unwrap();
        ListTransactionsRequest::new()
            .transaction_type(TransactionType::OrderFill)
            .transaction_type(TransactionType::MarketOrder)
            .set_params(&mut url);
        let params: BTreeMap<_, _> = url.query_pairs().into_owned().collect();
        assert_eq!(params.len(), 1);
        assert_eq!(params["type"], "ORDER_FILL,MARKET_ORDER");

        let mut url = Url::parse("https://example.com/transactions/idrange").unwrap();
        GetTransactionsByIDRangeRequest::new("1".into(), "5".into())
            .filter(TransactionFilter::OrderFill)
            .filter(TransactionFilter::Funding)
            .set_params(&mut url);
        let params: BTreeMap<_, _> = url.query_pairs().into_owned().collect();
        assert_eq!(params.len(), 3);
        assert_eq!(params["from"], "1");
        assert_eq!(params["to"], "5");
        assert_eq!(params["type"], "ORDER_FILL,FUNDING");

        let mut url = Url::parse("https://example.com/transactions/sinceid").unwrap();
        GetTransactionsBySinceIDRequest::new("5".into())
            .filter(TransactionFilter::OrderFill)
            .filter(TransactionFilter::Funding)
            .set_params(&mut url);
        let params: BTreeMap<_, _> = url.query_pairs().into_owned().collect();
        assert_eq!(params.len(), 2);
        assert_eq!(params["id"], "5");
        assert_eq!(params["type"], "ORDER_FILL,FUNDING");
    }

    #[tokio::test]
    async fn test_list_transactions() {
        let client = setup_test_client();
        let req = ListTransactionsRequest::new();
        let resp = client.transaction().list(req).await.unwrap();
        println!("{:#?}", resp);
    }

    #[tokio::test]
    async fn test_get_transaction_details() {
        let client = setup_test_client();
        let list = client
            .transaction()
            .list(ListTransactionsRequest::new())
            .await
            .unwrap();
        let resp = client
            .transaction()
            .get_details(list.last_transaction_id)
            .await
            .unwrap();
        println!("{:#?}", resp);
    }

    #[tokio::test]
    async fn test_get_transactions_by_id_range() {
        let client = setup_test_client();
        let resp = client
            .transaction()
            .list(ListTransactionsRequest::new())
            .await
            .unwrap();
        let req = GetTransactionsByIDRangeRequest::new(
            resp.last_transaction_id.clone(),
            resp.last_transaction_id.clone(),
        );
        let resp = client.transaction().get_by_id_range(req).await.unwrap();
        println!("{:#?}", resp);
    }

    #[tokio::test]
    async fn test_get_transactions_by_since_id() {
        let client = setup_test_client();
        let resp = client
            .transaction()
            .list(ListTransactionsRequest::new())
            .await
            .unwrap();
        let req = GetTransactionsBySinceIDRequest::new(resp.last_transaction_id.to_string());
        let resp = client.transaction().get_by_since_id(req).await.unwrap();
        println!("{:#?}", resp);
    }
}
