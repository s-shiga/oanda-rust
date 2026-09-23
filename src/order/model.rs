use super::*;

// ---------------------------------------------------------------------------
// Order enum (tagged union)
// ---------------------------------------------------------------------------

/// A polymorphic representation of any order type returned by the OANDA API.
///
/// Deserialize from JSON using the `"type"` field as a tag, so each variant
/// maps to its corresponding order struct.
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Order {
    /// An order to buy or sell at the current market price.
    #[serde(rename = "MARKET")]
    MarketOrder(MarketOrder),
    /// An order to buy or sell at a specified price or better.
    #[serde(rename = "LIMIT")]
    LimitOrder(LimitOrder),
    /// An order to buy or sell once price crosses a specified threshold,
    /// executed at market price with an optional worst-price bound.
    #[serde(rename = "STOP")]
    StopOrder(StopOrder),
    /// An order that becomes a market order once a specified price is touched.
    #[serde(rename = "MARKET_IF_TOUCHED")]
    MarketIfTouchedOrder(MarketIfTouchedOrder),
    /// A trade-attached order that closes the position at a profit target price.
    #[serde(rename = "TAKE_PROFIT")]
    TakeProfitOrder(TakeProfitOrder),
    /// A trade-attached order that closes the position at a loss-limit price.
    #[serde(rename = "STOP_LOSS")]
    StopLossOrder(StopLossOrder),
    /// A trade-attached stop-loss order with guaranteed execution at the stop price.
    #[serde(rename = "GUARANTEED_STOP_LOSS")]
    GuaranteedStopLossOrder(GuaranteedStopLossOrder),
    /// A trade-attached stop that trails the market price by a fixed distance.
    #[serde(rename = "TRAILING_STOP_LOSS")]
    TrailingStopLossOrder(TrailingStopLossOrder),
    /// An internal order used by OANDA to fill a trade at a fixed price
    /// (e.g. during account transfers or corrections).
    #[serde(rename = "FIXED_PRICE")]
    FixedPriceOrder(FixedPriceOrder),
}

// ---------------------------------------------------------------------------
// Order Structs
// ---------------------------------------------------------------------------

/// A market order as returned by the OANDA API.
///
/// Market orders are executed immediately at the best available price.
/// The `time_in_force` is always `FOK` (fill-or-kill) or `IOC`
/// (immediate-or-cancel).
#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MarketOrder {
    /// The order's unique identifier.
    pub id: OrderID,
    /// Timestamp at which the order was created.
    pub create_time: DateTime<Utc>,
    /// Current lifecycle state of the order.
    pub state: OrderState,
    /// Optional client-supplied metadata attached to the order.
    pub client_extensions: Option<ClientExtensions>,
    /// The instrument to be traded.
    pub instrument: InstrumentName,
    /// Number of units to trade. Positive = buy (long), negative = sell (short).
    pub units: DecimalNumber,
    /// How long the order remains active (`FOK` or `IOC` for market orders).
    pub time_in_force: TimeInForce,
    /// The worst fill price acceptable. If the order cannot be filled within
    /// this bound, it is cancelled.
    pub price_bound: Option<PriceValue>,
    /// How the order interacts with an existing position on the instrument.
    pub position_fill: OrderPositionFill,
    /// Take-profit order to attach to any trade opened by this order.
    pub take_profit_on_fill: Option<TakeProfitDetails>,
    /// Stop-loss order to attach to any trade opened by this order.
    pub stop_loss_on_fill: Option<StopLossDetails>,
    /// Trailing stop-loss to attach to any trade opened by this order.
    pub trailing_stop_loss_on_fill: Option<TrailingStopLossDetails>,
    /// Guaranteed stop-loss to attach to any trade opened by this order.
    pub guaranteed_stop_loss_on_fill: Option<GuaranteedStopLossDetails>,
    /// Client extensions to apply to the trade opened by this order.
    pub trade_client_extensions: Option<ClientExtensions>,
    /// ID of the transaction that filled this order, if it has been filled.
    #[serde(rename = "fillingTransactionID")]
    pub filling_transaction_id: Option<TransactionID>,
    /// Timestamp at which the order was filled.
    pub filled_time: Option<DateTime<Utc>>,
    /// ID of the trade opened by this order, if any.
    #[serde(rename = "tradeOpenedID")]
    pub trade_opened_id: Option<TradeID>,
    /// ID of the trade reduced (partially closed) by this order, if any.
    #[serde(rename = "tradeReducedID")]
    pub trade_reduced_id: Option<TradeID>,
    /// IDs of trades fully closed by this order, if any.
    #[serde(rename = "tradeClosedIDs")]
    pub trade_closed_ids: Option<Vec<TradeID>>,
    /// ID of the transaction that cancelled this order, if it was cancelled.
    #[serde(rename = "cancellingTransactionID")]
    pub cancelling_transaction_id: Option<TransactionID>,
    /// Timestamp at which the order was cancelled.
    pub cancelled_time: Option<DateTime<Utc>>,
    /// Details of the trade close this market order was created to perform.
    pub trade_close: Option<MarketOrderTradeClose>,
    /// Details of the long position closeout this order was created to perform.
    pub long_position_closeout: Option<MarketOrderPositionCloseout>,
    /// Details of the short position closeout this order was created to perform.
    pub short_position_closeout: Option<MarketOrderPositionCloseout>,
    /// Details when this order was created as part of a margin closeout.
    pub margin_closeout: Option<MarketOrderMarginCloseout>,
    /// Details when this order was created to close a trade that could not be
    /// closed at the time it was reduced.
    pub delayed_trade_close: Option<MarketOrderDelayedTradeClose>,
}

/// A fixed-price order as returned by the OANDA API.
///
/// An internal order type created by OANDA to fill a trade at a specific fixed
/// price, for example during account corrections or transfers. Cannot be created
/// by clients directly.
#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FixedPriceOrder {
    /// The order's unique identifier.
    pub id: OrderID,
    /// Timestamp at which the order was created.
    pub create_time: DateTime<Utc>,
    /// Current lifecycle state of the order.
    pub state: OrderState,
    /// Optional client-supplied metadata attached to the order.
    pub client_extensions: Option<ClientExtensions>,
    /// The instrument to be traded.
    pub instrument: InstrumentName,
    /// Number of units to trade. Positive = buy (long), negative = sell (short).
    pub units: DecimalNumber,
    /// The fixed price at which the order will be filled.
    pub price: PriceValue,
    /// How the order interacts with an existing position on the instrument.
    pub position_fill: OrderPositionFill,
    /// The state of the trade the order is intended to result in.
    pub trade_state: String,
    /// Take-profit order to attach to any trade opened by this order.
    pub take_profit_on_fill: Option<TakeProfitDetails>,
    /// Stop-loss order to attach to any trade opened by this order.
    pub stop_loss_on_fill: Option<StopLossDetails>,
    /// Trailing stop-loss to attach to any trade opened by this order.
    pub trailing_stop_loss_on_fill: Option<TrailingStopLossDetails>,
    /// Guaranteed stop-loss to attach to any trade opened by this order.
    pub guaranteed_stop_loss_on_fill: Option<GuaranteedStopLossDetails>,
    /// Client extensions to apply to the trade opened by this order.
    pub trade_client_extensions: Option<ClientExtensions>,
    /// ID of the transaction that filled this order, if filled.
    #[serde(rename = "fillingTransactionID")]
    pub filling_transaction_id: Option<TransactionID>,
    /// Timestamp at which the order was filled.
    pub filled_time: Option<DateTime<Utc>>,
    /// ID of any trade opened as a result of this order being filled.
    #[serde(rename = "tradeOpenedID")]
    pub trade_opened_id: Option<TradeID>,
    /// ID of any trade reduced by this order.
    #[serde(rename = "tradeReducedID")]
    pub trade_reduced_id: Option<TradeID>,
    /// IDs of trades fully closed by this order.
    #[serde(rename = "tradeClosedIDs")]
    pub trade_closed_ids: Option<Vec<TradeID>>,
    /// ID of the transaction that cancelled this order, if cancelled.
    #[serde(rename = "cancellingTransactionID")]
    pub cancelling_transaction_id: Option<TransactionID>,
    /// Timestamp at which the order was cancelled.
    pub cancelled_time: Option<DateTime<Utc>>,
}

/// A limit order as returned by the OANDA API.
///
/// Executes at `price` or better once the market reaches that level.
/// Supports `GTC`, `GTD`, and `GFD` time-in-force values.
#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LimitOrder {
    /// The order's unique identifier.
    pub id: OrderID,
    /// Timestamp at which the order was created.
    pub create_time: DateTime<Utc>,
    /// Current lifecycle state of the order.
    pub state: OrderState,
    /// Optional client-supplied metadata attached to the order.
    pub client_extensions: Option<ClientExtensions>,
    /// The instrument to be traded.
    pub instrument: InstrumentName,
    /// Number of units to trade. Positive = buy (long), negative = sell (short).
    pub units: DecimalNumber,
    /// The limit price at which the order will execute.
    pub price: PriceValue,
    /// How long the order remains active.
    pub time_in_force: TimeInForce,
    /// Expiry timestamp when `time_in_force` is `GTD`.
    pub gtd_time: Option<DateTime<Utc>>,
    /// How the order interacts with an existing position on the instrument.
    pub position_fill: OrderPositionFill,
    /// Which price stream triggers the order.
    pub trigger_condition: OrderTriggerCondition,
    /// Take-profit order to attach to any trade opened by this order.
    pub take_profit_on_fill: Option<TakeProfitDetails>,
    /// Stop-loss order to attach to any trade opened by this order.
    pub stop_loss_on_fill: Option<StopLossDetails>,
    /// Trailing stop-loss to attach to any trade opened by this order.
    pub trailing_stop_loss_on_fill: Option<TrailingStopLossDetails>,
    /// Guaranteed stop-loss to attach to any trade opened by this order.
    pub guaranteed_stop_loss_on_fill: Option<GuaranteedStopLossDetails>,
    /// Client extensions to apply to the trade opened by this order.
    pub trade_client_extensions: Option<ClientExtensions>,
    /// ID of the transaction that filled this order, if filled.
    #[serde(rename = "fillingTransactionID")]
    pub filling_transaction_id: Option<TransactionID>,
    /// Timestamp at which the order was filled.
    pub filled_time: Option<DateTime<Utc>>,
    /// ID of the trade opened by this order, if any.
    #[serde(rename = "tradeOpenedID")]
    pub trade_opened_id: Option<TradeID>,
    /// ID of the trade reduced by this order, if any.
    #[serde(rename = "tradeReducedID")]
    pub trade_reduced_id: Option<TradeID>,
    /// IDs of trades fully closed by this order, if any.
    #[serde(rename = "tradeClosedIDs")]
    pub trade_closed_ids: Option<Vec<TradeID>>,
    /// ID of the transaction that cancelled this order, if cancelled.
    #[serde(rename = "cancellingTransactionID")]
    pub cancelling_transaction_id: Option<TransactionID>,
    /// Timestamp at which the order was cancelled.
    pub cancelled_time: Option<DateTime<Utc>>,
    /// The ID of the order this order replaced, if applicable.
    #[serde(rename = "replacesOrderID")]
    pub replaces_order_id: Option<OrderID>,
    /// The ID of the order that replaced this order, if applicable.
    #[serde(rename = "replacedByOrderID")]
    pub replaced_by_order_id: Option<OrderID>,
}

/// A stop order as returned by the OANDA API.
///
/// Becomes a market order once the stop `price` is reached, with an optional
/// `price_bound` to limit the worst acceptable fill.
#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StopOrder {
    /// The order's unique identifier.
    pub id: OrderID,
    /// Timestamp at which the order was created.
    pub create_time: DateTime<Utc>,
    /// Current lifecycle state of the order.
    pub state: OrderState,
    /// Optional client-supplied metadata attached to the order.
    pub client_extensions: Option<ClientExtensions>,
    /// The instrument to be traded.
    pub instrument: InstrumentName,
    /// Number of units to trade. Positive = buy (long), negative = sell (short).
    pub units: DecimalNumber,
    /// The stop trigger price.
    pub price: PriceValue,
    /// The worst fill price acceptable after the stop triggers.
    pub price_bound: Option<PriceValue>,
    /// How long the order remains active.
    pub time_in_force: TimeInForce,
    /// Expiry timestamp when `time_in_force` is `GTD`.
    pub gtd_time: Option<DateTime<Utc>>,
    /// How the order interacts with an existing position on the instrument.
    pub position_fill: OrderPositionFill,
    /// Which price stream triggers the order.
    pub trigger_condition: OrderTriggerCondition,
    /// Take-profit order to attach to any trade opened by this order.
    pub take_profit_on_fill: Option<TakeProfitDetails>,
    /// Stop-loss order to attach to any trade opened by this order.
    pub stop_loss_on_fill: Option<StopLossDetails>,
    /// Trailing stop-loss to attach to any trade opened by this order.
    pub trailing_stop_loss_on_fill: Option<TrailingStopLossDetails>,
    /// Guaranteed stop-loss to attach to any trade opened by this order.
    pub guaranteed_stop_loss_on_fill: Option<GuaranteedStopLossDetails>,
    /// Client extensions to apply to the trade opened by this order.
    pub trade_client_extensions: Option<ClientExtensions>,
    /// ID of the transaction that filled this order, if filled.
    #[serde(rename = "fillingTransactionID")]
    pub filling_transaction_id: Option<TransactionID>,
    /// Timestamp at which the order was filled.
    pub filled_time: Option<DateTime<Utc>>,
    /// ID of the trade opened by this order, if any.
    #[serde(rename = "tradeOpenedID")]
    pub trade_opened_id: Option<TradeID>,
    /// ID of the trade reduced by this order, if any.
    #[serde(rename = "tradeReducedID")]
    pub trade_reduced_id: Option<TradeID>,
    /// IDs of trades fully closed by this order, if any.
    #[serde(rename = "tradeClosedIDs")]
    pub trade_closed_ids: Option<Vec<TradeID>>,
    /// ID of the transaction that cancelled this order, if cancelled.
    #[serde(rename = "cancellingTransactionID")]
    pub cancelling_transaction_id: Option<TransactionID>,
    /// Timestamp at which the order was cancelled.
    pub cancelled_time: Option<DateTime<Utc>>,
    /// The ID of the order this order replaced, if applicable.
    #[serde(rename = "replacesOrderID")]
    pub replaces_order_id: Option<OrderID>,
    /// The ID of the order that replaced this order, if applicable.
    #[serde(rename = "replacedByOrderID")]
    pub replaced_by_order_id: Option<OrderID>,
}

/// A market-if-touched (MIT) order as returned by the OANDA API.
///
/// Sits as a pending order until the market price touches `price`, at which
/// point it becomes a market order.
#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MarketIfTouchedOrder {
    /// The order's unique identifier.
    pub id: OrderID,
    /// Timestamp at which the order was created.
    pub create_time: DateTime<Utc>,
    /// Current lifecycle state of the order.
    pub state: OrderState,
    /// Optional client-supplied metadata attached to the order.
    pub client_extensions: Option<ClientExtensions>,
    /// The instrument to be traded.
    pub instrument: InstrumentName,
    /// Number of units to trade. Positive = buy (long), negative = sell (short).
    pub units: DecimalNumber,
    /// The trigger price. When the market touches this level the order converts
    /// to a market order.
    pub price: PriceValue,
    /// Worst acceptable fill price after the order triggers.
    pub price_bound: Option<PriceValue>,
    /// How long the order remains active.
    pub time_in_force: TimeInForce,
    /// Expiry timestamp when `time_in_force` is `GTD`.
    pub gtd_time: Option<DateTime<Utc>>,
    /// How the order interacts with an existing position on the instrument.
    pub position_fill: OrderPositionFill,
    /// Which price stream triggers the order.
    pub trigger_condition: OrderTriggerCondition,
    /// The market price at the time the order was created.
    pub initial_market_price: PriceValue,
    /// Take-profit order to attach to any trade opened by this order.
    pub take_profit_on_fill: Option<TakeProfitDetails>,
    /// Stop-loss order to attach to any trade opened by this order.
    pub stop_loss_on_fill: Option<StopLossDetails>,
    /// Trailing stop-loss to attach to any trade opened by this order.
    pub trailing_stop_loss_on_fill: Option<TrailingStopLossDetails>,
    /// Guaranteed stop-loss to attach to any trade opened by this order.
    pub guaranteed_stop_loss_on_fill: Option<GuaranteedStopLossDetails>,
    /// Client extensions to apply to the trade opened by this order.
    pub trade_client_extensions: Option<ClientExtensions>,
    /// ID of the transaction that filled this order, if filled.
    #[serde(rename = "fillingTransactionID")]
    pub filling_transaction_id: Option<TransactionID>,
    /// Timestamp at which the order was filled.
    pub filled_time: Option<DateTime<Utc>>,
    /// ID of the trade opened by this order, if any.
    #[serde(rename = "tradeOpenedID")]
    pub trade_opened_id: Option<TradeID>,
    /// ID of the trade reduced by this order, if any.
    #[serde(rename = "tradeReducedID")]
    pub trade_reduced_id: Option<TradeID>,
    /// IDs of trades fully closed by this order, if any.
    #[serde(rename = "tradeClosedIDs")]
    pub trade_closed_ids: Option<Vec<TradeID>>,
    /// ID of the transaction that cancelled this order, if cancelled.
    #[serde(rename = "cancellingTransactionID")]
    pub cancelling_transaction_id: Option<TransactionID>,
    /// Timestamp at which the order was cancelled.
    pub cancelled_time: Option<DateTime<Utc>>,
    /// The ID of the order this order replaced, if applicable.
    #[serde(rename = "replacesOrderID")]
    pub replaces_order_id: Option<OrderID>,
    /// The ID of the order that replaced this order, if applicable.
    #[serde(rename = "replacedByOrderID")]
    pub replaced_by_order_id: Option<OrderID>,
}

/// A take-profit order as returned by the OANDA API.
///
/// A trade-attached order that closes the linked trade when the market reaches
/// the target `price`, locking in profit.
#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TakeProfitOrder {
    /// The order's unique identifier.
    pub id: OrderID,
    /// Timestamp at which the order was created.
    pub create_time: DateTime<Utc>,
    /// Current lifecycle state of the order.
    pub state: OrderState,
    /// Optional client-supplied metadata attached to the order.
    pub client_extensions: Option<ClientExtensions>,
    /// The ID of the trade this order is attached to.
    #[serde(rename = "tradeID")]
    pub trade_id: TradeID,
    /// The client-provided trade ID, if any.
    #[serde(rename = "clientTradeID")]
    pub client_trade_id: Option<ClientID>,
    /// The price at which the take-profit will trigger and close the trade.
    pub price: PriceValue,
    /// How long the order remains active.
    pub time_in_force: TimeInForce,
    /// Expiry timestamp when `time_in_force` is `GTD`.
    pub gtd_time: Option<DateTime<Utc>>,
    /// Which price stream triggers the order.
    pub trigger_condition: OrderTriggerCondition,
    /// ID of the transaction that filled this order, if filled.
    #[serde(rename = "fillingTransactionID")]
    pub filling_transaction_id: Option<TransactionID>,
    /// Timestamp at which the order was filled.
    pub filled_time: Option<DateTime<Utc>>,
    /// ID of any trade opened as a result of this order being filled.
    #[serde(rename = "tradeOpenedID")]
    pub trade_opened_id: Option<TradeID>,
    /// ID of any trade reduced by this order.
    #[serde(rename = "tradeReducedID")]
    pub trade_reduced_id: Option<TradeID>,
    /// IDs of trades fully closed by this order.
    #[serde(rename = "tradeClosedIDs")]
    pub trade_closed_ids: Option<Vec<TradeID>>,
    /// ID of the transaction that cancelled this order, if cancelled.
    #[serde(rename = "cancellingTransactionID")]
    pub cancelling_transaction_id: Option<TransactionID>,
    /// Timestamp at which the order was cancelled.
    pub cancelled_time: Option<DateTime<Utc>>,
    /// The ID of the order this order replaced, if applicable.
    #[serde(rename = "replacesOrderID")]
    pub replaces_order_id: Option<OrderID>,
    /// The ID of the order that replaced this order, if applicable.
    #[serde(rename = "replacedByOrderID")]
    pub replaced_by_order_id: Option<OrderID>,
}

/// A stop-loss order as returned by the OANDA API.
///
/// A trade-attached order that closes the linked trade to limit losses when
/// price moves adversely. May be specified by absolute `price` or by `distance`
/// from the current price. Can optionally be `guaranteed`.
#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StopLossOrder {
    /// The order's unique identifier.
    pub id: OrderID,
    /// Timestamp at which the order was created.
    pub create_time: DateTime<Utc>,
    /// Current lifecycle state of the order.
    pub state: OrderState,
    /// Optional client-supplied metadata attached to the order.
    pub client_extensions: Option<ClientExtensions>,
    /// The ID of the trade this order is attached to.
    #[serde(rename = "tradeID")]
    pub trade_id: TradeID,
    /// The client-provided trade ID, if any.
    #[serde(rename = "clientTradeID")]
    pub client_trade_id: Option<ClientID>,
    /// Absolute stop price. Mutually exclusive with `distance`.
    pub price: PriceValue,
    /// Distance from current price at which the stop is placed.
    /// Mutually exclusive with `price`.
    pub distance: Option<DecimalNumber>,
    /// How long the order remains active.
    pub time_in_force: TimeInForce,
    /// Expiry timestamp when `time_in_force` is `GTD`.
    pub gtd_time: Option<DateTime<Utc>>,
    /// Which price stream triggers the order.
    pub trigger_condition: OrderTriggerCondition,
    /// ID of the transaction that filled this order, if filled.
    #[serde(rename = "fillingTransactionID")]
    pub filling_transaction_id: Option<TransactionID>,
    /// Timestamp at which the order was filled.
    pub filled_time: Option<DateTime<Utc>>,
    /// ID of any trade opened as a result of this order being filled.
    #[serde(rename = "tradeOpenedID")]
    pub trade_opened_id: Option<TradeID>,
    /// ID of any trade reduced by this order.
    #[serde(rename = "tradeReducedID")]
    pub trade_reduced_id: Option<TradeID>,
    /// IDs of trades fully closed by this order.
    #[serde(rename = "tradeClosedIDs")]
    pub trade_closed_ids: Option<Vec<TradeID>>,
    /// ID of the transaction that cancelled this order, if cancelled.
    #[serde(rename = "cancellingTransactionID")]
    pub cancelling_transaction_id: Option<TransactionID>,
    /// Timestamp at which the order was cancelled.
    pub cancelled_time: Option<DateTime<Utc>>,
    /// The ID of the order this order replaced, if applicable.
    #[serde(rename = "replacesOrderID")]
    pub replaces_order_id: Option<OrderID>,
    /// The ID of the order that replaced this order, if applicable.
    #[serde(rename = "replacedByOrderID")]
    pub replaced_by_order_id: Option<OrderID>,
}

/// A guaranteed stop-loss order as returned by the OANDA API.
///
/// Like a stop-loss order but guaranteed to execute at exactly the stop `price`
/// regardless of gapping or slippage. An execution premium is charged.
#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GuaranteedStopLossOrder {
    /// The order's unique identifier.
    pub id: OrderID,
    /// Timestamp at which the order was created.
    pub create_time: DateTime<Utc>,
    /// Current lifecycle state of the order.
    pub state: OrderState,
    /// Optional client-supplied metadata attached to the order.
    pub client_extensions: Option<ClientExtensions>,
    /// The ID of the trade this order is attached to.
    #[serde(rename = "tradeID")]
    pub trade_id: TradeID,
    /// The client-provided trade ID, if any.
    #[serde(rename = "clientTradeID")]
    pub client_trade_id: Option<ClientID>,
    /// The guaranteed stop price at which the trade will be closed.
    pub price: PriceValue,
    /// Distance from current price used when creating the order; converted to
    /// an absolute `price` by the server.
    pub distance: Option<DecimalNumber>,
    /// How long the order remains active.
    pub time_in_force: TimeInForce,
    /// Expiry timestamp when `time_in_force` is `GTD`.
    pub gtd_time: Option<DateTime<Utc>>,
    /// Which price stream triggers the order.
    pub trigger_condition: OrderTriggerCondition,
    /// The premium paid for guaranteed execution.
    pub guaranteed_execution_premium: DecimalNumber,
    /// ID of the transaction that filled this order, if filled.
    #[serde(rename = "fillingTransactionID")]
    pub filling_transaction_id: Option<TransactionID>,
    /// Timestamp at which the order was filled.
    pub filled_time: Option<DateTime<Utc>>,
    /// ID of any trade opened as a result of this order being filled.
    #[serde(rename = "tradeOpenedID")]
    pub trade_opened_id: Option<TradeID>,
    /// ID of any trade reduced by this order.
    #[serde(rename = "tradeReducedID")]
    pub trade_reduced_id: Option<TradeID>,
    /// IDs of trades fully closed by this order.
    #[serde(rename = "tradeClosedIDs")]
    pub trade_closed_ids: Option<Vec<TradeID>>,
    /// ID of the transaction that cancelled this order, if cancelled.
    #[serde(rename = "cancellingTransactionID")]
    pub cancelling_transaction_id: Option<TransactionID>,
    /// Timestamp at which the order was cancelled.
    pub cancelled_time: Option<DateTime<Utc>>,
    /// The ID of the order this order replaced, if applicable.
    #[serde(rename = "replacesOrderID")]
    pub replaces_order_id: Option<OrderID>,
    /// The ID of the order that replaced this order, if applicable.
    #[serde(rename = "replacedByOrderID")]
    pub replaced_by_order_id: Option<OrderID>,
}

/// A trailing stop-loss order as returned by the OANDA API.
///
/// A trade-attached order whose stop price follows the market at a fixed
/// `distance` below (for long trades) or above (for short trades) the current price.
#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TrailingStopLossOrder {
    /// The order's unique identifier.
    pub id: OrderID,
    /// Timestamp at which the order was created.
    pub create_time: DateTime<Utc>,
    /// Current lifecycle state of the order.
    pub state: OrderState,
    /// Optional client-supplied metadata attached to the order.
    pub client_extensions: Option<ClientExtensions>,
    /// The ID of the trade this order is attached to.
    #[serde(rename = "tradeID")]
    pub trade_id: TradeID,
    /// The client-provided trade ID, if any.
    #[serde(rename = "clientTradeID")]
    pub client_trade_id: Option<ClientID>,
    /// The trailing distance (in price units) kept between the market price
    /// and the stop.
    pub distance: DecimalNumber,
    /// How long the order remains active.
    pub time_in_force: TimeInForce,
    /// Expiry timestamp when `time_in_force` is `GTD`.
    pub gtd_time: Option<DateTime<Utc>>,
    /// Which price stream triggers the order.
    pub trigger_condition: OrderTriggerCondition,
    /// The current calculated absolute stop price, derived from `distance`
    /// and the current market price.
    pub trailing_stop_value: PriceValue,
    /// ID of the transaction that filled this order, if filled.
    #[serde(rename = "fillingTransactionID")]
    pub filling_transaction_id: Option<TransactionID>,
    /// Timestamp at which the order was filled.
    pub filled_time: Option<DateTime<Utc>>,
    /// ID of any trade opened as a result of this order being filled.
    #[serde(rename = "tradeOpenedID")]
    pub trade_opened_id: Option<TradeID>,
    /// ID of any trade reduced by this order.
    #[serde(rename = "tradeReducedID")]
    pub trade_reduced_id: Option<TradeID>,
    /// IDs of trades fully closed by this order.
    #[serde(rename = "tradeClosedIDs")]
    pub trade_closed_ids: Option<Vec<TradeID>>,
    /// ID of the transaction that cancelled this order, if cancelled.
    #[serde(rename = "cancellingTransactionID")]
    pub cancelling_transaction_id: Option<TransactionID>,
    /// Timestamp at which the order was cancelled.
    pub cancelled_time: Option<DateTime<Utc>>,
    /// The ID of the order this order replaced, if applicable.
    #[serde(rename = "replacesOrderID")]
    pub replaces_order_id: Option<OrderID>,
    /// The ID of the order that replaced this order, if applicable.
    #[serde(rename = "replacedByOrderID")]
    pub replaced_by_order_id: Option<OrderID>,
}

// ---------------------------------------------------------------------------
// DynamicOrderState
// ---------------------------------------------------------------------------

/// The price-dependent (dynamic) state of a pending order, returned as part
/// of [`AccountChangesState`](crate::account::AccountChangesState).
///
/// Contains only the fields that change as the market moves; the static order
/// fields are in the corresponding [`Order`] variant.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DynamicOrderState {
    /// The order's unique identifier.
    pub id: OrderID,
    /// Current absolute stop price for a trailing stop-loss order,
    /// recalculated as the market moves.
    pub trailing_stop_value: Option<PriceValue>,
    /// The distance between the current market price and the order's trigger
    /// price. Positive = the order has not yet triggered.
    pub trigger_distance: Option<PriceValue>,
    /// `true` if `trigger_distance` is an exact value; `false` if it is
    /// approximate (e.g. when the market is closed).
    pub is_trigger_distance_exact: Option<bool>,
}

// ---------------------------------------------------------------------------
// Enums
// ---------------------------------------------------------------------------

/// Discriminant used in the `"type"` field of serialised order and order-request objects.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum OrderType {
    Market,
    Limit,
    Stop,
    MarketIfTouched,
    TakeProfit,
    StopLoss,
    GuaranteedStopLoss,
    TrailingStopLoss,
    FixedPrice,
}

/// The subset of [`OrderType`] values that can be cancelled by a client request.
///
/// Market orders cannot be cancelled once submitted; fixed-price orders are
/// internal.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum CancellableOrderType {
    Limit,
    Stop,
    MarketIfTouched,
    TakeProfit,
    StopLoss,
    GuaranteedStopLoss,
    TrailingStopLoss,
}

/// The lifecycle state of an order.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum OrderState {
    /// The order is active and waiting to be triggered or filled.
    Pending,
    /// The order has been fully executed.
    Filled,
    /// The order has been triggered (applies to stop/MIT orders that become
    /// market orders before filling).
    Triggered,
    /// The order has been cancelled and is no longer active.
    Cancelled,
}

/// Extends [`OrderState`] with an `All` variant for use as a filter parameter
/// when listing orders.
#[derive(Debug, Serialize, Deserialize, Display)]
#[serde(rename_all = "UPPERCASE")]
#[strum(serialize_all = "SCREAMING_SNAKE_CASE")]
pub enum OrderStateFilter {
    /// Return only pending orders.
    Pending,
    /// Return only filled orders.
    Filled,
    /// Return only triggered orders.
    Triggered,
    /// Return only cancelled orders.
    Cancelled,
    /// Return orders in any state.
    All,
}

/// Pairs an OANDA-assigned order ID with its client-provided order ID.
#[derive(Debug, Serialize, Deserialize)]
pub struct OrderIdentifier {
    /// The OANDA-assigned order ID.
    #[serde(rename = "orderID")]
    pub order_id: OrderID,
    /// The client-provided order ID.
    #[serde(rename = "clientOrderID")]
    pub client_order_id: ClientID,
}

/// A string that uniquely identifies an order within an account.
///
/// Can be either an OANDA-assigned [`OrderID`] or a client-provided order ID
/// prefixed with `"@"` (e.g. `"@my-order-id"`).
pub type OrderSpecifier = String;

/// Controls how long a pending order remains active.
#[derive(Debug, Serialize, Deserialize)]
pub enum TimeInForce {
    /// Good-till-cancelled: the order remains active until filled or cancelled.
    GTC,
    /// Good-till-date: the order expires at `gtd_time` if not filled before then.
    GTD,
    /// Good-for-day: the order expires at the end of the current trading day.
    GFD,
    /// Fill-or-kill: the order must be filled in its entirety immediately or
    /// it is cancelled.
    FOK,
    /// Immediate-or-cancel: as much of the order as possible is filled
    /// immediately; the rest is cancelled.
    IOC,
}

/// Determines how an order interacts with an existing position on the same
/// instrument.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum OrderPositionFill {
    /// Only open new trades; do not reduce existing positions.
    OpenOnly,
    /// Reduce existing opposite-side positions first, then open new trades
    /// with any remaining units.
    ReduceFirst,
    /// Only reduce existing positions; do not open new trades.
    ReduceOnly,
    /// Use the account's default position fill behaviour.
    Default,
}

/// Specifies which price stream is used to determine when a pending order triggers.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum OrderTriggerCondition {
    /// Use the account's default trigger condition (typically the opposite side
    /// of the trade: bid for long, ask for short).
    Default,
    /// Trigger on the price opposite to the default (ask for long, bid for short).
    Inverse,
    /// Trigger when the bid price crosses the order price.
    Bid,
    /// Trigger when the ask price crosses the order price.
    Ask,
    /// Trigger when the mid price crosses the order price.
    Mid,
}
