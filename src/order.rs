use crate::client::Client;
use crate::errors::APIError;
use crate::instrument::InstrumentName;
use crate::pricing::PriceValue;
use crate::primitives::DecimalNumber;
use crate::transaction::{
    ClientExtensions, ClientID, GuaranteedStopLossDetails, MarketOrderDelayedTradeClose,
    MarketOrderMarginCloseout, MarketOrderPositionCloseout, MarketOrderTradeClose,
    OrderCancelTransaction, OrderCreateRejectTransaction, OrderCreateTransaction,
    OrderFillTransaction, StopLossDetails, TakeProfitDetails, TradeID, TrailingStopLossDetails,
    TransactionID,
};
use crate::{request_option_setter, request_setter};
use chrono::{DateTime, Utc};
use reqwest::{Request, StatusCode};
use serde::{Deserialize, Serialize};
use std::ops::Not;
use strum_macros::Display;
use url::Url;

/// A unique identifier for an order (e.g. `"12345"`).
pub type OrderID = String;

// ---------------------------------------------------------------------------
// Order enum (tagged union)
// ---------------------------------------------------------------------------

/// A polymorphic representation of any order type returned by the OANDA API.
///
/// Deserialised from JSON using the `"type"` field as a tag, so each variant
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
pub struct MarketOrder {
    /// The order's unique identifier.
    pub id: OrderID,
    /// Timestamp at which the order was created.
    #[serde(rename = "createTime")]
    pub create_time: DateTime<Utc>,
    /// Current lifecycle state of the order.
    pub state: OrderState,
    /// Optional client-supplied metadata attached to the order.
    #[serde(rename = "clientExtensions", skip_serializing_if = "Option::is_none")]
    pub client_extensions: Option<ClientExtensions>,
    /// The instrument to be traded.
    pub instrument: InstrumentName,
    /// Number of units to trade. Positive = buy (long), negative = sell (short).
    pub units: DecimalNumber,
    /// How long the order remains active (`FOK` or `IOC` for market orders).
    #[serde(rename = "timeInForce")]
    pub time_in_force: TimeInForce,
    /// The worst fill price acceptable. If the order cannot be filled within
    /// this bound, it is cancelled.
    #[serde(rename = "priceBound")]
    pub price_bound: Option<PriceValue>,
    /// How the order interacts with an existing position on the instrument.
    #[serde(rename = "positionFill")]
    pub position_fill: OrderPositionFill,
    /// Take-profit order to attach to any trade opened by this order.
    #[serde(rename = "takeProfitOnFill")]
    pub take_profit_on_fill: Option<TakeProfitDetails>,
    /// Stop-loss order to attach to any trade opened by this order.
    #[serde(rename = "stopLossOnFill")]
    pub stop_loss_on_fill: Option<StopLossDetails>,
    /// Trailing stop-loss to attach to any trade opened by this order.
    #[serde(rename = "trailingStopLossOnFill")]
    pub trailing_stop_loss_on_fill: Option<TrailingStopLossDetails>,
    /// Guaranteed stop-loss to attach to any trade opened by this order.
    #[serde(rename = "guaranteedStopLossOnFill")]
    pub guaranteed_stop_loss_on_fill: Option<GuaranteedStopLossDetails>,
    /// Client extensions to apply to the trade opened by this order.
    #[serde(rename = "tradeClientExtensions")]
    pub trade_client_extensions: Option<ClientExtensions>,
    /// ID of the transaction that filled this order, if it has been filled.
    #[serde(rename = "fillingTransactionID")]
    pub filling_transaction_id: Option<TransactionID>,
    /// Timestamp at which the order was filled.
    #[serde(rename = "filledTime")]
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
    #[serde(rename = "cancelledTime")]
    pub cancelled_time: Option<DateTime<Utc>>,
    /// Details of the trade close this market order was created to perform.
    #[serde(rename = "tradeClose")]
    pub trade_close: Option<MarketOrderTradeClose>,
    /// Details of the long position closeout this order was created to perform.
    #[serde(rename = "longPositionCloseout")]
    pub long_position_closeout: Option<MarketOrderPositionCloseout>,
    /// Details of the short position closeout this order was created to perform.
    #[serde(rename = "shortPositionCloseout")]
    pub short_position_closeout: Option<MarketOrderPositionCloseout>,
    /// Details when this order was created as part of a margin closeout.
    #[serde(rename = "marginCloseout")]
    pub margin_closeout: Option<MarketOrderMarginCloseout>,
    /// Details when this order was created to close a trade that could not be
    /// closed at the time it was reduced.
    #[serde(rename = "delayedTradeClose")]
    pub delayed_trade_close: Option<MarketOrderDelayedTradeClose>,
}

/// A fixed-price order as returned by the OANDA API.
///
/// An internal order type created by OANDA to fill a trade at a specific fixed
/// price, for example during account corrections or transfers. Cannot be created
/// by clients directly.
#[derive(Debug, Deserialize, Serialize)]
pub struct FixedPriceOrder {
    /// The order's unique identifier.
    pub id: OrderID,
    /// Timestamp at which the order was created.
    #[serde(rename = "createTime")]
    pub create_time: DateTime<Utc>,
    /// Current lifecycle state of the order.
    pub state: OrderState,
    /// Optional client-supplied metadata attached to the order.
    #[serde(rename = "clientExtensions")]
    pub client_extensions: Option<ClientExtensions>,
    /// The instrument to be traded.
    pub instrument: InstrumentName,
    /// Number of units to trade. Positive = buy (long), negative = sell (short).
    pub units: DecimalNumber,
    /// The fixed price at which the order will be filled.
    pub price: PriceValue,
    /// How the order interacts with an existing position on the instrument.
    #[serde(rename = "positionFill")]
    pub position_fill: OrderPositionFill,
    /// The state of the trade the order is intended to result in.
    #[serde(rename = "tradeState")]
    pub trade_state: String,
    /// Take-profit order to attach to any trade opened by this order.
    #[serde(rename = "takeProfitOnFill")]
    pub take_profit_on_fill: Option<TakeProfitDetails>,
    /// Stop-loss order to attach to any trade opened by this order.
    #[serde(rename = "stopLossOnFill")]
    pub stop_loss_on_fill: Option<StopLossDetails>,
    /// Trailing stop-loss to attach to any trade opened by this order.
    #[serde(rename = "trailingStopLossOnFill")]
    pub trailing_stop_loss_on_fill: Option<TrailingStopLossDetails>,
    /// Guaranteed stop-loss to attach to any trade opened by this order.
    #[serde(rename = "guaranteedStopLossOnFill")]
    pub guaranteed_stop_loss_on_fill: Option<GuaranteedStopLossDetails>,
    /// Client extensions to apply to the trade opened by this order.
    #[serde(rename = "tradeClientExtensions")]
    pub trade_client_extensions: Option<ClientExtensions>,
    /// ID of the transaction that filled this order, if filled.
    #[serde(rename = "fillingTransactionID")]
    pub filling_transaction_id: Option<TransactionID>,
    /// Timestamp at which the order was filled.
    #[serde(rename = "filledTime")]
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
    #[serde(rename = "cancelledTime")]
    pub cancelled_time: Option<DateTime<Utc>>,
}

/// A limit order as returned by the OANDA API.
///
/// Executes at `price` or better once the market reaches that level.
/// Supports `GTC`, `GTD`, and `GFD` time-in-force values.
#[derive(Debug, Deserialize, Serialize)]
pub struct LimitOrder {
    /// The order's unique identifier.
    pub id: OrderID,
    /// Timestamp at which the order was created.
    #[serde(rename = "createTime")]
    pub create_time: DateTime<Utc>,
    /// Current lifecycle state of the order.
    pub state: OrderState,
    /// Optional client-supplied metadata attached to the order.
    #[serde(rename = "clientExtensions")]
    pub client_extensions: Option<ClientExtensions>,
    /// The instrument to be traded.
    pub instrument: InstrumentName,
    /// Number of units to trade. Positive = buy (long), negative = sell (short).
    pub units: DecimalNumber,
    /// The limit price at which the order will execute.
    pub price: PriceValue,
    /// How long the order remains active.
    #[serde(rename = "timeInForce")]
    pub time_in_force: TimeInForce,
    /// Expiry timestamp when `time_in_force` is `GTD`.
    #[serde(rename = "gtdTime")]
    pub gtd_time: Option<DateTime<Utc>>,
    /// How the order interacts with an existing position on the instrument.
    #[serde(rename = "positionFill")]
    pub position_fill: OrderPositionFill,
    /// Which price stream triggers the order.
    #[serde(rename = "triggerCondition")]
    pub trigger_condition: OrderTriggerCondition,
    /// Take-profit order to attach to any trade opened by this order.
    #[serde(rename = "takeProfitOnFill")]
    pub take_profit_on_fill: Option<TakeProfitDetails>,
    /// Stop-loss order to attach to any trade opened by this order.
    #[serde(rename = "stopLossOnFill")]
    pub stop_loss_on_fill: Option<StopLossDetails>,
    /// Trailing stop-loss to attach to any trade opened by this order.
    #[serde(rename = "trailingStopLossOnFill")]
    pub trailing_stop_loss_on_fill: Option<TrailingStopLossDetails>,
    /// Guaranteed stop-loss to attach to any trade opened by this order.
    #[serde(rename = "guaranteedStopLossOnFill")]
    pub guaranteed_stop_loss_on_fill: Option<GuaranteedStopLossDetails>,
    /// Client extensions to apply to the trade opened by this order.
    #[serde(rename = "tradeClientExtensions")]
    pub trade_client_extensions: Option<ClientExtensions>,
    /// ID of the transaction that filled this order, if filled.
    #[serde(rename = "fillingTransactionID")]
    pub filling_transaction_id: Option<TransactionID>,
    /// Timestamp at which the order was filled.
    #[serde(rename = "filledTime")]
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
    #[serde(rename = "cancelledTime")]
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
pub struct StopOrder {
    /// The order's unique identifier.
    pub id: OrderID,
    /// Timestamp at which the order was created.
    #[serde(rename = "createTime")]
    pub create_time: DateTime<Utc>,
    /// Current lifecycle state of the order.
    pub state: OrderState,
    /// Optional client-supplied metadata attached to the order.
    #[serde(rename = "clientExtensions")]
    pub client_extensions: Option<ClientExtensions>,
    /// The instrument to be traded.
    pub instrument: InstrumentName,
    /// Number of units to trade. Positive = buy (long), negative = sell (short).
    pub units: DecimalNumber,
    /// The stop trigger price.
    pub price: PriceValue,
    /// The worst fill price acceptable after the stop triggers.
    #[serde(rename = "priceBound")]
    pub price_bound: Option<PriceValue>,
    /// How long the order remains active.
    #[serde(rename = "timeInForce")]
    pub time_in_force: TimeInForce,
    /// Expiry timestamp when `time_in_force` is `GTD`.
    #[serde(rename = "gtdTime")]
    pub gtd_time: Option<DateTime<Utc>>,
    /// How the order interacts with an existing position on the instrument.
    #[serde(rename = "positionFill")]
    pub position_fill: OrderPositionFill,
    /// Which price stream triggers the order.
    #[serde(rename = "triggerCondition")]
    pub trigger_condition: OrderTriggerCondition,
    /// Take-profit order to attach to any trade opened by this order.
    #[serde(rename = "takeProfitOnFill")]
    pub take_profit_on_fill: Option<TakeProfitDetails>,
    /// Stop-loss order to attach to any trade opened by this order.
    #[serde(rename = "stopLossOnFill")]
    pub stop_loss_on_fill: Option<StopLossDetails>,
    /// Trailing stop-loss to attach to any trade opened by this order.
    #[serde(rename = "trailingStopLossOnFill")]
    pub trailing_stop_loss_on_fill: Option<TrailingStopLossDetails>,
    /// Guaranteed stop-loss to attach to any trade opened by this order.
    #[serde(rename = "guaranteedStopLossOnFill")]
    pub guaranteed_stop_loss_on_fill: Option<GuaranteedStopLossDetails>,
    /// Client extensions to apply to the trade opened by this order.
    #[serde(rename = "tradeClientExtensions")]
    pub trade_client_extensions: Option<ClientExtensions>,
    /// ID of the transaction that filled this order, if filled.
    #[serde(rename = "fillingTransactionID")]
    pub filling_transaction_id: Option<TransactionID>,
    /// Timestamp at which the order was filled.
    #[serde(rename = "filledTime")]
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
    #[serde(rename = "cancelledTime")]
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
pub struct MarketIfTouchedOrder {
    /// The order's unique identifier.
    pub id: OrderID,
    /// Timestamp at which the order was created.
    #[serde(rename = "createTime")]
    pub create_time: DateTime<Utc>,
    /// Current lifecycle state of the order.
    pub state: OrderState,
    /// Optional client-supplied metadata attached to the order.
    #[serde(rename = "clientExtensions")]
    pub client_extensions: Option<ClientExtensions>,
    /// The instrument to be traded.
    pub instrument: InstrumentName,
    /// Number of units to trade. Positive = buy (long), negative = sell (short).
    pub units: DecimalNumber,
    /// The trigger price. When the market touches this level the order converts
    /// to a market order.
    pub price: PriceValue,
    /// Worst acceptable fill price after the order triggers.
    #[serde(rename = "priceBound")]
    pub price_bound: Option<PriceValue>,
    /// How long the order remains active.
    #[serde(rename = "timeInForce")]
    pub time_in_force: TimeInForce,
    /// Expiry timestamp when `time_in_force` is `GTD`.
    #[serde(rename = "gtdTime")]
    pub gtd_time: Option<DateTime<Utc>>,
    /// How the order interacts with an existing position on the instrument.
    #[serde(rename = "positionFill")]
    pub position_fill: OrderPositionFill,
    /// Which price stream triggers the order.
    #[serde(rename = "triggerCondition")]
    pub trigger_condition: OrderTriggerCondition,
    /// The market price at the time the order was created.
    #[serde(rename = "initialMarketPrice")]
    pub initial_market_price: PriceValue,
    /// Take-profit order to attach to any trade opened by this order.
    #[serde(rename = "takeProfitOnFill")]
    pub take_profit_on_fill: Option<TakeProfitDetails>,
    /// Stop-loss order to attach to any trade opened by this order.
    #[serde(rename = "stopLossOnFill")]
    pub stop_loss_on_fill: Option<StopLossDetails>,
    /// Trailing stop-loss to attach to any trade opened by this order.
    #[serde(rename = "trailingStopLossOnFill")]
    pub trailing_stop_loss_on_fill: Option<TrailingStopLossDetails>,
    /// Guaranteed stop-loss to attach to any trade opened by this order.
    #[serde(rename = "guaranteedStopLossOnFill")]
    pub guaranteed_stop_loss_on_fill: Option<GuaranteedStopLossDetails>,
    /// Client extensions to apply to the trade opened by this order.
    #[serde(rename = "tradeClientExtensions")]
    pub trade_client_extensions: Option<ClientExtensions>,
    /// ID of the transaction that filled this order, if filled.
    #[serde(rename = "fillingTransactionID")]
    pub filling_transaction_id: Option<TransactionID>,
    /// Timestamp at which the order was filled.
    #[serde(rename = "filledTime")]
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
    #[serde(rename = "cancelledTime")]
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
pub struct TakeProfitOrder {
    /// The order's unique identifier.
    pub id: OrderID,
    /// Timestamp at which the order was created.
    #[serde(rename = "createTime")]
    pub create_time: DateTime<Utc>,
    /// Current lifecycle state of the order.
    pub state: OrderState,
    /// Optional client-supplied metadata attached to the order.
    #[serde(rename = "clientExtensions")]
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
    #[serde(rename = "timeInForce")]
    pub time_in_force: TimeInForce,
    /// Expiry timestamp when `time_in_force` is `GTD`.
    #[serde(rename = "gtdTime")]
    pub gtd_time: Option<DateTime<Utc>>,
    /// Which price stream triggers the order.
    #[serde(rename = "triggerCondition")]
    pub trigger_condition: OrderTriggerCondition,
    /// ID of the transaction that filled this order, if filled.
    #[serde(rename = "fillingTransactionID")]
    pub filling_transaction_id: Option<TransactionID>,
    /// Timestamp at which the order was filled.
    #[serde(rename = "filledTime")]
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
    #[serde(rename = "cancelledTime")]
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
pub struct StopLossOrder {
    /// The order's unique identifier.
    pub id: OrderID,
    /// Timestamp at which the order was created.
    #[serde(rename = "createTime")]
    pub create_time: DateTime<Utc>,
    /// Current lifecycle state of the order.
    pub state: OrderState,
    /// Optional client-supplied metadata attached to the order.
    #[serde(rename = "clientExtensions")]
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
    #[serde(rename = "timeInForce")]
    pub time_in_force: TimeInForce,
    /// Expiry timestamp when `time_in_force` is `GTD`.
    #[serde(rename = "gtdTime")]
    pub gtd_time: Option<DateTime<Utc>>,
    /// Which price stream triggers the order.
    #[serde(rename = "triggerCondition")]
    pub trigger_condition: OrderTriggerCondition,
    /// ID of the transaction that filled this order, if filled.
    #[serde(rename = "fillingTransactionID")]
    pub filling_transaction_id: Option<TransactionID>,
    /// Timestamp at which the order was filled.
    #[serde(rename = "filledTime")]
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
    #[serde(rename = "cancelledTime")]
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
pub struct GuaranteedStopLossOrder {
    /// The order's unique identifier.
    pub id: OrderID,
    /// Timestamp at which the order was created.
    #[serde(rename = "createTime")]
    pub create_time: DateTime<Utc>,
    /// Current lifecycle state of the order.
    pub state: OrderState,
    /// Optional client-supplied metadata attached to the order.
    #[serde(rename = "clientExtensions")]
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
    #[serde(rename = "timeInForce")]
    pub time_in_force: TimeInForce,
    /// Expiry timestamp when `time_in_force` is `GTD`.
    #[serde(rename = "gtdTime")]
    pub gtd_time: Option<DateTime<Utc>>,
    /// Which price stream triggers the order.
    #[serde(rename = "triggerCondition")]
    pub trigger_condition: OrderTriggerCondition,
    /// The premium paid for guaranteed execution.
    #[serde(rename = "guaranteedExecutionPremium")]
    pub guaranteed_execution_premium: DecimalNumber,
    /// ID of the transaction that filled this order, if filled.
    #[serde(rename = "fillingTransactionID")]
    pub filling_transaction_id: Option<TransactionID>,
    /// Timestamp at which the order was filled.
    #[serde(rename = "filledTime")]
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
    #[serde(rename = "cancelledTime")]
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
pub struct TrailingStopLossOrder {
    /// The order's unique identifier.
    pub id: OrderID,
    /// Timestamp at which the order was created.
    #[serde(rename = "createTime")]
    pub create_time: DateTime<Utc>,
    /// Current lifecycle state of the order.
    pub state: OrderState,
    /// Optional client-supplied metadata attached to the order.
    #[serde(rename = "clientExtensions")]
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
    #[serde(rename = "timeInForce")]
    pub time_in_force: TimeInForce,
    /// Expiry timestamp when `time_in_force` is `GTD`.
    #[serde(rename = "gtdTime")]
    pub gtd_time: Option<DateTime<Utc>>,
    /// Which price stream triggers the order.
    #[serde(rename = "triggerCondition")]
    pub trigger_condition: OrderTriggerCondition,
    /// The current calculated absolute stop price, derived from `distance`
    /// and the current market price.
    #[serde(rename = "trailingStopValue")]
    pub trailing_stop_value: PriceValue,
    /// ID of the transaction that filled this order, if filled.
    #[serde(rename = "fillingTransactionID")]
    pub filling_transaction_id: Option<TransactionID>,
    /// Timestamp at which the order was filled.
    #[serde(rename = "filledTime")]
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
    #[serde(rename = "cancelledTime")]
    pub cancelled_time: Option<DateTime<Utc>>,
    /// The ID of the order this order replaced, if applicable.
    #[serde(rename = "replacesOrderID")]
    pub replaces_order_id: Option<OrderID>,
    /// The ID of the order that replaced this order, if applicable.
    #[serde(rename = "replacedByOrderID")]
    pub replaced_by_order_id: Option<OrderID>,
}

// ---------------------------------------------------------------------------
// Order Request Types (for creating/replacing orders)
// ---------------------------------------------------------------------------

/// A polymorphic order creation request.
///
/// Wrap a concrete `*OrderRequest` struct in the appropriate variant and pass
/// it to [`OrderService::create`] or [`OrderService::replace`]. The `"type"`
/// field is written automatically by serde.
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum OrderRequest {
    #[serde(rename = "MARKET")]
    Market(MarketOrderRequest),
    #[serde(rename = "LIMIT")]
    Limit(LimitOrderRequest),
    #[serde(rename = "STOP")]
    Stop(StopOrderRequest),
    #[serde(rename = "MARKET_IF_TOUCHED")]
    MarketIfTouched(MarketIfTouchedOrderRequest),
    #[serde(rename = "TAKE_PROFIT")]
    TakeProfit(TakeProfitOrderRequest),
    #[serde(rename = "STOP_LOSS")]
    StopLoss(StopLossOrderRequest),
    #[serde(rename = "GUARANTEED_STOP_LOSS")]
    GuaranteedStopLoss(GuaranteedStopLossOrderRequest),
    #[serde(rename = "TRAILING_STOP_LOSS")]
    TrailingStopLoss(TrailingStopLossOrderRequest),
}

/// Request body for creating a market order.
///
/// Build with [`MarketOrderRequest::new`] then pass to [`OrderService::create`]
/// wrapped in [`OrderRequest::Market`].
///
/// Defaults: `time_in_force = FOK`, `position_fill = Default`.
#[derive(Debug, Serialize, Deserialize)]
pub struct MarketOrderRequest {
    /// Always `OrderType::Market`; set automatically by `new`.
    #[serde(rename = "type")]
    pub order_type: OrderType,
    /// The instrument to trade.
    pub instrument: InstrumentName,
    /// Units to trade. Positive = buy (long), negative = sell (short).
    pub units: DecimalNumber,
    /// Time-in-force (`FOK` or `IOC` for market orders).
    #[serde(rename = "timeInForce")]
    pub time_in_force: TimeInForce,
    /// Worst acceptable fill price. Omitted if `None`.
    #[serde(rename = "priceBound", skip_serializing_if = "Option::is_none")]
    pub price_bound: Option<PriceValue>,
    /// How the order interacts with an existing position.
    #[serde(rename = "positionFill")]
    pub position_fill: OrderPositionFill,
    /// Client metadata to attach to the order. Omitted if `None`.
    #[serde(rename = "clientExtensions", skip_serializing_if = "Option::is_none")]
    pub client_extensions: Option<ClientExtensions>,
    /// Take-profit to attach to any resulting trade. Omitted if `None`.
    #[serde(rename = "takeProfitOnFill", skip_serializing_if = "Option::is_none")]
    pub take_profit_on_fill: Option<TakeProfitDetails>,
    /// Stop-loss to attach to any resulting trade. Omitted if `None`.
    #[serde(rename = "stopLossOnFill", skip_serializing_if = "Option::is_none")]
    pub stop_loss_on_fill: Option<StopLossDetails>,
    /// Guaranteed stop-loss to attach to any resulting trade. Omitted if `None`.
    #[serde(
        rename = "guaranteedStopLossOnFill",
        skip_serializing_if = "Option::is_none"
    )]
    pub guaranteed_stop_loss_on_fill: Option<GuaranteedStopLossDetails>,
    /// Trailing stop-loss to attach to any resulting trade. Omitted if `None`.
    #[serde(
        rename = "trailingStopLossOnFill",
        skip_serializing_if = "Option::is_none"
    )]
    pub trailing_stop_loss_on_fill: Option<TrailingStopLossDetails>,
    /// Client extensions to apply to the resulting trade. Omitted if `None`.
    #[serde(
        rename = "tradeClientExtensions",
        skip_serializing_if = "Option::is_none"
    )]
    pub trade_client_extensions: Option<ClientExtensions>,
}

impl MarketOrderRequest {
    /// Creates a new market order request for `instrument` with the given `units`.
    ///
    /// Defaults: `time_in_force = FOK`, `position_fill = Default`.
    pub fn new(instrument: InstrumentName, units: DecimalNumber) -> MarketOrderRequest {
        MarketOrderRequest {
            order_type: OrderType::Market,
            instrument,
            units,
            time_in_force: TimeInForce::FOK,
            price_bound: None,
            position_fill: OrderPositionFill::Default,
            client_extensions: None,
            take_profit_on_fill: None,
            stop_loss_on_fill: None,
            guaranteed_stop_loss_on_fill: None,
            trailing_stop_loss_on_fill: None,
            trade_client_extensions: None,
        }
    }

    /// Sets `time_in_force` to `IOC` (immediate-or-cancel).
    ///
    /// By default market orders use `FOK` (fill-or-kill).
    pub fn ioc(mut self) -> Self {
        self.time_in_force = TimeInForce::IOC;
        self
    }

    request_setter!(position_fill, OrderPositionFill);
    request_option_setter!(price_bound, PriceValue);
    request_option_setter!(client_extensions, ClientExtensions);
    request_option_setter!(take_profit_on_fill, TakeProfitDetails);
    request_option_setter!(stop_loss_on_fill, StopLossDetails);
    request_option_setter!(guaranteed_stop_loss_on_fill, GuaranteedStopLossDetails);
    request_option_setter!(trailing_stop_loss_on_fill, TrailingStopLossDetails);
    request_option_setter!(trade_client_extensions, ClientExtensions);
}

/// Request body for creating a limit order.
///
/// Build with [`LimitOrderRequest::new`] then pass to [`OrderService::create`]
/// wrapped in [`OrderRequest::Limit`].
///
/// Defaults: `time_in_force = GTC`, `position_fill = Default`,
/// `trigger_condition = Default`.
#[derive(Debug, Serialize, Deserialize)]
pub struct LimitOrderRequest {
    /// Always `OrderType::Limit`; set automatically by `new`.
    #[serde(rename = "type")]
    pub order_type: OrderType,
    /// The instrument to trade.
    pub instrument: InstrumentName,
    /// Units to trade. Positive = buy (long), negative = sell (short).
    pub units: DecimalNumber,
    /// The limit price. The order fills only at this price or better.
    pub price: PriceValue,
    /// How long the order remains active.
    #[serde(rename = "timeInForce")]
    pub time_in_force: TimeInForce,
    /// Expiry timestamp used when `time_in_force` is changed to `GTD` via [`Self::gtd`].
    #[serde(rename = "gtdTime", skip_serializing_if = "Option::is_none")]
    pub gtd_time: Option<DateTime<Utc>>,
    /// How the order interacts with an existing position.
    #[serde(rename = "positionFill")]
    pub position_fill: OrderPositionFill,
    /// Which price stream triggers the order.
    #[serde(rename = "triggerCondition")]
    pub trigger_condition: OrderTriggerCondition,
    /// Client metadata to attach to the order. Omitted if `None`.
    #[serde(rename = "clientExtensions", skip_serializing_if = "Option::is_none")]
    pub client_extensions: Option<ClientExtensions>,
    /// Take-profit to attach to any resulting trade. Omitted if `None`.
    #[serde(rename = "takeProfitOnFill", skip_serializing_if = "Option::is_none")]
    pub take_profit_on_fill: Option<TakeProfitDetails>,
    /// Stop-loss to attach to any resulting trade. Omitted if `None`.
    #[serde(rename = "stopLossOnFill", skip_serializing_if = "Option::is_none")]
    pub stop_loss_on_fill: Option<StopLossDetails>,
    /// Guaranteed stop-loss to attach to any resulting trade. Omitted if `None`.
    #[serde(
        rename = "guaranteedStopLossOnFill",
        skip_serializing_if = "Option::is_none"
    )]
    pub guaranteed_stop_loss_on_fill: Option<GuaranteedStopLossDetails>,
    /// Trailing stop-loss to attach to any resulting trade. Omitted if `None`.
    #[serde(
        rename = "trailingStopLossOnFill",
        skip_serializing_if = "Option::is_none"
    )]
    pub trailing_stop_loss_on_fill: Option<TrailingStopLossDetails>,
    /// Client extensions to apply to the resulting trade. Omitted if `None`.
    #[serde(
        rename = "tradeClientExtensions",
        skip_serializing_if = "Option::is_none"
    )]
    pub trade_client_extensions: Option<ClientExtensions>,
}

impl LimitOrderRequest {
    /// Creates a new limit order request.
    ///
    /// Defaults: `time_in_force = GTC`, `position_fill = Default`,
    /// `trigger_condition = Default`.
    pub fn new(
        instrument: InstrumentName,
        units: DecimalNumber,
        price: PriceValue,
    ) -> LimitOrderRequest {
        LimitOrderRequest {
            order_type: OrderType::Limit,
            instrument,
            units,
            price,
            time_in_force: TimeInForce::GTC,
            gtd_time: None,
            position_fill: OrderPositionFill::Default,
            trigger_condition: OrderTriggerCondition::Default,
            client_extensions: None,
            take_profit_on_fill: None,
            stop_loss_on_fill: None,
            guaranteed_stop_loss_on_fill: None,
            trailing_stop_loss_on_fill: None,
            trade_client_extensions: None,
        }
    }

    /// Sets `time_in_force` to `GTD` and records the expiry timestamp.
    pub fn gtd(mut self, gtd_time: DateTime<Utc>) -> Self {
        self.time_in_force = TimeInForce::GTD;
        self.gtd_time = Some(gtd_time);
        self
    }

    /// Sets `time_in_force` to `GFD` (good-for-day).
    pub fn gfd(mut self) -> Self {
        self.time_in_force = TimeInForce::GFD;
        self
    }

    request_setter!(position_fill, OrderPositionFill);
    request_setter!(trigger_condition, OrderTriggerCondition);
    request_option_setter!(client_extensions, ClientExtensions);
    request_option_setter!(take_profit_on_fill, TakeProfitDetails);
    request_option_setter!(stop_loss_on_fill, StopLossDetails);
    request_option_setter!(guaranteed_stop_loss_on_fill, GuaranteedStopLossDetails);
    request_option_setter!(trailing_stop_loss_on_fill, TrailingStopLossDetails);
    request_option_setter!(trade_client_extensions, ClientExtensions);
}

/// Request body for creating a stop order.
///
/// Build with [`StopOrderRequest::new`] then pass to [`OrderService::create`]
/// wrapped in [`OrderRequest::Stop`].
///
/// Defaults: `time_in_force = GTC`, `position_fill = Default`,
/// `trigger_condition = Default`.
#[derive(Debug, Serialize, Deserialize)]
pub struct StopOrderRequest {
    /// Always `OrderType::Stop`; set automatically by `new`.
    #[serde(rename = "type")]
    pub order_type: OrderType,
    /// The instrument to trade.
    pub instrument: InstrumentName,
    /// Units to trade. Positive = buy (long), negative = sell (short).
    pub units: DecimalNumber,
    /// The stop trigger price.
    pub price: PriceValue,
    /// Worst acceptable fill price after the stop triggers. Omitted if `None`.
    #[serde(rename = "priceBound", skip_serializing_if = "Option::is_none")]
    pub price_bound: Option<PriceValue>,
    /// How long the order remains active.
    #[serde(rename = "timeInForce")]
    pub time_in_force: TimeInForce,
    /// Expiry timestamp used when `time_in_force` is `GTD`.
    #[serde(rename = "gtdTime", skip_serializing_if = "Option::is_none")]
    pub gtd_time: Option<DateTime<Utc>>,
    /// How the order interacts with an existing position.
    #[serde(rename = "positionFill")]
    pub position_fill: OrderPositionFill,
    /// Which price stream triggers the order.
    #[serde(rename = "triggerCondition")]
    pub trigger_condition: OrderTriggerCondition,
    /// Client metadata to attach to the order. Omitted if `None`.
    #[serde(rename = "clientExtensions", skip_serializing_if = "Option::is_none")]
    pub client_extensions: Option<ClientExtensions>,
    /// Take-profit to attach to any resulting trade. Omitted if `None`.
    #[serde(rename = "takeProfitOnFill", skip_serializing_if = "Option::is_none")]
    pub take_profit_on_fill: Option<TakeProfitDetails>,
    /// Stop-loss to attach to any resulting trade. Omitted if `None`.
    #[serde(rename = "stopLossOnFill", skip_serializing_if = "Option::is_none")]
    pub stop_loss_on_fill: Option<StopLossDetails>,
    /// Guaranteed stop-loss to attach to any resulting trade. Omitted if `None`.
    #[serde(
        rename = "guaranteedStopLossOnFill",
        skip_serializing_if = "Option::is_none"
    )]
    pub guaranteed_stop_loss_on_fill: Option<GuaranteedStopLossDetails>,
    /// Trailing stop-loss to attach to any resulting trade. Omitted if `None`.
    #[serde(
        rename = "trailingStopLossOnFill",
        skip_serializing_if = "Option::is_none"
    )]
    pub trailing_stop_loss_on_fill: Option<TrailingStopLossDetails>,
    /// Client extensions to apply to the resulting trade. Omitted if `None`.
    #[serde(
        rename = "tradeClientExtensions",
        skip_serializing_if = "Option::is_none"
    )]
    pub trade_client_extensions: Option<ClientExtensions>,
}

impl StopOrderRequest {
    /// Creates a new stop order request.
    ///
    /// Defaults: `time_in_force = GTC`, `position_fill = Default`,
    /// `trigger_condition = Default`.
    pub fn new(instrument: InstrumentName, units: DecimalNumber, price: PriceValue) -> Self {
        StopOrderRequest {
            order_type: OrderType::Stop,
            instrument,
            units,
            price,
            price_bound: None,
            time_in_force: TimeInForce::GTC,
            gtd_time: None,
            position_fill: OrderPositionFill::Default,
            trigger_condition: OrderTriggerCondition::Default,
            client_extensions: None,
            take_profit_on_fill: None,
            stop_loss_on_fill: None,
            guaranteed_stop_loss_on_fill: None,
            trailing_stop_loss_on_fill: None,
            trade_client_extensions: None,
        }
    }

    /// Sets `time_in_force` to `GTD` and records the expiry timestamp.
    pub fn gtd(mut self, gtd_time: DateTime<Utc>) -> Self {
        self.time_in_force = TimeInForce::GTD;
        self.gtd_time = Some(gtd_time);
        self
    }

    /// Sets `time_in_force` to `GFD` (good-for-day).
    pub fn gfd(mut self) -> Self {
        self.time_in_force = TimeInForce::GFD;
        self
    }

    request_option_setter!(price_bound, PriceValue);
    request_setter!(position_fill, OrderPositionFill);
    request_setter!(trigger_condition, OrderTriggerCondition);
    request_option_setter!(client_extensions, ClientExtensions);
    request_option_setter!(take_profit_on_fill, TakeProfitDetails);
    request_option_setter!(stop_loss_on_fill, StopLossDetails);
    request_option_setter!(guaranteed_stop_loss_on_fill, GuaranteedStopLossDetails);
    request_option_setter!(trailing_stop_loss_on_fill, TrailingStopLossDetails);
    request_option_setter!(trade_client_extensions, ClientExtensions);
}

/// Request body for creating a market-if-touched order.
///
/// Build with [`MarketIfTouchedOrderRequest::new`] then pass to
/// [`OrderService::create`] wrapped in [`OrderRequest::MarketIfTouched`].
///
/// Defaults: `time_in_force = GTC`, `position_fill = Default`,
/// `trigger_condition = Default`.
#[derive(Debug, Serialize, Deserialize)]
pub struct MarketIfTouchedOrderRequest {
    /// Always `OrderType::MarketIfTouched`; set automatically by `new`.
    #[serde(rename = "type")]
    pub order_type: OrderType,
    /// The instrument to trade.
    pub instrument: InstrumentName,
    /// Units to trade. Positive = buy (long), negative = sell (short).
    pub units: DecimalNumber,
    /// The trigger price. When touched, the order converts to a market order.
    pub price: PriceValue,
    /// Worst acceptable fill price after triggering. Omitted if `None`.
    #[serde(rename = "priceBound", skip_serializing_if = "Option::is_none")]
    pub price_bound: Option<PriceValue>,
    /// How long the order remains active.
    #[serde(rename = "timeInForce")]
    pub time_in_force: TimeInForce,
    /// Expiry timestamp used when `time_in_force` is `GTD`.
    #[serde(rename = "gtdTime", skip_serializing_if = "Option::is_none")]
    pub gtd_time: Option<DateTime<Utc>>,
    /// How the order interacts with an existing position.
    #[serde(rename = "positionFill")]
    pub position_fill: OrderPositionFill,
    /// Which price stream triggers the order.
    #[serde(rename = "triggerCondition")]
    pub trigger_condition: OrderTriggerCondition,
    /// Client metadata to attach to the order. Omitted if `None`.
    #[serde(rename = "clientExtensions", skip_serializing_if = "Option::is_none")]
    pub client_extensions: Option<ClientExtensions>,
    /// Take-profit to attach to any resulting trade. Omitted if `None`.
    #[serde(rename = "takeProfitOnFill", skip_serializing_if = "Option::is_none")]
    pub take_profit_on_fill: Option<TakeProfitDetails>,
    /// Stop-loss to attach to any resulting trade. Omitted if `None`.
    #[serde(rename = "stopLossOnFill", skip_serializing_if = "Option::is_none")]
    pub stop_loss_on_fill: Option<StopLossDetails>,
    /// Guaranteed stop-loss to attach to any resulting trade. Omitted if `None`.
    #[serde(
        rename = "guaranteedStopLossOnFill",
        skip_serializing_if = "Option::is_none"
    )]
    pub guaranteed_stop_loss_on_fill: Option<GuaranteedStopLossDetails>,
    /// Trailing stop-loss to attach to any resulting trade. Omitted if `None`.
    #[serde(
        rename = "trailingStopLossOnFill",
        skip_serializing_if = "Option::is_none"
    )]
    pub trailing_stop_loss_on_fill: Option<TrailingStopLossDetails>,
    /// Client extensions to apply to the resulting trade. Omitted if `None`.
    #[serde(
        rename = "tradeClientExtensions",
        skip_serializing_if = "Option::is_none"
    )]
    pub trade_client_extensions: Option<ClientExtensions>,
}

impl MarketIfTouchedOrderRequest {
    /// Creates a new market-if-touched order request.
    ///
    /// Defaults: `time_in_force = GTC`, `position_fill = Default`,
    /// `trigger_condition = Default`.
    pub fn new(instrument: InstrumentName, units: DecimalNumber, price: PriceValue) -> Self {
        MarketIfTouchedOrderRequest {
            order_type: OrderType::MarketIfTouched,
            instrument,
            units,
            price,
            price_bound: None,
            time_in_force: TimeInForce::GTC,
            gtd_time: None,
            position_fill: OrderPositionFill::Default,
            trigger_condition: OrderTriggerCondition::Default,
            client_extensions: None,
            take_profit_on_fill: None,
            stop_loss_on_fill: None,
            guaranteed_stop_loss_on_fill: None,
            trailing_stop_loss_on_fill: None,
            trade_client_extensions: None,
        }
    }

    /// Sets `time_in_force` to `GFD` (good-for-day).
    pub fn gfd(mut self) -> Self {
        self.time_in_force = TimeInForce::GFD;
        self
    }

    /// Sets `time_in_force` to `GTD` and records the expiry timestamp.
    pub fn gtd(mut self, gtd_time: DateTime<Utc>) -> Self {
        self.time_in_force = TimeInForce::GTD;
        self.gtd_time = Some(gtd_time);
        self
    }

    request_setter!(position_fill, OrderPositionFill);
    request_setter!(trigger_condition, OrderTriggerCondition);
    request_option_setter!(client_extensions, ClientExtensions);
    request_option_setter!(take_profit_on_fill, TakeProfitDetails);
    request_option_setter!(stop_loss_on_fill, StopLossDetails);
    request_option_setter!(guaranteed_stop_loss_on_fill, GuaranteedStopLossDetails);
    request_option_setter!(trailing_stop_loss_on_fill, TrailingStopLossDetails);
    request_option_setter!(trade_client_extensions, ClientExtensions);
}

/// Request body for creating a take-profit order on an existing trade.
///
/// Build with [`TakeProfitOrderRequest::new`] then pass to
/// [`OrderService::create`] wrapped in [`OrderRequest::TakeProfit`].
///
/// Defaults: `time_in_force = GTC`, `trigger_condition = Default`.
#[derive(Debug, Serialize, Deserialize)]
pub struct TakeProfitOrderRequest {
    /// Always `OrderType::TakeProfit`; set automatically by `new`.
    #[serde(rename = "type")]
    pub order_type: OrderType,
    /// The ID of the trade to attach the take-profit to.
    #[serde(rename = "tradeID")]
    pub trade_id: TradeID,
    /// Client-provided trade ID, if any. Omitted if `None`.
    #[serde(rename = "clientTradeID", skip_serializing_if = "Option::is_none")]
    pub client_trade_id: Option<ClientID>,
    /// The price at which the trade will be closed to take profit.
    pub price: PriceValue,
    /// How long the order remains active.
    #[serde(rename = "timeInForce")]
    pub time_in_force: TimeInForce,
    /// Expiry timestamp used when `time_in_force` is `GTD`.
    #[serde(rename = "gtdTime", skip_serializing_if = "Option::is_none")]
    pub gtd_time: Option<DateTime<Utc>>,
    /// Which price stream triggers the order.
    #[serde(rename = "triggerCondition")]
    pub trigger_condition: OrderTriggerCondition,
    /// Client metadata to attach to the order. Omitted if `None`.
    #[serde(rename = "clientExtensions", skip_serializing_if = "Option::is_none")]
    pub client_extensions: Option<ClientExtensions>,
}

impl TakeProfitOrderRequest {
    /// Creates a new take-profit order request for `trade_id` at the given `price`.
    ///
    /// Defaults: `time_in_force = GTC`, `trigger_condition = Default`.
    pub fn new(trade_id: TradeID, price: PriceValue) -> Self {
        TakeProfitOrderRequest {
            order_type: OrderType::TakeProfit,
            trade_id,
            client_trade_id: None,
            price,
            time_in_force: TimeInForce::GTC,
            gtd_time: None,
            trigger_condition: OrderTriggerCondition::Default,
            client_extensions: None,
        }
    }

    /// Sets `time_in_force` to `GFD` (good-for-day).
    pub fn gfd(mut self) -> Self {
        self.time_in_force = TimeInForce::GFD;
        self
    }

    /// Sets `time_in_force` to `GTD` and records the expiry timestamp.
    pub fn gtd(mut self, gtd_time: DateTime<Utc>) -> Self {
        self.time_in_force = TimeInForce::GTD;
        self.gtd_time = Some(gtd_time);
        self
    }

    request_setter!(trigger_condition, OrderTriggerCondition);
    request_option_setter!(client_extensions, ClientExtensions);
}

/// Request body for creating a stop-loss order on an existing trade.
///
/// Build with [`StopLossOrderRequest::new`] then pass to
/// [`OrderService::create`] wrapped in [`OrderRequest::StopLoss`].
///
/// Defaults: `time_in_force = GTC`, `trigger_condition = Default`.
#[derive(Debug, Serialize, Deserialize)]
pub struct StopLossOrderRequest {
    /// Always `OrderType::StopLoss`; set automatically by `new`.
    #[serde(rename = "type")]
    pub order_type: OrderType,
    /// The ID of the trade to attach the stop-loss to.
    #[serde(rename = "tradeID")]
    pub trade_id: TradeID,
    /// Client-provided trade ID, if any. Omitted if `None`.
    #[serde(rename = "clientTradeID", skip_serializing_if = "Option::is_none")]
    pub client_trade_id: Option<ClientID>,
    /// Absolute stop price. Mutually exclusive with `distance`.
    pub price: PriceValue,
    /// Distance from current price for the stop. Mutually exclusive with `price`.
    /// Omitted if `None`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub distance: Option<DecimalNumber>,
    /// Whether to request guaranteed execution at the stop price. Omitted if `None`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub guaranteed: Option<bool>,
    /// How long the order remains active.
    #[serde(rename = "timeInForce")]
    pub time_in_force: TimeInForce,
    /// Expiry timestamp used when `time_in_force` is `GTD`.
    #[serde(rename = "gtdTime", skip_serializing_if = "Option::is_none")]
    pub gtd_time: Option<DateTime<Utc>>,
    /// Which price stream triggers the order.
    #[serde(rename = "triggerCondition")]
    pub trigger_condition: OrderTriggerCondition,
    /// Client metadata to attach to the order. Omitted if `None`.
    #[serde(rename = "clientExtensions", skip_serializing_if = "Option::is_none")]
    pub client_extensions: Option<ClientExtensions>,
}

impl StopLossOrderRequest {
    /// Creates a new stop-loss order request for `trade_id` at the given `price`.
    ///
    /// Defaults: `time_in_force = GTC`, `trigger_condition = Default`.
    pub fn new(trade_id: TradeID, price: PriceValue) -> Self {
        StopLossOrderRequest {
            order_type: OrderType::StopLoss,
            trade_id,
            client_trade_id: None,
            price,
            distance: None,
            guaranteed: None,
            time_in_force: TimeInForce::GTC,
            gtd_time: None,
            trigger_condition: OrderTriggerCondition::Default,
            client_extensions: None,
        }
    }

    /// Sets `time_in_force` to `GFD` (good-for-day).
    pub fn gfd(mut self) -> Self {
        self.time_in_force = TimeInForce::GFD;
        self
    }

    /// Sets `time_in_force` to `GTD` and records the expiry timestamp.
    pub fn gtd(mut self, gtd_time: DateTime<Utc>) -> Self {
        self.time_in_force = TimeInForce::GTD;
        self.gtd_time = Some(gtd_time);
        self
    }

    request_option_setter!(client_trade_id, TradeID);
    request_option_setter!(distance, PriceValue);
    request_setter!(trigger_condition, OrderTriggerCondition);
    request_option_setter!(client_extensions, ClientExtensions);
}

/// Request body for creating a guaranteed stop-loss order on an existing trade.
///
/// Build with [`GuaranteedStopLossOrderRequest::new`] then pass to
/// [`OrderService::create`] wrapped in [`OrderRequest::GuaranteedStopLoss`].
///
/// Defaults: `time_in_force = GTC`, `trigger_condition = Default`.
#[derive(Debug, Serialize, Deserialize)]
pub struct GuaranteedStopLossOrderRequest {
    /// Always `OrderType::GuaranteedStopLoss`; set automatically by `new`.
    #[serde(rename = "type")]
    pub order_type: OrderType,
    /// The ID of the trade to attach the guaranteed stop-loss to.
    #[serde(rename = "tradeID")]
    pub trade_id: TradeID,
    /// Client-provided trade ID, if any. Omitted if `None`.
    #[serde(rename = "clientTradeID", skip_serializing_if = "Option::is_none")]
    pub client_trade_id: Option<ClientID>,
    /// The guaranteed stop price.
    pub price: PriceValue,
    /// Distance from current price; converted to an absolute price by the server.
    /// Omitted if `None`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub distance: Option<DecimalNumber>,
    /// How long the order remains active.
    #[serde(rename = "timeInForce")]
    pub time_in_force: TimeInForce,
    /// Expiry timestamp used when `time_in_force` is `GTD`.
    #[serde(rename = "gtdTime", skip_serializing_if = "Option::is_none")]
    pub gtd_time: Option<DateTime<Utc>>,
    /// Which price stream triggers the order.
    #[serde(rename = "triggerCondition")]
    pub trigger_condition: OrderTriggerCondition,
    /// Client metadata to attach to the order. Omitted if `None`.
    #[serde(rename = "clientExtensions", skip_serializing_if = "Option::is_none")]
    pub client_extensions: Option<ClientExtensions>,
}

impl GuaranteedStopLossOrderRequest {
    /// Creates a new guaranteed stop-loss order request for `trade_id` at `price`.
    ///
    /// Defaults: `time_in_force = GTC`, `trigger_condition = Default`.
    pub fn new(trade_id: TradeID, price: PriceValue) -> Self {
        GuaranteedStopLossOrderRequest {
            order_type: OrderType::GuaranteedStopLoss,
            trade_id,
            client_trade_id: None,
            price,
            distance: None,
            time_in_force: TimeInForce::GTC,
            gtd_time: None,
            trigger_condition: OrderTriggerCondition::Default,
            client_extensions: None,
        }
    }

    /// Sets `time_in_force` to `GFD` (good-for-day).
    pub fn gfd(mut self) -> Self {
        self.time_in_force = TimeInForce::GFD;
        self
    }

    /// Sets `time_in_force` to `GTD` and records the expiry timestamp.
    pub fn gtd(mut self, gtd_time: DateTime<Utc>) -> Self {
        self.time_in_force = TimeInForce::GTD;
        self.gtd_time = Some(gtd_time);
        self
    }

    request_option_setter!(client_trade_id, ClientID);
    request_option_setter!(distance, PriceValue);
    request_setter!(trigger_condition, OrderTriggerCondition);
    request_option_setter!(client_extensions, ClientExtensions);
}

/// Request body for creating a trailing stop-loss order on an existing trade.
///
/// Build with [`TrailingStopLossOrderRequest::new`] then pass to
/// [`OrderService::create`] wrapped in [`OrderRequest::TrailingStopLoss`].
///
/// Defaults: `time_in_force = GTC`, `trigger_condition = Default`.
#[derive(Debug, Serialize, Deserialize)]
pub struct TrailingStopLossOrderRequest {
    /// Always `OrderType::TrailingStopLoss`; set automatically by `new`.
    #[serde(rename = "type")]
    pub order_type: OrderType,
    /// The ID of the trade to attach the trailing stop-loss to.
    #[serde(rename = "tradeID")]
    pub trade_id: TradeID,
    /// Client-provided trade ID, if any. Omitted if `None`.
    #[serde(rename = "clientTradeID", skip_serializing_if = "Option::is_none")]
    pub client_trade_id: Option<ClientID>,
    /// The trailing distance kept between market price and the stop.
    pub distance: DecimalNumber,
    /// How long the order remains active.
    #[serde(rename = "timeInForce")]
    pub time_in_force: TimeInForce,
    /// Expiry timestamp used when `time_in_force` is `GTD`.
    #[serde(rename = "gtdTime", skip_serializing_if = "Option::is_none")]
    pub gtd_time: Option<DateTime<Utc>>,
    /// Which price stream triggers the order.
    #[serde(rename = "triggerCondition")]
    pub trigger_condition: OrderTriggerCondition,
    /// Client metadata to attach to the order. Omitted if `None`.
    #[serde(rename = "clientExtensions", skip_serializing_if = "Option::is_none")]
    pub client_extensions: Option<ClientExtensions>,
}

impl TrailingStopLossOrderRequest {
    /// Creates a new trailing stop-loss order request for `trade_id` with the
    /// given trailing `distance`.
    ///
    /// Defaults: `time_in_force = GTC`, `trigger_condition = Default`.
    pub fn new(trade_id: TradeID, distance: DecimalNumber) -> Self {
        TrailingStopLossOrderRequest {
            order_type: OrderType::TrailingStopLoss,
            trade_id,
            client_trade_id: None,
            distance,
            time_in_force: TimeInForce::GTC,
            gtd_time: None,
            trigger_condition: OrderTriggerCondition::Default,
            client_extensions: None,
        }
    }

    /// Sets `time_in_force` to `GFD` (good-for-day).
    pub fn gfd(mut self) -> Self {
        self.time_in_force = TimeInForce::GFD;
        self
    }

    /// Sets `time_in_force` to `GTD` and records the expiry timestamp.
    pub fn gtd(mut self, gtd_time: DateTime<Utc>) -> Self {
        self.time_in_force = TimeInForce::GTD;
        self.gtd_time = Some(gtd_time);
        self
    }

    request_option_setter!(client_trade_id, ClientID);
    request_setter!(trigger_condition, OrderTriggerCondition);
    request_option_setter!(client_extensions, ClientExtensions);
}

// ---------------------------------------------------------------------------
// Create/Replace/Cancel Request & Response Types
// ---------------------------------------------------------------------------

/// JSON request body sent to `POST /v3/accounts/{accountID}/orders`.
#[derive(Debug, Serialize)]
pub struct CreateOrderBody {
    /// The order to create.
    pub order: OrderRequest,
}

/// Response body for a successful `POST /v3/accounts/{accountID}/orders` (HTTP 201).
#[derive(Debug, Serialize, Deserialize)]
pub struct CreateOrderResponse {
    /// The transaction that recorded the order creation.
    #[serde(rename = "orderCreateTransaction")]
    pub order_create_transaction: Option<OrderCreateTransaction>,
    /// The transaction that filled the order, if it was immediately filled
    /// (e.g. a market order).
    #[serde(rename = "orderFillTransaction")]
    pub order_fill_transaction: Option<OrderFillTransaction>,
    /// The transaction that cancelled the order, if it was immediately cancelled
    /// (e.g. a FOK order that could not be filled).
    #[serde(rename = "orderCancelTransaction")]
    pub order_cancel_transaction: Option<OrderCancelTransaction>,
    /// The transaction that re-issued the order (e.g. an IOC order partially filled).
    #[serde(rename = "orderReissueTransaction")]
    pub order_reissue_transaction: Option<OrderCreateTransaction>,
    /// The transaction that rejected the re-issued order, if applicable.
    #[serde(rename = "orderReissueRejectTransaction")]
    pub order_reissue_reject_transaction: Option<OrderCreateRejectTransaction>,
    /// IDs of all transactions related to this request.
    #[serde(rename = "relatedTransactionIDs")]
    pub related_transaction_ids: Vec<TransactionID>,
    /// ID of the most recent transaction on the account after this request.
    #[serde(rename = "lastTransactionID")]
    pub last_transaction_id: TransactionID,
}

/// Response body for a successful `PUT /v3/accounts/{accountID}/orders/{orderSpecifier}`
/// (HTTP 201). Fields are raw JSON values because the response contains a
/// polymorphic union of transaction types.
#[derive(Debug, Serialize, Deserialize)]
pub struct ReplaceOrderResponse {
    /// The transaction that cancelled the replaced order.
    #[serde(rename = "orderCancelTransaction")]
    pub order_cancel_transaction: Option<OrderCancelTransaction>,
    /// The transaction that created the replacement order.
    #[serde(rename = "orderCreateTransaction")]
    pub order_create_transaction: Option<OrderCreateTransaction>,
    /// The transaction that filled the replacement order, if immediately filled.
    #[serde(rename = "orderFillTransaction")]
    pub order_fill_transaction: Option<OrderFillTransaction>,
    /// The transaction that re-issued the order, if applicable.
    #[serde(rename = "orderReissueTransaction")]
    pub order_reissue_transaction: Option<OrderCreateTransaction>,
    /// The transaction that rejected the re-issue, if applicable.
    #[serde(rename = "orderReissueRejectTransaction")]
    pub order_reissue_reject_transaction: Option<OrderCreateRejectTransaction>,
    /// IDs of all transactions related to this request.
    #[serde(rename = "relatedTransactionIDs")]
    pub related_transaction_ids: Option<Vec<TransactionID>>,
    /// ID of the most recent transaction on the account after this request.
    #[serde(rename = "lastTransactionID")]
    pub last_transaction_id: Option<TransactionID>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ReplaceOrderErrorResponse {
    #[serde(rename = "orderCancelRejectTransaction")]
    pub order_cancel_reject_transaction: Option<OrderCreateRejectTransaction>,
    #[serde(rename = "relatedTransactionIDs")]
    pub related_transaction_ids: Vec<TransactionID>,
    #[serde(rename = "lastTransactionID")]
    pub last_transaction_id: TransactionID,
    #[serde(rename = "errorCode")]
    pub error_code: String,
    #[serde(rename = "errorMessage")]
    pub error_message: String,
}

/// Response body for a successful
/// `PUT /v3/accounts/{accountID}/orders/{orderSpecifier}/cancel` (HTTP 200).
#[derive(Debug, Serialize, Deserialize)]
pub struct CancelOrderResponse {
    /// The transaction that cancelled the order.
    #[serde(rename = "orderCancelTransaction")]
    pub order_cancel_transaction: Option<serde_json::Value>,
    /// IDs of all transactions related to this request.
    #[serde(rename = "relatedTransactionIDs")]
    pub related_transaction_ids: Option<Vec<TransactionID>>,
    /// ID of the most recent transaction on the account after this request.
    #[serde(rename = "lastTransactionID")]
    pub last_transaction_id: Option<TransactionID>,
}

/// Request body for
/// `PUT /v3/accounts/{accountID}/orders/{orderSpecifier}/clientExtensions`.
///
/// At least one of `client_extensions` or `trade_client_extensions` must be set.
#[derive(Debug, Serialize)]
pub struct UpdateClientExtensionsBody {
    /// New client extensions for the order itself. Omitted if `None`.
    #[serde(rename = "clientExtensions", skip_serializing_if = "Option::is_none")]
    pub client_extensions: Option<ClientExtensions>,
    /// New client extensions for the trade that would result from filling this
    /// order. Omitted if `None`.
    #[serde(
        rename = "tradeClientExtensions",
        skip_serializing_if = "Option::is_none"
    )]
    pub trade_client_extensions: Option<ClientExtensions>,
}

/// Response body for a successful client-extensions update (HTTP 200).
#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateClientExtensionsResponse {
    /// The transaction recording the modification.
    #[serde(rename = "orderClientExtensionsModifyTransaction")]
    pub order_client_extensions_modify_transaction: Option<serde_json::Value>,
    /// IDs of all transactions related to this request.
    #[serde(rename = "relatedTransactionIDs")]
    pub related_transaction_ids: Option<Vec<TransactionID>>,
    /// ID of the most recent transaction on the account after this request.
    #[serde(rename = "lastTransactionID")]
    pub last_transaction_id: Option<TransactionID>,
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
pub struct DynamicOrderState {
    /// The order's unique identifier.
    pub id: OrderID,
    /// Current absolute stop price for a trailing stop-loss order,
    /// recalculated as the market moves.
    #[serde(rename = "trailingStopValue")]
    pub trailing_stop_value: Option<PriceValue>,
    /// The distance between the current market price and the order's trigger
    /// price. Positive = the order has not yet triggered.
    #[serde(rename = "triggerDistance")]
    pub trigger_distance: Option<PriceValue>,
    /// `true` if `trigger_distance` is an exact value; `false` if it is
    /// approximate (e.g. when the market is closed).
    #[serde(rename = "isTriggerDistanceExact")]
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

/// JSON request body for `POST /v3/accounts/{accountID}/orders`.
///
/// Prefer constructing this via [`OrderRequest`] variants and
/// [`OrderService::create`] directly.
#[derive(Debug, Serialize, Deserialize)]
pub struct CreateOrderRequest {
    order: OrderRequest,
}

impl CreateOrderRequest {
    /// Wraps an [`OrderRequest`] in the JSON envelope expected by the API.
    pub fn new(order: OrderRequest) -> CreateOrderRequest {
        CreateOrderRequest { order }
    }
}

/// Builder for a `GET /v3/accounts/{accountID}/orders` request.
///
/// Construct via [`ListOrdersRequest::new`], configure with the builder
/// methods, then pass to [`OrderService::list`].
pub struct ListOrdersRequest {
    /// Filter by specific order IDs.
    ids: Vec<OrderID>,
    /// Filter by order state.
    state: Option<OrderStateFilter>,
    /// Filter by instrument name.
    instrument: Option<InstrumentName>,
    /// Maximum number of orders to return.
    count: Option<u16>,
    /// Return only orders with IDs strictly less than this value (pagination).
    before_id: Option<OrderID>,
}

impl ListOrdersRequest {
    /// Creates a new, unconfigured request (returns all orders by default).
    pub fn new() -> Self {
        ListOrdersRequest {
            ids: Vec::new(),
            state: None,
            instrument: None,
            count: None,
            before_id: None,
        }
    }

    /// Adds a single order ID to the `ids` filter. Call multiple times to
    /// filter on several IDs.
    pub fn ids(mut self, id: OrderID) -> Self {
        self.ids.push(id);
        self
    }

    /// Filters results to orders in the given `state`.
    pub fn state(mut self, state: OrderStateFilter) -> Self {
        self.state = Some(state);
        self
    }

    /// Filters results to orders for the given `instrument`.
    pub fn instrument(mut self, instrument: InstrumentName) -> Self {
        self.instrument = Some(instrument);
        self
    }

    /// Sets the maximum number of orders returned (server-side limit applies).
    pub fn count(mut self, count: u16) -> Self {
        self.count = Some(count);
        self
    }

    /// Returns only orders whose IDs are strictly less than `before_id`,
    /// enabling backward pagination through results.
    pub fn before_id(mut self, before_id: OrderID) -> Self {
        self.before_id = Some(before_id);
        self
    }

    /// Appends all configured query parameters to `url`.
    ///
    /// Called internally by [`OrderService::list`] before dispatching the
    /// HTTP request.
    pub fn set_params(&self, url: &mut Url) {
        self.ids.is_empty().not().then(|| {
            url.query_pairs_mut().append_pair(
                "ids",
                self.ids
                    .iter()
                    .map(|id| id.to_string())
                    .collect::<Vec<String>>()
                    .join(",")
                    .as_str(),
            );
        });
        self.state.is_some().then(|| {
            url.query_pairs_mut()
                .append_pair("state", self.state.as_ref().unwrap().to_string().as_str());
        });
        self.instrument.is_some().then(|| {
            url.query_pairs_mut()
                .append_pair("instrument", self.instrument.as_ref().unwrap().as_str());
        });
        self.count.is_some().then(|| {
            url.query_pairs_mut()
                .append_pair("count", &self.count.unwrap().to_string().as_str());
        });
        self.before_id.as_ref().map(|id| {
            url.query_pairs_mut().append_pair("beforeID", id.as_str());
        });
    }
}

/// Response body for `GET /v3/accounts/{accountID}/orders` and
/// `GET /v3/accounts/{accountID}/pendingOrders`.
#[derive(Debug, Serialize, Deserialize)]
pub struct ListOrdersResponse {
    /// The list of orders matching the request filters.
    pub orders: Vec<Order>,
    /// ID of the most recent transaction on the account.
    #[serde(rename = "lastTransactionID")]
    pub last_transaction_id: TransactionID,
}

/// Response body for `GET /v3/accounts/{accountID}/orders/{orderSpecifier}`.
#[derive(Debug, Serialize, Deserialize)]
pub struct GetOrderDetailsResponse {
    /// The requested order.
    pub order: Order,
    /// ID of the most recent transaction on the account.
    #[serde(rename = "lastTransactionID")]
    pub last_transaction_id: TransactionID,
}

/// Provides access to the OANDA Order endpoints (`/v3/accounts/{id}/orders/...`).
///
/// Obtain an instance via [`Client::order`](crate::client::Client::order).
pub struct OrderService<'a> {
    client: &'a Client,
}

impl<'a> OrderService<'a> {
    /// Creates a new `OrderService` bound to the given client.
    pub fn new(client: &'a Client) -> Self {
        OrderService { client }
    }

    /// Creates a new order on the account.
    ///
    /// Calls `POST /v3/accounts/{accountID}/orders`. On success (HTTP 201)
    /// returns [`CreateOrderResponse`].
    ///
    /// # Panics
    ///
    /// Panics if no `account_id` has been set on the client.
    pub async fn create(&self, order: OrderRequest) -> Result<CreateOrderResponse, APIError> {
        let url = self
            .client
            .base_url
            .join(
                format!(
                    "/v3/accounts/{}/orders",
                    self.client.account_id.as_ref().expect("Missing account_id")
                )
                .as_str(),
            )
            .unwrap();
        let body = CreateOrderBody { order };
        let http_resp = self.client.http_client.post(url).json(&body).send().await?;
        match http_resp.status() {
            StatusCode::CREATED => {
                let resp = http_resp.json::<CreateOrderResponse>().await?;
                Ok(resp)
            }
            status => Err(APIError::ApiErrorResponse {
                status,
                message: http_resp.text().await?,
            }),
        }
    }

    /// Lists orders on the account, optionally filtered by the parameters in `req`.
    ///
    /// Calls `GET /v3/accounts/{accountID}/orders`.
    ///
    /// # Panics
    ///
    /// Panics if no `account_id` has been set on the client.
    pub async fn list(&self, req: ListOrdersRequest) -> Result<ListOrdersResponse, APIError> {
        let mut url = self
            .client
            .base_url
            .join(
                format!(
                    "/v3/accounts/{}/orders",
                    self.client
                        .account_id
                        .as_ref()
                        .expect("Missing account_id in client")
                )
                .as_str(),
            )
            .unwrap();
        req.set_params(&mut url);
        let http_req = Request::new(reqwest::Method::GET, url);
        let http_resp = self.client.http_client.execute(http_req).await?;
        match http_resp.status() {
            StatusCode::OK => {
                let resp = http_resp.json::<ListOrdersResponse>().await?;
                Ok(resp)
            }
            status => Err(APIError::ApiErrorResponse {
                status,
                message: http_resp.text().await?,
            }),
        }
    }

    /// Returns all pending (not yet filled or cancelled) orders on the account.
    ///
    /// Calls `GET /v3/accounts/{accountID}/pendingOrders`.
    ///
    /// # Panics
    ///
    /// Panics if no `account_id` has been set on the client.
    pub async fn list_pending(&self) -> Result<ListOrdersResponse, APIError> {
        let url = self
            .client
            .base_url
            .join(
                format!(
                    "/v3/accounts/{}/pendingOrders",
                    self.client.account_id.as_ref().expect("Missing account_id")
                )
                .as_str(),
            )
            .unwrap();
        let http_req = Request::new(reqwest::Method::GET, url);
        let http_resp = self.client.http_client.execute(http_req).await?;
        match http_resp.status() {
            StatusCode::OK => {
                let resp = http_resp.json::<ListOrdersResponse>().await?;
                Ok(resp)
            }
            status => Err(APIError::ApiErrorResponse {
                status,
                message: http_resp.text().await?,
            }),
        }
    }

    /// Returns the details of a single order identified by `specifier`.
    ///
    /// Calls `GET /v3/accounts/{accountID}/orders/{orderSpecifier}`.
    ///
    /// # Panics
    ///
    /// Panics if no `account_id` has been set on the client.
    pub async fn get_details(
        &self,
        specifier: OrderSpecifier,
    ) -> Result<GetOrderDetailsResponse, APIError> {
        let url = self
            .client
            .base_url
            .join(
                format!(
                    "/v3/accounts/{}/orders/{}",
                    self.client.account_id.as_ref().expect("Missing account_id"),
                    specifier
                )
                .as_str(),
            )
            .unwrap();
        let http_req = Request::new(reqwest::Method::GET, url);
        let http_resp = self.client.http_client.execute(http_req).await?;
        match http_resp.status() {
            StatusCode::OK => {
                let resp = http_resp.json::<GetOrderDetailsResponse>().await?;
                Ok(resp)
            }
            status => Err(APIError::ApiErrorResponse {
                status,
                message: http_resp.text().await?,
            }),
        }
    }

    /// Replaces the order identified by `specifier` with a new `order`.
    ///
    /// Calls `PUT /v3/accounts/{accountID}/orders/{orderSpecifier}`.
    /// The original order is cancelled and a new one is created atomically.
    ///
    /// # Panics
    ///
    /// Panics if no `account_id` has been set on the client.
    pub async fn replace(
        &self,
        specifier: OrderSpecifier,
        req: OrderRequest,
    ) -> Result<ReplaceOrderResponse, APIError> {
        let url = self
            .client
            .base_url
            .join(
                format!(
                    "/v3/accounts/{}/orders/{}",
                    self.client.account_id.as_ref().expect("Missing account_id"),
                    specifier
                )
                .as_str(),
            )
            .unwrap();
        let body = CreateOrderBody { order: req };
        let http_resp = self.client.http_client.put(url).json(&body).send().await?;
        match http_resp.status() {
            StatusCode::CREATED => {
                let resp = http_resp.json::<ReplaceOrderResponse>().await?;
                Ok(resp)
            }
            status => Err(APIError::ApiErrorResponse {
                status,
                message: http_resp.text().await?,
            }),
        }
    }

    /// Cancels the pending order identified by `specifier`.
    ///
    /// Calls `PUT /v3/accounts/{accountID}/orders/{orderSpecifier}/cancel`.
    ///
    /// # Panics
    ///
    /// Panics if no `account_id` has been set on the client.
    pub async fn cancel(&self, specifier: OrderSpecifier) -> Result<CancelOrderResponse, APIError> {
        let url = self
            .client
            .base_url
            .join(
                format!(
                    "/v3/accounts/{}/orders/{}/cancel",
                    self.client.account_id.as_ref().expect("Missing account_id"),
                    specifier
                )
                .as_str(),
            )
            .unwrap();
        let http_resp = self.client.http_client.put(url).send().await?;
        match http_resp.status() {
            StatusCode::OK => {
                let resp = http_resp.json::<CancelOrderResponse>().await?;
                Ok(resp)
            }
            status => Err(APIError::ApiErrorResponse {
                status,
                message: http_resp.text().await?,
            }),
        }
    }

    /// Updates the client extensions on the order and/or its associated trade.
    ///
    /// Calls `PUT /v3/accounts/{accountID}/orders/{orderSpecifier}/clientExtensions`.
    ///
    /// # Panics
    ///
    /// Panics if no `account_id` has been set on the client.
    pub async fn update_client_extensions(
        &self,
        specifier: OrderSpecifier,
        body: UpdateClientExtensionsBody,
    ) -> Result<UpdateClientExtensionsResponse, APIError> {
        let url = self
            .client
            .base_url
            .join(
                format!(
                    "/v3/accounts/{}/orders/{}/clientExtensions",
                    self.client.account_id.as_ref().expect("Missing account_id"),
                    specifier
                )
                .as_str(),
            )
            .unwrap();
        let http_resp = self.client.http_client.put(url).json(&body).send().await?;
        match http_resp.status() {
            StatusCode::OK => {
                let resp = http_resp.json::<UpdateClientExtensionsResponse>().await?;
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
    use crate::order::{LimitOrderRequest, ListOrdersRequest, OrderRequest};

    #[tokio::test]
    async fn test_limit_order() {
        let client = setup_test_client();
        // Create limit order
        let req = LimitOrderRequest::new(
            "USD_JPY".to_string(),
            "10000".to_string(),
            "100.00".to_string(),
        );
        let resp = client
            .order()
            .create(OrderRequest::Limit(req))
            .await
            .unwrap();
        println!("{:#?}", resp);
        let order_id = resp.order_create_transaction.as_ref().unwrap().get_id();

        let req = ListOrdersRequest::new().instrument("USD_JPY".to_string());
        let resp = client.order().list(req).await.unwrap();
        println!("{:#?}", resp);

        let req = LimitOrderRequest::new(
            "USD_JPY".to_string(),
            "10000".to_string(),
            "101.00".to_string(),
        );
        let resp = client
            .order()
            .replace(order_id, OrderRequest::Limit(req))
            .await
            .unwrap();
        println!("{:#?}", resp);
        let order_id = resp.order_create_transaction.as_ref().unwrap().get_id();

        let resp = client.order().list_pending().await.unwrap();
        println!("{:#?}", resp);

        // Get details
        let resp = client.order().get_details(order_id.clone()).await.unwrap();
        println!("{:#?}", resp);

        // Cancel
        let resp = client.order().cancel(order_id).await.unwrap();
        println!("{:#?}", resp);
    }
}

#[cfg(test)]
pub(crate) async fn create_market_order(client: &Client) -> TransactionID {
    let req = MarketOrderRequest::new("USD_JPY".to_string(), "10000".to_string());
    let resp = client
        .order()
        .create(OrderRequest::Market(req))
        .await
        .unwrap();
    println!("{:#?}", resp);
    resp.order_create_transaction.unwrap().get_id()
}
