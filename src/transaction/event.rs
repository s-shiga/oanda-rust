use super::*;

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

/// An item from the OANDA transaction stream.
///
/// The stream emits either a [`TransactionHeartbeat`] (when idle) or a full
/// [`Transaction`] (for every account event). Deserialised via the `"type"` tag.
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum TransactionStreamItem {
    /// A periodic keepalive sent while the account is idle.
    #[serde(rename = "HEARTBEAT")]
    Heartbeat(TransactionHeartbeat),
    /// Boxed because `Transaction` is much larger than a heartbeat.
    #[serde(untagged)]
    Transaction(Box<Transaction>),
}
