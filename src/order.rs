use crate::client::Client;
use crate::errors::APIError;
use crate::instrument::InstrumentName;
use crate::pricing::PriceValue;
use crate::primitives::DecimalNumber;
use crate::transaction::{
    ClientExtensions, ClientID, GuaranteedStopLossDetails, StopLossDetails, TakeProfitDetails,
    TradeID, TrailingStopLossDetails, TransactionID,
};
use chrono::{DateTime, Utc};
use reqwest::{Request, StatusCode};
use serde::{Deserialize, Serialize};
use std::ops::Not;
use strum_macros::Display;
use url::Url;

pub type OrderID = String;

// ---------------------------------------------------------------------------
// Order enum (tagged union)
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Order {
    #[serde(rename = "MARKET")]
    MarketOrder(MarketOrder),
    #[serde(rename = "LIMIT")]
    LimitOrder(LimitOrder),
    #[serde(rename = "STOP")]
    StopOrder(StopOrder),
    #[serde(rename = "MARKET_IF_TOUCHED")]
    MarketIfTouchedOrder(MarketIfTouchedOrder),
    #[serde(rename = "TAKE_PROFIT")]
    TakeProfitOrder(TakeProfitOrder),
    #[serde(rename = "STOP_LOSS")]
    StopLossOrder(StopLossOrder),
    #[serde(rename = "GUARANTEED_STOP_LOSS")]
    GuaranteedStopLossOrder(GuaranteedStopLossOrder),
    #[serde(rename = "TRAILING_STOP_LOSS")]
    TrailingStopLossOrder(TrailingStopLossOrder),
    #[serde(rename = "FIXED_PRICE")]
    FixedPriceOrder(FixedPriceOrder),
}

// ---------------------------------------------------------------------------
// Order Structs
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize, Serialize)]
pub struct MarketOrder {
    pub id: OrderID,
    #[serde(rename = "createTime")]
    pub create_time: DateTime<Utc>,
    pub state: OrderState,
    #[serde(rename = "clientExtensions", skip_serializing_if = "Option::is_none")]
    pub client_extensions: Option<ClientExtensions>,
    pub instrument: Option<InstrumentName>,
    pub units: Option<DecimalNumber>,
    #[serde(rename = "timeInForce")]
    pub time_in_force: Option<TimeInForce>,
    #[serde(rename = "priceBound")]
    pub price_bound: Option<PriceValue>,
    #[serde(rename = "positionFill")]
    pub position_fill: Option<OrderPositionFill>,
    #[serde(rename = "takeProfitOnFill")]
    pub take_profit_on_fill: Option<TakeProfitDetails>,
    #[serde(rename = "stopLossOnFill")]
    pub stop_loss_on_fill: Option<StopLossDetails>,
    #[serde(rename = "trailingStopLossOnFill")]
    pub trailing_stop_loss_on_fill: Option<TrailingStopLossDetails>,
    #[serde(rename = "guaranteedStopLossOnFill")]
    pub guaranteed_stop_loss_on_fill: Option<GuaranteedStopLossDetails>,
    #[serde(rename = "tradeClientExtensions")]
    pub trade_client_extensions: Option<ClientExtensions>,
    #[serde(rename = "fillingTransactionID")]
    pub filling_transaction_id: Option<TransactionID>,
    #[serde(rename = "filledTime")]
    pub filled_time: Option<DateTime<Utc>>,
    #[serde(rename = "tradeOpenedID")]
    pub trade_opened_id: Option<TradeID>,
    #[serde(rename = "tradeReducedID")]
    pub trade_reduced_id: Option<TradeID>,
    #[serde(rename = "tradeClosedIDs")]
    pub trade_closed_ids: Option<Vec<TradeID>>,
    #[serde(rename = "cancellingTransactionID")]
    pub cancelling_transaction_id: Option<TransactionID>,
    #[serde(rename = "cancelledTime")]
    pub cancelled_time: Option<DateTime<Utc>>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct LimitOrder {
    pub id: OrderID,
    #[serde(rename = "createTime")]
    pub create_time: DateTime<Utc>,
    pub state: OrderState,
    #[serde(rename = "clientExtensions")]
    pub client_extensions: Option<ClientExtensions>,
    pub instrument: Option<InstrumentName>,
    pub units: Option<DecimalNumber>,
    pub price: Option<PriceValue>,
    #[serde(rename = "timeInForce")]
    pub time_in_force: Option<TimeInForce>,
    #[serde(rename = "gtdTime")]
    pub gtd_time: Option<DateTime<Utc>>,
    #[serde(rename = "positionFill")]
    pub position_fill: Option<OrderPositionFill>,
    #[serde(rename = "triggerCondition")]
    pub trigger_condition: Option<OrderTriggerCondition>,
    #[serde(rename = "takeProfitOnFill")]
    pub take_profit_on_fill: Option<TakeProfitDetails>,
    #[serde(rename = "stopLossOnFill")]
    pub stop_loss_on_fill: Option<StopLossDetails>,
    #[serde(rename = "trailingStopLossOnFill")]
    pub trailing_stop_loss_on_fill: Option<TrailingStopLossDetails>,
    #[serde(rename = "guaranteedStopLossOnFill")]
    pub guaranteed_stop_loss_on_fill: Option<GuaranteedStopLossDetails>,
    #[serde(rename = "tradeClientExtensions")]
    pub trade_client_extensions: Option<ClientExtensions>,
    #[serde(rename = "fillingTransactionID")]
    pub filling_transaction_id: Option<TransactionID>,
    #[serde(rename = "filledTime")]
    pub filled_time: Option<DateTime<Utc>>,
    #[serde(rename = "tradeOpenedID")]
    pub trade_opened_id: Option<TradeID>,
    #[serde(rename = "tradeReducedID")]
    pub trade_reduced_id: Option<TradeID>,
    #[serde(rename = "tradeClosedIDs")]
    pub trade_closed_ids: Option<Vec<TradeID>>,
    #[serde(rename = "cancellingTransactionID")]
    pub cancelling_transaction_id: Option<TransactionID>,
    #[serde(rename = "cancelledTime")]
    pub cancelled_time: Option<DateTime<Utc>>,
    #[serde(rename = "replacesOrderID")]
    pub replaces_order_id: Option<OrderID>,
    #[serde(rename = "replacedByOrderID")]
    pub replaced_by_order_id: Option<OrderID>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct StopOrder {
    pub id: OrderID,
    #[serde(rename = "createTime")]
    pub create_time: DateTime<Utc>,
    pub state: OrderState,
    #[serde(rename = "clientExtensions")]
    pub client_extensions: Option<ClientExtensions>,
    pub instrument: Option<InstrumentName>,
    pub units: Option<DecimalNumber>,
    pub price: Option<PriceValue>,
    #[serde(rename = "priceBound")]
    pub price_bound: Option<PriceValue>,
    #[serde(rename = "timeInForce")]
    pub time_in_force: Option<TimeInForce>,
    #[serde(rename = "gtdTime")]
    pub gtd_time: Option<DateTime<Utc>>,
    #[serde(rename = "positionFill")]
    pub position_fill: Option<OrderPositionFill>,
    #[serde(rename = "triggerCondition")]
    pub trigger_condition: Option<OrderTriggerCondition>,
    #[serde(rename = "takeProfitOnFill")]
    pub take_profit_on_fill: Option<TakeProfitDetails>,
    #[serde(rename = "stopLossOnFill")]
    pub stop_loss_on_fill: Option<StopLossDetails>,
    #[serde(rename = "trailingStopLossOnFill")]
    pub trailing_stop_loss_on_fill: Option<TrailingStopLossDetails>,
    #[serde(rename = "guaranteedStopLossOnFill")]
    pub guaranteed_stop_loss_on_fill: Option<GuaranteedStopLossDetails>,
    #[serde(rename = "tradeClientExtensions")]
    pub trade_client_extensions: Option<ClientExtensions>,
    #[serde(rename = "fillingTransactionID")]
    pub filling_transaction_id: Option<TransactionID>,
    #[serde(rename = "filledTime")]
    pub filled_time: Option<DateTime<Utc>>,
    #[serde(rename = "tradeOpenedID")]
    pub trade_opened_id: Option<TradeID>,
    #[serde(rename = "tradeReducedID")]
    pub trade_reduced_id: Option<TradeID>,
    #[serde(rename = "tradeClosedIDs")]
    pub trade_closed_ids: Option<Vec<TradeID>>,
    #[serde(rename = "cancellingTransactionID")]
    pub cancelling_transaction_id: Option<TransactionID>,
    #[serde(rename = "cancelledTime")]
    pub cancelled_time: Option<DateTime<Utc>>,
    #[serde(rename = "replacesOrderID")]
    pub replaces_order_id: Option<OrderID>,
    #[serde(rename = "replacedByOrderID")]
    pub replaced_by_order_id: Option<OrderID>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct MarketIfTouchedOrder {
    pub id: OrderID,
    #[serde(rename = "createTime")]
    pub create_time: DateTime<Utc>,
    pub state: OrderState,
    #[serde(rename = "clientExtensions")]
    pub client_extensions: Option<ClientExtensions>,
    pub instrument: Option<InstrumentName>,
    pub units: Option<DecimalNumber>,
    pub price: Option<PriceValue>,
    #[serde(rename = "priceBound")]
    pub price_bound: Option<PriceValue>,
    #[serde(rename = "timeInForce")]
    pub time_in_force: Option<TimeInForce>,
    #[serde(rename = "gtdTime")]
    pub gtd_time: Option<DateTime<Utc>>,
    #[serde(rename = "positionFill")]
    pub position_fill: Option<OrderPositionFill>,
    #[serde(rename = "triggerCondition")]
    pub trigger_condition: Option<OrderTriggerCondition>,
    #[serde(rename = "initialMarketPrice")]
    pub initial_market_price: Option<PriceValue>,
    #[serde(rename = "takeProfitOnFill")]
    pub take_profit_on_fill: Option<TakeProfitDetails>,
    #[serde(rename = "stopLossOnFill")]
    pub stop_loss_on_fill: Option<StopLossDetails>,
    #[serde(rename = "trailingStopLossOnFill")]
    pub trailing_stop_loss_on_fill: Option<TrailingStopLossDetails>,
    #[serde(rename = "guaranteedStopLossOnFill")]
    pub guaranteed_stop_loss_on_fill: Option<GuaranteedStopLossDetails>,
    #[serde(rename = "tradeClientExtensions")]
    pub trade_client_extensions: Option<ClientExtensions>,
    #[serde(rename = "fillingTransactionID")]
    pub filling_transaction_id: Option<TransactionID>,
    #[serde(rename = "filledTime")]
    pub filled_time: Option<DateTime<Utc>>,
    #[serde(rename = "tradeOpenedID")]
    pub trade_opened_id: Option<TradeID>,
    #[serde(rename = "tradeReducedID")]
    pub trade_reduced_id: Option<TradeID>,
    #[serde(rename = "tradeClosedIDs")]
    pub trade_closed_ids: Option<Vec<TradeID>>,
    #[serde(rename = "cancellingTransactionID")]
    pub cancelling_transaction_id: Option<TransactionID>,
    #[serde(rename = "cancelledTime")]
    pub cancelled_time: Option<DateTime<Utc>>,
    #[serde(rename = "replacesOrderID")]
    pub replaces_order_id: Option<OrderID>,
    #[serde(rename = "replacedByOrderID")]
    pub replaced_by_order_id: Option<OrderID>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TakeProfitOrder {
    pub id: OrderID,
    #[serde(rename = "createTime")]
    pub create_time: DateTime<Utc>,
    pub state: OrderState,
    #[serde(rename = "clientExtensions")]
    pub client_extensions: Option<ClientExtensions>,
    #[serde(rename = "tradeID")]
    pub trade_id: TradeID,
    #[serde(rename = "clientTradeID")]
    pub client_trade_id: Option<ClientID>,
    pub price: PriceValue,
    #[serde(rename = "timeInForce")]
    pub time_in_force: Option<TimeInForce>,
    #[serde(rename = "gtdTime")]
    pub gtd_time: Option<DateTime<Utc>>,
    #[serde(rename = "triggerCondition")]
    pub trigger_condition: Option<OrderTriggerCondition>,
    #[serde(rename = "fillingTransactionID")]
    pub filling_transaction_id: Option<TransactionID>,
    #[serde(rename = "filledTime")]
    pub filled_time: Option<DateTime<Utc>>,
    #[serde(rename = "tradeOpenedID")]
    pub trade_opened_id: Option<TradeID>,
    #[serde(rename = "tradeReducedID")]
    pub trade_reduced_id: Option<TradeID>,
    #[serde(rename = "tradeClosedIDs")]
    pub trade_closed_ids: Option<Vec<TradeID>>,
    #[serde(rename = "cancellingTransactionID")]
    pub cancelling_transaction_id: Option<TransactionID>,
    #[serde(rename = "cancelledTime")]
    pub cancelled_time: Option<DateTime<Utc>>,
    #[serde(rename = "replacesOrderID")]
    pub replaces_order_id: Option<OrderID>,
    #[serde(rename = "replacedByOrderID")]
    pub replaced_by_order_id: Option<OrderID>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct StopLossOrder {
    pub id: OrderID,
    #[serde(rename = "createTime")]
    pub create_time: DateTime<Utc>,
    pub state: OrderState,
    #[serde(rename = "clientExtensions")]
    pub client_extensions: Option<ClientExtensions>,
    #[serde(rename = "tradeID")]
    pub trade_id: TradeID,
    #[serde(rename = "clientTradeID")]
    pub client_trade_id: Option<ClientID>,
    pub price: Option<PriceValue>,
    pub distance: Option<DecimalNumber>,
    #[serde(rename = "timeInForce")]
    pub time_in_force: Option<TimeInForce>,
    #[serde(rename = "gtdTime")]
    pub gtd_time: Option<DateTime<Utc>>,
    #[serde(rename = "triggerCondition")]
    pub trigger_condition: Option<OrderTriggerCondition>,
    pub guaranteed: Option<bool>,
    #[serde(rename = "guaranteedExecutionPremium")]
    pub guaranteed_execution_premium: Option<DecimalNumber>,
    #[serde(rename = "fillingTransactionID")]
    pub filling_transaction_id: Option<TransactionID>,
    #[serde(rename = "filledTime")]
    pub filled_time: Option<DateTime<Utc>>,
    #[serde(rename = "tradeOpenedID")]
    pub trade_opened_id: Option<TradeID>,
    #[serde(rename = "tradeReducedID")]
    pub trade_reduced_id: Option<TradeID>,
    #[serde(rename = "tradeClosedIDs")]
    pub trade_closed_ids: Option<Vec<TradeID>>,
    #[serde(rename = "cancellingTransactionID")]
    pub cancelling_transaction_id: Option<TransactionID>,
    #[serde(rename = "cancelledTime")]
    pub cancelled_time: Option<DateTime<Utc>>,
    #[serde(rename = "replacesOrderID")]
    pub replaces_order_id: Option<OrderID>,
    #[serde(rename = "replacedByOrderID")]
    pub replaced_by_order_id: Option<OrderID>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct GuaranteedStopLossOrder {
    pub id: OrderID,
    #[serde(rename = "createTime")]
    pub create_time: DateTime<Utc>,
    pub state: OrderState,
    #[serde(rename = "clientExtensions")]
    pub client_extensions: Option<ClientExtensions>,
    #[serde(rename = "tradeID")]
    pub trade_id: TradeID,
    #[serde(rename = "clientTradeID")]
    pub client_trade_id: Option<ClientID>,
    pub price: PriceValue,
    pub distance: Option<DecimalNumber>,
    #[serde(rename = "timeInForce")]
    pub time_in_force: Option<TimeInForce>,
    #[serde(rename = "gtdTime")]
    pub gtd_time: Option<DateTime<Utc>>,
    #[serde(rename = "triggerCondition")]
    pub trigger_condition: Option<OrderTriggerCondition>,
    #[serde(rename = "guaranteedExecutionPremium")]
    pub guaranteed_execution_premium: Option<DecimalNumber>,
    #[serde(rename = "fillingTransactionID")]
    pub filling_transaction_id: Option<TransactionID>,
    #[serde(rename = "filledTime")]
    pub filled_time: Option<DateTime<Utc>>,
    #[serde(rename = "tradeOpenedID")]
    pub trade_opened_id: Option<TradeID>,
    #[serde(rename = "tradeReducedID")]
    pub trade_reduced_id: Option<TradeID>,
    #[serde(rename = "tradeClosedIDs")]
    pub trade_closed_ids: Option<Vec<TradeID>>,
    #[serde(rename = "cancellingTransactionID")]
    pub cancelling_transaction_id: Option<TransactionID>,
    #[serde(rename = "cancelledTime")]
    pub cancelled_time: Option<DateTime<Utc>>,
    #[serde(rename = "replacesOrderID")]
    pub replaces_order_id: Option<OrderID>,
    #[serde(rename = "replacedByOrderID")]
    pub replaced_by_order_id: Option<OrderID>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TrailingStopLossOrder {
    pub id: OrderID,
    #[serde(rename = "createTime")]
    pub create_time: DateTime<Utc>,
    pub state: OrderState,
    #[serde(rename = "clientExtensions")]
    pub client_extensions: Option<ClientExtensions>,
    #[serde(rename = "tradeID")]
    pub trade_id: TradeID,
    #[serde(rename = "clientTradeID")]
    pub client_trade_id: Option<ClientID>,
    pub distance: DecimalNumber,
    #[serde(rename = "timeInForce")]
    pub time_in_force: Option<TimeInForce>,
    #[serde(rename = "gtdTime")]
    pub gtd_time: Option<DateTime<Utc>>,
    #[serde(rename = "triggerCondition")]
    pub trigger_condition: Option<OrderTriggerCondition>,
    #[serde(rename = "trailingStopValue")]
    pub trailing_stop_value: Option<PriceValue>,
    #[serde(rename = "fillingTransactionID")]
    pub filling_transaction_id: Option<TransactionID>,
    #[serde(rename = "filledTime")]
    pub filled_time: Option<DateTime<Utc>>,
    #[serde(rename = "tradeOpenedID")]
    pub trade_opened_id: Option<TradeID>,
    #[serde(rename = "tradeReducedID")]
    pub trade_reduced_id: Option<TradeID>,
    #[serde(rename = "tradeClosedIDs")]
    pub trade_closed_ids: Option<Vec<TradeID>>,
    #[serde(rename = "cancellingTransactionID")]
    pub cancelling_transaction_id: Option<TransactionID>,
    #[serde(rename = "cancelledTime")]
    pub cancelled_time: Option<DateTime<Utc>>,
    #[serde(rename = "replacesOrderID")]
    pub replaces_order_id: Option<OrderID>,
    #[serde(rename = "replacedByOrderID")]
    pub replaced_by_order_id: Option<OrderID>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct FixedPriceOrder {
    pub id: OrderID,
    #[serde(rename = "createTime")]
    pub create_time: DateTime<Utc>,
    pub state: OrderState,
    #[serde(rename = "clientExtensions")]
    pub client_extensions: Option<ClientExtensions>,
    pub instrument: Option<InstrumentName>,
    pub units: Option<DecimalNumber>,
    pub price: Option<PriceValue>,
    #[serde(rename = "positionFill")]
    pub position_fill: Option<OrderPositionFill>,
    #[serde(rename = "tradeState")]
    pub trade_state: Option<String>,
    #[serde(rename = "takeProfitOnFill")]
    pub take_profit_on_fill: Option<TakeProfitDetails>,
    #[serde(rename = "stopLossOnFill")]
    pub stop_loss_on_fill: Option<StopLossDetails>,
    #[serde(rename = "trailingStopLossOnFill")]
    pub trailing_stop_loss_on_fill: Option<TrailingStopLossDetails>,
    #[serde(rename = "guaranteedStopLossOnFill")]
    pub guaranteed_stop_loss_on_fill: Option<GuaranteedStopLossDetails>,
    #[serde(rename = "tradeClientExtensions")]
    pub trade_client_extensions: Option<ClientExtensions>,
    #[serde(rename = "fillingTransactionID")]
    pub filling_transaction_id: Option<TransactionID>,
    #[serde(rename = "filledTime")]
    pub filled_time: Option<DateTime<Utc>>,
    #[serde(rename = "tradeOpenedID")]
    pub trade_opened_id: Option<TradeID>,
    #[serde(rename = "tradeReducedID")]
    pub trade_reduced_id: Option<TradeID>,
    #[serde(rename = "tradeClosedIDs")]
    pub trade_closed_ids: Option<Vec<TradeID>>,
    #[serde(rename = "cancellingTransactionID")]
    pub cancelling_transaction_id: Option<TransactionID>,
    #[serde(rename = "cancelledTime")]
    pub cancelled_time: Option<DateTime<Utc>>,
}

// ---------------------------------------------------------------------------
// Order Request Types (for creating/replacing orders)
// ---------------------------------------------------------------------------

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

#[derive(Debug, Serialize, Deserialize)]
pub struct MarketOrderRequest {
    pub instrument: InstrumentName,
    pub units: DecimalNumber,
    #[serde(rename = "timeInForce")]
    pub time_in_force: Option<TimeInForce>,
    #[serde(rename = "priceBound")]
    pub price_bound: Option<PriceValue>,
    #[serde(rename = "positionFill")]
    pub position_fill: Option<OrderPositionFill>,
    #[serde(rename = "clientExtensions", skip_serializing_if = "Option::is_none")]
    pub client_extensions: Option<ClientExtensions>,
    #[serde(rename = "takeProfitOnFill", skip_serializing_if = "Option::is_none")]
    pub take_profit_on_fill: Option<TakeProfitDetails>,
    #[serde(rename = "stopLossOnFill", skip_serializing_if = "Option::is_none")]
    pub stop_loss_on_fill: Option<StopLossDetails>,
    #[serde(rename = "guaranteedStopLossOnFill", skip_serializing_if = "Option::is_none")]
    pub guaranteed_stop_loss_on_fill: Option<GuaranteedStopLossDetails>,
    #[serde(rename = "trailingStopLossOnFill", skip_serializing_if = "Option::is_none")]
    pub trailing_stop_loss_on_fill: Option<TrailingStopLossDetails>,
    #[serde(rename = "tradeClientExtensions", skip_serializing_if = "Option::is_none")]
    pub trade_client_extensions: Option<ClientExtensions>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LimitOrderRequest {
    pub instrument: InstrumentName,
    pub units: DecimalNumber,
    pub price: PriceValue,
    #[serde(rename = "timeInForce")]
    pub time_in_force: Option<TimeInForce>,
    #[serde(rename = "gtdTime", skip_serializing_if = "Option::is_none")]
    pub gtd_time: Option<DateTime<Utc>>,
    #[serde(rename = "positionFill")]
    pub position_fill: Option<OrderPositionFill>,
    #[serde(rename = "triggerCondition")]
    pub trigger_condition: Option<OrderTriggerCondition>,
    #[serde(rename = "clientExtensions", skip_serializing_if = "Option::is_none")]
    pub client_extensions: Option<ClientExtensions>,
    #[serde(rename = "takeProfitOnFill", skip_serializing_if = "Option::is_none")]
    pub take_profit_on_fill: Option<TakeProfitDetails>,
    #[serde(rename = "stopLossOnFill", skip_serializing_if = "Option::is_none")]
    pub stop_loss_on_fill: Option<StopLossDetails>,
    #[serde(rename = "guaranteedStopLossOnFill", skip_serializing_if = "Option::is_none")]
    pub guaranteed_stop_loss_on_fill: Option<GuaranteedStopLossDetails>,
    #[serde(rename = "trailingStopLossOnFill", skip_serializing_if = "Option::is_none")]
    pub trailing_stop_loss_on_fill: Option<TrailingStopLossDetails>,
    #[serde(rename = "tradeClientExtensions", skip_serializing_if = "Option::is_none")]
    pub trade_client_extensions: Option<ClientExtensions>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StopOrderRequest {
    pub instrument: InstrumentName,
    pub units: DecimalNumber,
    pub price: PriceValue,
    #[serde(rename = "priceBound", skip_serializing_if = "Option::is_none")]
    pub price_bound: Option<PriceValue>,
    #[serde(rename = "timeInForce")]
    pub time_in_force: Option<TimeInForce>,
    #[serde(rename = "gtdTime", skip_serializing_if = "Option::is_none")]
    pub gtd_time: Option<DateTime<Utc>>,
    #[serde(rename = "positionFill")]
    pub position_fill: Option<OrderPositionFill>,
    #[serde(rename = "triggerCondition")]
    pub trigger_condition: Option<OrderTriggerCondition>,
    #[serde(rename = "clientExtensions", skip_serializing_if = "Option::is_none")]
    pub client_extensions: Option<ClientExtensions>,
    #[serde(rename = "takeProfitOnFill", skip_serializing_if = "Option::is_none")]
    pub take_profit_on_fill: Option<TakeProfitDetails>,
    #[serde(rename = "stopLossOnFill", skip_serializing_if = "Option::is_none")]
    pub stop_loss_on_fill: Option<StopLossDetails>,
    #[serde(rename = "guaranteedStopLossOnFill", skip_serializing_if = "Option::is_none")]
    pub guaranteed_stop_loss_on_fill: Option<GuaranteedStopLossDetails>,
    #[serde(rename = "trailingStopLossOnFill", skip_serializing_if = "Option::is_none")]
    pub trailing_stop_loss_on_fill: Option<TrailingStopLossDetails>,
    #[serde(rename = "tradeClientExtensions", skip_serializing_if = "Option::is_none")]
    pub trade_client_extensions: Option<ClientExtensions>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MarketIfTouchedOrderRequest {
    pub instrument: InstrumentName,
    pub units: DecimalNumber,
    pub price: PriceValue,
    #[serde(rename = "priceBound", skip_serializing_if = "Option::is_none")]
    pub price_bound: Option<PriceValue>,
    #[serde(rename = "timeInForce")]
    pub time_in_force: Option<TimeInForce>,
    #[serde(rename = "gtdTime", skip_serializing_if = "Option::is_none")]
    pub gtd_time: Option<DateTime<Utc>>,
    #[serde(rename = "positionFill")]
    pub position_fill: Option<OrderPositionFill>,
    #[serde(rename = "triggerCondition")]
    pub trigger_condition: Option<OrderTriggerCondition>,
    #[serde(rename = "clientExtensions", skip_serializing_if = "Option::is_none")]
    pub client_extensions: Option<ClientExtensions>,
    #[serde(rename = "takeProfitOnFill", skip_serializing_if = "Option::is_none")]
    pub take_profit_on_fill: Option<TakeProfitDetails>,
    #[serde(rename = "stopLossOnFill", skip_serializing_if = "Option::is_none")]
    pub stop_loss_on_fill: Option<StopLossDetails>,
    #[serde(rename = "guaranteedStopLossOnFill", skip_serializing_if = "Option::is_none")]
    pub guaranteed_stop_loss_on_fill: Option<GuaranteedStopLossDetails>,
    #[serde(rename = "trailingStopLossOnFill", skip_serializing_if = "Option::is_none")]
    pub trailing_stop_loss_on_fill: Option<TrailingStopLossDetails>,
    #[serde(rename = "tradeClientExtensions", skip_serializing_if = "Option::is_none")]
    pub trade_client_extensions: Option<ClientExtensions>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TakeProfitOrderRequest {
    #[serde(rename = "tradeID")]
    pub trade_id: TradeID,
    #[serde(rename = "clientTradeID", skip_serializing_if = "Option::is_none")]
    pub client_trade_id: Option<ClientID>,
    pub price: PriceValue,
    #[serde(rename = "timeInForce")]
    pub time_in_force: Option<TimeInForce>,
    #[serde(rename = "gtdTime", skip_serializing_if = "Option::is_none")]
    pub gtd_time: Option<DateTime<Utc>>,
    #[serde(rename = "triggerCondition")]
    pub trigger_condition: Option<OrderTriggerCondition>,
    #[serde(rename = "clientExtensions", skip_serializing_if = "Option::is_none")]
    pub client_extensions: Option<ClientExtensions>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StopLossOrderRequest {
    #[serde(rename = "tradeID")]
    pub trade_id: TradeID,
    #[serde(rename = "clientTradeID", skip_serializing_if = "Option::is_none")]
    pub client_trade_id: Option<ClientID>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub price: Option<PriceValue>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub distance: Option<DecimalNumber>,
    #[serde(rename = "timeInForce")]
    pub time_in_force: Option<TimeInForce>,
    #[serde(rename = "gtdTime", skip_serializing_if = "Option::is_none")]
    pub gtd_time: Option<DateTime<Utc>>,
    #[serde(rename = "triggerCondition")]
    pub trigger_condition: Option<OrderTriggerCondition>,
    #[serde(rename = "clientExtensions", skip_serializing_if = "Option::is_none")]
    pub client_extensions: Option<ClientExtensions>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GuaranteedStopLossOrderRequest {
    #[serde(rename = "tradeID")]
    pub trade_id: TradeID,
    #[serde(rename = "clientTradeID", skip_serializing_if = "Option::is_none")]
    pub client_trade_id: Option<ClientID>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub price: Option<PriceValue>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub distance: Option<DecimalNumber>,
    #[serde(rename = "timeInForce")]
    pub time_in_force: Option<TimeInForce>,
    #[serde(rename = "gtdTime", skip_serializing_if = "Option::is_none")]
    pub gtd_time: Option<DateTime<Utc>>,
    #[serde(rename = "triggerCondition")]
    pub trigger_condition: Option<OrderTriggerCondition>,
    #[serde(rename = "clientExtensions", skip_serializing_if = "Option::is_none")]
    pub client_extensions: Option<ClientExtensions>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TrailingStopLossOrderRequest {
    #[serde(rename = "tradeID")]
    pub trade_id: TradeID,
    #[serde(rename = "clientTradeID", skip_serializing_if = "Option::is_none")]
    pub client_trade_id: Option<ClientID>,
    pub distance: DecimalNumber,
    #[serde(rename = "timeInForce")]
    pub time_in_force: Option<TimeInForce>,
    #[serde(rename = "gtdTime", skip_serializing_if = "Option::is_none")]
    pub gtd_time: Option<DateTime<Utc>>,
    #[serde(rename = "triggerCondition")]
    pub trigger_condition: Option<OrderTriggerCondition>,
    #[serde(rename = "clientExtensions", skip_serializing_if = "Option::is_none")]
    pub client_extensions: Option<ClientExtensions>,
}

// ---------------------------------------------------------------------------
// Create/Replace/Cancel Request & Response Types
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize)]
pub struct CreateOrderBody {
    pub order: OrderRequest,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateOrderResponse {
    #[serde(rename = "orderCreateTransaction")]
    pub order_create_transaction: Option<serde_json::Value>,
    #[serde(rename = "orderFillTransaction")]
    pub order_fill_transaction: Option<serde_json::Value>,
    #[serde(rename = "orderCancelTransaction")]
    pub order_cancel_transaction: Option<serde_json::Value>,
    #[serde(rename = "orderReissueTransaction")]
    pub order_reissue_transaction: Option<serde_json::Value>,
    #[serde(rename = "orderReissueRejectTransaction")]
    pub order_reissue_reject_transaction: Option<serde_json::Value>,
    #[serde(rename = "relatedTransactionIDs")]
    pub related_transaction_ids: Option<Vec<TransactionID>>,
    #[serde(rename = "lastTransactionID")]
    pub last_transaction_id: Option<TransactionID>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ReplaceOrderResponse {
    #[serde(rename = "orderCancelTransaction")]
    pub order_cancel_transaction: Option<serde_json::Value>,
    #[serde(rename = "orderCreateTransaction")]
    pub order_create_transaction: Option<serde_json::Value>,
    #[serde(rename = "orderFillTransaction")]
    pub order_fill_transaction: Option<serde_json::Value>,
    #[serde(rename = "orderReissueTransaction")]
    pub order_reissue_transaction: Option<serde_json::Value>,
    #[serde(rename = "orderReissueRejectTransaction")]
    pub order_reissue_reject_transaction: Option<serde_json::Value>,
    #[serde(rename = "relatedTransactionIDs")]
    pub related_transaction_ids: Option<Vec<TransactionID>>,
    #[serde(rename = "lastTransactionID")]
    pub last_transaction_id: Option<TransactionID>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CancelOrderResponse {
    #[serde(rename = "orderCancelTransaction")]
    pub order_cancel_transaction: Option<serde_json::Value>,
    #[serde(rename = "relatedTransactionIDs")]
    pub related_transaction_ids: Option<Vec<TransactionID>>,
    #[serde(rename = "lastTransactionID")]
    pub last_transaction_id: Option<TransactionID>,
}

#[derive(Debug, Serialize)]
pub struct UpdateClientExtensionsBody {
    #[serde(rename = "clientExtensions", skip_serializing_if = "Option::is_none")]
    pub client_extensions: Option<ClientExtensions>,
    #[serde(rename = "tradeClientExtensions", skip_serializing_if = "Option::is_none")]
    pub trade_client_extensions: Option<ClientExtensions>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateClientExtensionsResponse {
    #[serde(rename = "orderClientExtensionsModifyTransaction")]
    pub order_client_extensions_modify_transaction: Option<serde_json::Value>,
    #[serde(rename = "relatedTransactionIDs")]
    pub related_transaction_ids: Option<Vec<TransactionID>>,
    #[serde(rename = "lastTransactionID")]
    pub last_transaction_id: Option<TransactionID>,
}

// ---------------------------------------------------------------------------
// DynamicOrderState
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize, Deserialize)]
pub struct DynamicOrderState {
    pub id: OrderID,
    #[serde(rename = "trailingStopValue")]
    pub trailing_stop_value: Option<PriceValue>,
    #[serde(rename = "triggerDistance")]
    pub trigger_distance: Option<PriceValue>,
    #[serde(rename = "isTriggerDistanceExact")]
    pub is_trigger_distance_exact: Option<bool>,
}

// ---------------------------------------------------------------------------
// Enums
// ---------------------------------------------------------------------------

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

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum OrderState {
    Pending,
    Filled,
    Triggered,
    Cancelled,
}

#[derive(Debug, Serialize, Deserialize, Display)]
#[serde(rename_all = "UPPERCASE")]
pub enum OrderStateFilter {
    Pending,
    Filled,
    Triggered,
    Cancelled,
    All,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OrderIdentifier {
    #[serde(rename = "orderID")]
    pub order_id: OrderID,
    #[serde(rename = "clientOrderID")]
    pub client_order_id: ClientID,
}

pub type OrderSpecifier = String;

#[derive(Debug, Serialize, Deserialize)]
pub enum TimeInForce {
    GTC,
    GTD,
    GFD,
    FOK,
    IOC,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum OrderPositionFill {
    OpenOnly,
    ReduceFirst,
    ReduceOnly,
    Default,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum OrderTriggerCondition {
    Default,
    Inverse,
    Bid,
    Ask,
    Mid,
}

pub struct ListOrdersRequest {
    ids: Vec<OrderID>,
    state: Option<OrderStateFilter>,
    instrument: Option<InstrumentName>,
    count: Option<u16>,
    before_id: Option<OrderID>,
}

impl ListOrdersRequest {
    pub fn new() -> Self {
        ListOrdersRequest {
            ids: Vec::new(),
            state: None,
            instrument: None,
            count: None,
            before_id: None,
        }
    }

    pub fn ids(mut self, id: OrderID) -> Self {
        self.ids.push(id);
        self
    }

    pub fn state(mut self, state: OrderStateFilter) -> Self {
        self.state = Some(state);
        self
    }

    pub fn instrument(mut self, instrument: InstrumentName) -> Self {
        self.instrument = Some(instrument);
        self
    }

    pub fn count(mut self, count: u16) -> Self {
        self.count = Some(count);
        self
    }

    pub fn before_id(mut self, before_id: OrderID) -> Self {
        self.before_id = Some(before_id);
        self
    }

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

#[derive(Debug, Serialize, Deserialize)]
pub struct OrdersResponse {
    pub orders: Vec<Order>,
    #[serde(rename = "lastTransactionID")]
    pub last_transaction_id: TransactionID,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OrderResponse {
    pub order: Order,
    #[serde(rename = "lastTransactionID")]
    pub last_transaction_id: TransactionID,
}

pub struct OrderService<'a> {
    client: &'a Client,
}

impl<'a> OrderService<'a> {
    pub fn new(client: &'a Client) -> Self {
        OrderService { client }
    }



    pub async fn list(&self, req: ListOrdersRequest) -> Result<OrdersResponse, APIError> {
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
                let resp = http_resp.json::<OrdersResponse>().await?;
                Ok(resp)
            }
            status => Err(APIError::ApiErrorResponse {
                status,
                message: http_resp.text().await?,
            }),
        }
    }

    pub async fn list_pending(&self) -> Result<OrdersResponse, APIError> {
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
                let resp = http_resp.json::<OrdersResponse>().await?;
                Ok(resp)
            }
            status => Err(APIError::ApiErrorResponse {
                status,
                message: http_resp.text().await?,
            }),
        }
    }

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
        let http_resp = self
            .client
            .http_client
            .post(url)
            .json(&body)
            .send()
            .await?;
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

    pub async fn replace(
        &self,
        specifier: OrderSpecifier,
        order: OrderRequest,
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
        let body = CreateOrderBody { order };
        let http_resp = self
            .client
            .http_client
            .put(url)
            .json(&body)
            .send()
            .await?;
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

    pub async fn cancel(
        &self,
        specifier: OrderSpecifier,
    ) -> Result<CancelOrderResponse, APIError> {
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
        let http_resp = self
            .client
            .http_client
            .put(url)
            .json(&body)
            .send()
            .await?;
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

    pub async fn get_details(&self, specifier: OrderSpecifier) -> Result<OrderResponse, APIError> {
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
                let resp = http_resp.json::<OrderResponse>().await?;
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
    use crate::order::ListOrdersRequest;

    #[tokio::test]
    async fn test_list_orders() {
        let client = setup_test_client();
        let req = ListOrdersRequest::new().instrument(String::from("USD_JPY"));
        let resp = client.order().list(req).await.unwrap();
        println!("{:#?}", resp);
    }

    #[tokio::test]
    async fn test_list_pending() {
        let client = setup_test_client();
        let resp = client.order().list_pending().await.unwrap();
        println!("{:#?}", resp);
    }

    #[tokio::test]
    async fn test_get_details() {
        let client = setup_test_client();
        let resp = client.order().get_details(String::from("100")).await.unwrap();
        println!("{:#?}", resp);
    }
}
