use crate::account::AccountID;
use crate::client::Client;
use crate::errors::{APIError, CommonErrorResponse, ErrorResponse};
use crate::instrument::InstrumentName;
use crate::order::{OrderPositionFill, OrderTriggerCondition, TimeInForce};
use crate::pricing::{ClientPrice, PriceValue};
use crate::primitives::{Currency, DecimalNumber, HomeConversionFactors};
use crate::request_option_setter;
use chrono::{DateTime, Utc};
use reqwest::{Request, StatusCode};
use serde::{Deserialize, Serialize};
use strum_macros::Display;
use url::Url;

pub type TransactionID = String;
pub type ClientID = String;
pub type ClientTag = String;
pub type ClientComment = String;
pub type RequestID = String;
pub type AccountUnits = String;
pub type TradeID = String;
pub type OrderID = String;

// ---------------------------------------------------------------------------
// Transaction enum (tagged union)
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Transaction {
    #[serde(rename = "ORDER_FILL")]
    OrderFillTransaction(OrderFillTransaction),
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

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateTransaction {
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
    #[serde(rename = "divisionID")]
    pub division_id: Option<i64>,
    #[serde(rename = "siteID")]
    pub site_id: Option<i64>,
    #[serde(rename = "accountUserID")]
    pub account_user_id: Option<i64>,
    #[serde(rename = "accountNumber")]
    pub account_number: Option<i64>,
    #[serde(rename = "homeCurrency")]
    pub home_currency: Option<Currency>,
}

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

#[derive(Debug, Serialize, Deserialize)]
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
    pub alias: Option<String>,
    #[serde(rename = "marginRate")]
    pub margin_rate: Option<DecimalNumber>,
}

#[derive(Debug, Serialize, Deserialize)]
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
    #[serde(rename = "marginRate")]
    pub margin_rate: Option<DecimalNumber>,
    #[serde(rename = "rejectReason")]
    pub reject_reason: Option<TransactionRejectReason>,
}

#[derive(Debug, Serialize, Deserialize)]
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
    pub amount: Option<AccountUnits>,
    #[serde(rename = "fundingReason")]
    pub funding_reason: Option<FundingReason>,
    pub comment: Option<String>,
    #[serde(rename = "accountBalance")]
    pub account_balance: Option<AccountUnits>,
}

#[derive(Debug, Serialize, Deserialize)]
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
    pub amount: Option<AccountUnits>,
    #[serde(rename = "fundingReason")]
    pub funding_reason: Option<FundingReason>,
    pub comment: Option<String>,
    #[serde(rename = "rejectReason")]
    pub reject_reason: Option<TransactionRejectReason>,
}

#[derive(Debug, Serialize, Deserialize)]
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
    pub instrument: Option<InstrumentName>,
    pub units: Option<DecimalNumber>,
    #[serde(rename = "timeInForce")]
    pub time_in_force: Option<TimeInForce>,
    #[serde(rename = "priceBound")]
    pub price_bound: Option<PriceValue>,
    #[serde(rename = "positionFill")]
    pub position_fill: Option<OrderPositionFill>,
    #[serde(rename = "tradeClose")]
    pub trade_close: Option<MarketOrderTradeClose>,
    #[serde(rename = "longPositionCloseout")]
    pub long_position_closeout: Option<MarketOrderPositionCloseout>,
    #[serde(rename = "shortPositionCloseout")]
    pub short_position_closeout: Option<MarketOrderPositionCloseout>,
    #[serde(rename = "marginCloseout")]
    pub margin_closeout: Option<MarketOrderMarginCloseout>,
    #[serde(rename = "delayedTradeClose")]
    pub delayed_trade_close: Option<MarketOrderDelayedTradeClose>,
    pub reason: Option<MarketOrderReason>,
    #[serde(rename = "clientExtensions")]
    pub client_extensions: Option<ClientExtensions>,
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
}

#[derive(Debug, Serialize, Deserialize)]
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
    pub request_id: RequestID,
    pub instrument: InstrumentName,
    pub units: DecimalNumber,
    #[serde(rename = "timeInForce")]
    pub time_in_force: TimeInForce,
    #[serde(rename = "priceBound")]
    pub price_bound: Option<PriceValue>,
    #[serde(rename = "positionFill")]
    pub position_fill: OrderPositionFill,
    #[serde(rename = "tradeClose")]
    pub trade_close: Option<MarketOrderTradeClose>,
    #[serde(rename = "longPositionCloseout")]
    pub long_position_closeout: Option<MarketOrderPositionCloseout>,
    #[serde(rename = "shortPositionCloseout")]
    pub short_position_closeout: Option<MarketOrderPositionCloseout>,
    #[serde(rename = "marginCloseout")]
    pub margin_closeout: Option<MarketOrderMarginCloseout>,
    #[serde(rename = "delayedTradeClose")]
    pub delayed_trade_close: Option<MarketOrderDelayedTradeClose>,
    pub reason: MarketOrderReason,
    #[serde(rename = "clientExtensions")]
    pub client_extensions: Option<ClientExtensions>,
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
    #[serde(rename = "rejectReason")]
    pub reject_reason: Option<TransactionRejectReason>,
}

#[derive(Debug, Serialize, Deserialize)]
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
    pub request_id: RequestID,
    pub instrument: InstrumentName,
    pub units: DecimalNumber,
    pub price: PriceValue,
    #[serde(rename = "positionFill")]
    pub position_fill: OrderPositionFill,
    #[serde(rename = "tradeState")]
    pub trade_state: Option<String>,
    pub reason: Option<FixedPriceOrderReason>,
    #[serde(rename = "clientExtensions")]
    pub client_extensions: Option<ClientExtensions>,
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
}

#[derive(Debug, Serialize, Deserialize)]
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
    pub request_id: RequestID,
    pub instrument: InstrumentName,
    pub units: DecimalNumber,
    pub price: PriceValue,
    #[serde(rename = "timeInForce")]
    pub time_in_force: TimeInForce,
    #[serde(rename = "gtdTime")]
    pub gtd_time: Option<DateTime<Utc>>,
    #[serde(rename = "positionFill")]
    pub position_fill: OrderPositionFill,
    #[serde(rename = "triggerCondition")]
    pub trigger_condition: OrderTriggerCondition,
    pub reason: LimitOrderReason,
    #[serde(rename = "clientExtensions")]
    pub client_extensions: Option<ClientExtensions>,
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
    #[serde(rename = "replacesOrderID")]
    pub replaces_order_id: Option<OrderID>,
    #[serde(rename = "cancellingTransactionID")]
    pub cancelling_transaction_id: Option<TransactionID>,
}

#[derive(Debug, Serialize, Deserialize)]
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
    pub request_id: RequestID,
    pub instrument: InstrumentName,
    pub units: DecimalNumber,
    pub price: PriceValue,
    #[serde(rename = "timeInForce")]
    pub time_in_force: TimeInForce,
    #[serde(rename = "gtdTime")]
    pub gtd_time: Option<DateTime<Utc>>,
    #[serde(rename = "positionFill")]
    pub position_fill: OrderPositionFill,
    #[serde(rename = "triggerCondition")]
    pub trigger_condition: OrderTriggerCondition,
    pub reason: LimitOrderReason,
    #[serde(rename = "clientExtensions")]
    pub client_extensions: Option<ClientExtensions>,
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
    #[serde(rename = "replacesOrderID")]
    pub replaces_order_id: Option<OrderID>,
    #[serde(rename = "cancellingTransactionID")]
    pub cancelling_transaction_id: Option<TransactionID>,
    #[serde(rename = "intendedReplacesOrderID")]
    pub intended_replaces_order_id: Option<OrderID>,
    #[serde(rename = "rejectReason")]
    pub reject_reason: TransactionRejectReason,
}

#[derive(Debug, Serialize, Deserialize)]
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
    pub request_id: RequestID,
    pub instrument: InstrumentName,
    pub units: DecimalNumber,
    pub price: PriceValue,
    #[serde(rename = "priceBound")]
    pub price_bound: Option<PriceValue>,
    #[serde(rename = "timeInForce")]
    pub time_in_force: TimeInForce,
    #[serde(rename = "gtdTime")]
    pub gtd_time: Option<DateTime<Utc>>,
    #[serde(rename = "positionFill")]
    pub position_fill: OrderPositionFill,
    #[serde(rename = "triggerCondition")]
    pub trigger_condition: OrderTriggerCondition,
    pub reason: StopOrderReason,
    #[serde(rename = "clientExtensions")]
    pub client_extensions: Option<ClientExtensions>,
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
    #[serde(rename = "replacesOrderID")]
    pub replaces_order_id: Option<OrderID>,
    #[serde(rename = "cancellingTransactionID")]
    pub cancelling_transaction_id: Option<TransactionID>,
}

#[derive(Debug, Serialize, Deserialize)]
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
    pub request_id: RequestID,
    pub instrument: InstrumentName,
    pub units: DecimalNumber,
    pub price: PriceValue,
    #[serde(rename = "priceBound")]
    pub price_bound: Option<PriceValue>,
    #[serde(rename = "timeInForce")]
    pub time_in_force: TimeInForce,
    #[serde(rename = "gtdTime")]
    pub gtd_time: Option<DateTime<Utc>>,
    #[serde(rename = "positionFill")]
    pub position_fill: OrderPositionFill,
    #[serde(rename = "triggerCondition")]
    pub trigger_condition: OrderTriggerCondition,
    pub reason: StopOrderReason,
    #[serde(rename = "clientExtensions")]
    pub client_extensions: Option<ClientExtensions>,
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
    #[serde(rename = "replacesOrderID")]
    pub replaces_order_id: Option<OrderID>,
    #[serde(rename = "cancellingTransactionID")]
    pub cancelling_transaction_id: Option<TransactionID>,
    #[serde(rename = "intendedReplacesOrderID")]
    pub intended_replaces_order_id: Option<OrderID>,
    #[serde(rename = "rejectReason")]
    pub reject_reason: TransactionRejectReason,
}

#[derive(Debug, Serialize, Deserialize)]
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
    pub request_id: RequestID,
    pub instrument: InstrumentName,
    pub units: DecimalNumber,
    pub price: PriceValue,
    #[serde(rename = "priceBound")]
    pub price_bound: Option<PriceValue>,
    #[serde(rename = "timeInForce")]
    pub time_in_force: TimeInForce,
    #[serde(rename = "gtdTime")]
    pub gtd_time: Option<DateTime<Utc>>,
    #[serde(rename = "positionFill")]
    pub position_fill: OrderPositionFill,
    #[serde(rename = "triggerCondition")]
    pub trigger_condition: OrderTriggerCondition,
    pub reason: MarketIfTouchedOrderReason,
    #[serde(rename = "clientExtensions")]
    pub client_extensions: Option<ClientExtensions>,
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
    #[serde(rename = "replacesOrderID")]
    pub replaces_order_id: Option<OrderID>,
    #[serde(rename = "cancellingTransactionID")]
    pub cancelling_transaction_id: Option<TransactionID>,
}

#[derive(Debug, Serialize, Deserialize)]
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
    pub request_id: RequestID,
    pub instrument: InstrumentName,
    pub units: DecimalNumber,
    pub price: PriceValue,
    #[serde(rename = "priceBound")]
    pub price_bound: Option<PriceValue>,
    #[serde(rename = "timeInForce")]
    pub time_in_force: TimeInForce,
    #[serde(rename = "gtdTime")]
    pub gtd_time: Option<DateTime<Utc>>,
    #[serde(rename = "positionFill")]
    pub position_fill: OrderPositionFill,
    #[serde(rename = "triggerCondition")]
    pub trigger_condition: OrderTriggerCondition,
    pub reason: MarketIfTouchedOrderReason,
    #[serde(rename = "clientExtensions")]
    pub client_extensions: Option<ClientExtensions>,
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
    #[serde(rename = "replacesOrderID")]
    pub replaces_order_id: Option<OrderID>,
    #[serde(rename = "cancellingTransactionID")]
    pub cancelling_transaction_id: Option<TransactionID>,
    #[serde(rename = "intendedReplacesOrderID")]
    pub intended_replaces_order_id: Option<OrderID>,
    #[serde(rename = "rejectReason")]
    pub reject_reason: TransactionRejectReason,
}

#[derive(Debug, Serialize, Deserialize)]
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
    pub request_id: RequestID,
    #[serde(rename = "tradeID")]
    pub trade_id: TradeID,
    #[serde(rename = "clientTradeID")]
    pub client_trade_id: Option<ClientID>,
    pub price: PriceValue,
    #[serde(rename = "timeInForce")]
    pub time_in_force: TimeInForce,
    #[serde(rename = "gtdTime")]
    pub gtd_time: Option<DateTime<Utc>>,
    #[serde(rename = "triggerCondition")]
    pub trigger_condition: OrderTriggerCondition,
    pub reason: TakeProfitOrderReason,
    #[serde(rename = "clientExtensions")]
    pub client_extensions: Option<ClientExtensions>,
    #[serde(rename = "orderFillTransactionID")]
    pub order_fill_transaction_id: Option<TransactionID>,
    #[serde(rename = "replacesOrderID")]
    pub replaces_order_id: Option<OrderID>,
    #[serde(rename = "cancellingTransactionID")]
    pub cancelling_transaction_id: Option<TransactionID>,
}

#[derive(Debug, Serialize, Deserialize)]
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
    pub request_id: RequestID,
    #[serde(rename = "tradeID")]
    pub trade_id: TradeID,
    #[serde(rename = "clientTradeID")]
    pub client_trade_id: Option<ClientID>,
    pub price: PriceValue,
    #[serde(rename = "timeInForce")]
    pub time_in_force: TimeInForce,
    #[serde(rename = "gtdTime")]
    pub gtd_time: Option<DateTime<Utc>>,
    #[serde(rename = "triggerCondition")]
    pub trigger_condition: OrderTriggerCondition,
    pub reason: TakeProfitOrderReason,
    #[serde(rename = "clientExtensions")]
    pub client_extensions: Option<ClientExtensions>,
    #[serde(rename = "orderFillTransactionID")]
    pub order_fill_transaction_id: Option<TransactionID>,
    #[serde(rename = "replacesOrderID")]
    pub replaces_order_id: Option<OrderID>,
    #[serde(rename = "cancellingTransactionID")]
    pub cancelling_transaction_id: Option<TransactionID>,
    #[serde(rename = "intendedReplacesOrderID")]
    pub intended_replaces_order_id: Option<OrderID>,
    #[serde(rename = "rejectReason")]
    pub reject_reason: TransactionRejectReason,
}

#[derive(Debug, Serialize, Deserialize)]
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
    pub request_id: RequestID,
    #[serde(rename = "tradeID")]
    pub trade_id: TradeID,
    #[serde(rename = "clientTradeID")]
    pub client_trade_id: Option<ClientID>,
    pub price: PriceValue,
    pub distance: Option<DecimalNumber>,
    #[serde(rename = "timeInForce")]
    pub time_in_force: TimeInForce,
    #[serde(rename = "gtdTime")]
    pub gtd_time: Option<DateTime<Utc>>,
    #[serde(rename = "triggerCondition")]
    pub trigger_condition: OrderTriggerCondition,
    pub reason: StopLossOrderReason,
    #[serde(rename = "clientExtensions")]
    pub client_extensions: Option<ClientExtensions>,
    #[serde(rename = "orderFillTransactionID")]
    pub order_fill_transaction_id: Option<TransactionID>,
    #[serde(rename = "replacesOrderID")]
    pub replaces_order_id: Option<OrderID>,
    #[serde(rename = "cancellingTransactionID")]
    pub cancelling_transaction_id: Option<TransactionID>,
}

#[derive(Debug, Serialize, Deserialize)]
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
    pub request_id: RequestID,
    #[serde(rename = "tradeID")]
    pub trade_id: TradeID,
    #[serde(rename = "clientTradeID")]
    pub client_trade_id: Option<ClientID>,
    pub price: PriceValue,
    pub distance: Option<DecimalNumber>,
    #[serde(rename = "timeInForce")]
    pub time_in_force: TimeInForce,
    #[serde(rename = "gtdTime")]
    pub gtd_time: Option<DateTime<Utc>>,
    #[serde(rename = "triggerCondition")]
    pub trigger_condition: OrderTriggerCondition,
    pub reason: StopLossOrderReason,
    #[serde(rename = "clientExtensions")]
    pub client_extensions: Option<ClientExtensions>,
    #[serde(rename = "orderFillTransactionID")]
    pub order_fill_transaction_id: Option<TransactionID>,
    #[serde(rename = "replacesOrderID")]
    pub replaces_order_id: Option<OrderID>,
    #[serde(rename = "cancellingTransactionID")]
    pub cancelling_transaction_id: Option<TransactionID>,
    #[serde(rename = "intendedReplacesOrderID")]
    pub intended_replaces_order_id: Option<OrderID>,
    #[serde(rename = "rejectReason")]
    pub reject_reason: TransactionRejectReason,
}

#[derive(Debug, Serialize, Deserialize)]
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
    pub request_id: RequestID,
    #[serde(rename = "tradeID")]
    pub trade_id: TradeID,
    #[serde(rename = "clientTradeID")]
    pub client_trade_id: Option<ClientID>,
    pub price: PriceValue,
    pub distance: Option<DecimalNumber>,
    #[serde(rename = "timeInForce")]
    pub time_in_force: TimeInForce,
    #[serde(rename = "gtdTime")]
    pub gtd_time: Option<DateTime<Utc>>,
    #[serde(rename = "triggerCondition")]
    pub trigger_condition: OrderTriggerCondition,
    pub reason: GuaranteedStopLossOrderReason,
    #[serde(rename = "clientExtensions")]
    pub client_extensions: Option<ClientExtensions>,
    #[serde(rename = "orderFillTransactionID")]
    pub order_fill_transaction_id: Option<TransactionID>,
    #[serde(rename = "replacesOrderID")]
    pub replaces_order_id: Option<OrderID>,
    #[serde(rename = "cancellingTransactionID")]
    pub cancelling_transaction_id: Option<TransactionID>,
}

#[derive(Debug, Serialize, Deserialize)]
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
    pub request_id: RequestID,
    #[serde(rename = "tradeID")]
    pub trade_id: TradeID,
    #[serde(rename = "clientTradeID")]
    pub client_trade_id: Option<ClientID>,
    pub price: PriceValue,
    pub distance: Option<DecimalNumber>,
    #[serde(rename = "timeInForce")]
    pub time_in_force: TimeInForce,
    #[serde(rename = "gtdTime")]
    pub gtd_time: Option<DateTime<Utc>>,
    #[serde(rename = "triggerCondition")]
    pub trigger_condition: OrderTriggerCondition,
    pub reason: GuaranteedStopLossOrderReason,
    #[serde(rename = "clientExtensions")]
    pub client_extensions: Option<ClientExtensions>,
    #[serde(rename = "orderFillTransactionID")]
    pub order_fill_transaction_id: Option<TransactionID>,
    #[serde(rename = "replacesOrderID")]
    pub replaces_order_id: Option<OrderID>,
    #[serde(rename = "cancellingTransactionID")]
    pub cancelling_transaction_id: Option<TransactionID>,
    #[serde(rename = "intendedReplacesOrderID")]
    pub intended_replaces_order_id: Option<OrderID>,
    #[serde(rename = "rejectReason")]
    pub reject_reason: TransactionRejectReason,
}

#[derive(Debug, Serialize, Deserialize)]
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
    pub request_id: RequestID,
    #[serde(rename = "tradeID")]
    pub trade_id: TradeID,
    #[serde(rename = "clientTradeID")]
    pub client_trade_id: Option<ClientID>,
    pub distance: DecimalNumber,
    #[serde(rename = "timeInForce")]
    pub time_in_force: TimeInForce,
    #[serde(rename = "gtdTime")]
    pub gtd_time: Option<DateTime<Utc>>,
    #[serde(rename = "triggerCondition")]
    pub trigger_condition: OrderTriggerCondition,
    pub reason: TrailingStopLossOrderReason,
    #[serde(rename = "clientExtensions")]
    pub client_extensions: Option<ClientExtensions>,
    #[serde(rename = "orderFillTransactionID")]
    pub order_fill_transaction_id: Option<TransactionID>,
    #[serde(rename = "replacesOrderID")]
    pub replaces_order_id: Option<OrderID>,
    #[serde(rename = "cancellingTransactionID")]
    pub cancelling_transaction_id: Option<TransactionID>,
}

#[derive(Debug, Serialize, Deserialize)]
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
    pub request_id: RequestID,
    #[serde(rename = "tradeID")]
    pub trade_id: TradeID,
    #[serde(rename = "clientTradeID")]
    pub client_trade_id: Option<ClientID>,
    pub distance: DecimalNumber,
    #[serde(rename = "timeInForce")]
    pub time_in_force: TimeInForce,
    #[serde(rename = "gtdTime")]
    pub gtd_time: Option<DateTime<Utc>>,
    #[serde(rename = "triggerCondition")]
    pub trigger_condition: OrderTriggerCondition,
    pub reason: TrailingStopLossOrderReason,
    #[serde(rename = "clientExtensions")]
    pub client_extensions: Option<ClientExtensions>,
    #[serde(rename = "orderFillTransactionID")]
    pub order_fill_transaction_id: Option<TransactionID>,
    #[serde(rename = "replacesOrderID")]
    pub replaces_order_id: Option<OrderID>,
    #[serde(rename = "cancellingTransactionID")]
    pub cancelling_transaction_id: Option<TransactionID>,
    #[serde(rename = "intendedReplacesOrderID")]
    pub intended_replaces_order_id: Option<OrderID>,
    #[serde(rename = "rejectReason")]
    pub reject_reason: TransactionRejectReason,
}

#[derive(Debug, Serialize, Deserialize)]
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
    pub request_id: RequestID,
    #[serde(rename = "orderID")]
    pub order_id: OrderID,
    #[serde(rename = "clientOrderID")]
    pub client_order_id: Option<ClientID>,
    pub instrument: InstrumentName,
    pub units: DecimalNumber,
    #[serde(rename = "homeConversionFactors")]
    pub home_conversion_factors: Option<HomeConversionFactors>,
    #[serde(rename = "fullVWAP")]
    pub full_vwap: PriceValue,
    #[serde(rename = "fullPrice")]
    pub full_price: ClientPrice,
    pub reason: OrderFillReason,
    pub pl: AccountUnits,
    pub financing: AccountUnits,
    #[serde(rename = "baseFinancing")]
    pub base_financing: AccountUnits,
    #[serde(rename = "quoteFinancing")]
    pub quote_financing: Option<AccountUnits>,
    pub commission: AccountUnits,
    #[serde(rename = "guaranteedExecutionFee")]
    pub guaranteed_execution_fee: AccountUnits,
    #[serde(rename = "quoteGuaranteedExecutionFee")]
    pub quote_guaranteed_execution_fee: AccountUnits,
    #[serde(rename = "halfSpreadCost")]
    pub half_spread_cost: AccountUnits,
    #[serde(rename = "accountBalance")]
    pub account_balance: AccountUnits,
    #[serde(rename = "tradeOpened")]
    pub trade_opened: Option<TradeOpen>,
    #[serde(rename = "tradesClosed")]
    pub trades_closed: Option<Vec<TradeReduce>>,
    #[serde(rename = "tradeReduced")]
    pub trade_reduced: Option<TradeReduce>,
}

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
    pub request_id: RequestID,
    #[serde(rename = "orderID")]
    pub order_id: OrderID,
    #[serde(rename = "clientOrderID")]
    pub client_order_id: Option<ClientID>,
    pub reason: OrderCancelReason,
    #[serde(rename = "replacedByOrderID")]
    pub replaced_by_order_id: Option<OrderID>,
}

#[derive(Debug, Serialize, Deserialize)]
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
    pub request_id: RequestID,
    #[serde(rename = "orderID")]
    pub order_id: OrderID,
    #[serde(rename = "clientOrderID")]
    pub client_order_id: Option<ClientID>,
    #[serde(rename = "rejectReason")]
    pub reject_reason: TransactionRejectReason,
}

#[derive(Debug, Serialize, Deserialize)]
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
    pub request_id: RequestID,
    #[serde(rename = "orderID")]
    pub order_id: OrderID,
    #[serde(rename = "clientOrderID")]
    pub client_order_id: Option<ClientID>,
    #[serde(rename = "clientExtensionsModify")]
    pub client_extensions_modify: Option<ClientExtensions>,
    #[serde(rename = "tradeClientExtensionsModify")]
    pub trade_client_extensions_modify: Option<ClientExtensions>,
}

#[derive(Debug, Serialize, Deserialize)]
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
    pub request_id: RequestID,
    #[serde(rename = "orderID")]
    pub order_id: OrderID,
    #[serde(rename = "clientOrderID")]
    pub client_order_id: Option<ClientID>,
    #[serde(rename = "clientExtensionsModify")]
    pub client_extensions_modify: Option<ClientExtensions>,
    #[serde(rename = "tradeClientExtensionsModify")]
    pub trade_client_extensions_modify: Option<ClientExtensions>,
    #[serde(rename = "rejectReason")]
    pub reject_reason: TransactionRejectReason,
}

#[derive(Debug, Serialize, Deserialize)]
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
    pub request_id: RequestID,
    #[serde(rename = "tradeID")]
    pub trade_id: TradeID,
    #[serde(rename = "clientTradeID")]
    pub client_trade_id: Option<ClientID>,
    #[serde(rename = "tradeClientExtensionsModify")]
    pub trade_client_extensions_modify: Option<ClientExtensions>,
}

#[derive(Debug, Serialize, Deserialize)]
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
    pub request_id: RequestID,
    #[serde(rename = "tradeID")]
    pub trade_id: TradeID,
    #[serde(rename = "clientTradeID")]
    pub client_trade_id: Option<ClientID>,
    #[serde(rename = "tradeClientExtensionsModify")]
    pub trade_client_extensions_modify: Option<ClientExtensions>,
    #[serde(rename = "rejectReason")]
    pub reject_reason: TransactionRejectReason,
}

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
    pub request_id: RequestID,
}

#[derive(Debug, Serialize, Deserialize)]
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
    pub request_id: RequestID,
    #[serde(rename = "extensionNumber")]
    pub extension_number: Option<i64>,
}

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
    pub request_id: RequestID,
}

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
    pub request_id: RequestID,
    pub reason: MarketOrderReason,
    #[serde(rename = "tradeIDs")]
    pub trade_ids: Vec<TradeID>,
}

#[derive(Debug, Serialize, Deserialize)]
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
    pub request_id: RequestID,
    pub financing: AccountUnits,
    #[serde(rename = "accountBalance")]
    pub account_balance: AccountUnits,
    #[serde(rename = "positionFinancings")]
    pub position_financings: Vec<PositionFinancing>,
}

#[derive(Debug, Serialize, Deserialize)]
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
    pub request_id: RequestID,
    pub instrument: InstrumentName,
    #[serde(rename = "dividendAdjustment")]
    pub dividend_adjustment: AccountUnits,
    #[serde(rename = "quoteDividendAdjustment")]
    pub quote_dividend_adjustment: AccountUnits,
    #[serde(rename = "homeConversionFactors")]
    pub home_conversion_factors: HomeConversionFactors,
    #[serde(rename = "accountBalance")]
    pub account_balance: AccountUnits,
    #[serde(rename = "openTradeDividendAdjustments")]
    pub open_trade_dividend_adjustments: Vec<OpenTradeDividendAdjustment>,
}

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
    pub request_id: RequestID,
}

#[derive(Debug, Serialize, Deserialize, Display)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
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

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum FundingReason {
    ClientFunding,
    AccountTransfer,
    DivisionMigration,
    SiteMigration,
    Adjustment,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum MarketOrderReason {
    ClientOrder,
    TradeClose,
    PositionCloseout,
    MarginCloseout,
    DelayedTradeClose,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum FixedPriceOrderReason {
    PlatformAccountMigration,
    TradeCloseDivisionAccountMigration,
    TradeCloseAdministrativeAction,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum LimitOrderReason {
    ClientOrder,
    Replacement,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum StopOrderReason {
    ClientOrder,
    Replacement,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum MarketIfTouchedOrderReason {
    ClientOrder,
    Replacement,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TakeProfitOrderReason {
    ClientOrder,
    Replacement,
    OnFill,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum StopLossOrderReason {
    ClientOrder,
    Replacement,
    OnFill,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum GuaranteedStopLossOrderReason {
    ClientOrder,
    Replacement,
    OnFill,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TrailingStopLossOrderReason {
    ClientOrder,
    Replacement,
    OnFill,
}

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

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AccountFinancingMode {
    NoFinancing,
    SecondBySecond,
    Daily,
}

#[derive(Debug, Serialize, Deserialize, Display)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
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

#[derive(Debug, Serialize, Deserialize)]
pub struct MarketOrderTradeClose {
    #[serde(rename = "tradeID")]
    pub trade_id: TradeID,
    pub units: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MarketOrderMarginCloseout {
    pub reason: MarketOrderMarginCloseoutReason,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum MarketOrderMarginCloseoutReason {
    MarginCheckViolation,
    RegulatoryMarginCallViolation,
    RegulatoryMarginCheckViolation,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MarketOrderDelayedTradeClose {
    #[serde(rename = "tradeID")]
    pub trade_id: TradeID,
    #[serde(rename = "sourceTransactionID")]
    pub source_transaction_id: TransactionID,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MarketOrderPositionCloseout {
    pub instrument: InstrumentName,
    pub units: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TakeProfitDetails {
    pub price: PriceValue,
    #[serde(rename = "timeInForce")]
    pub time_in_force: Option<TimeInForce>,
    #[serde(rename = "gtdTime")]
    pub gtd_time: Option<DateTime<Utc>>,
    #[serde(rename = "clientExtensions")]
    pub client_extensions: Option<ClientExtensions>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StopLossDetails {
    pub price: Option<PriceValue>,
    pub distance: Option<DecimalNumber>,
    #[serde(rename = "timeInForce")]
    pub time_in_force: Option<TimeInForce>,
    #[serde(rename = "gtdTime")]
    pub gtd_time: Option<DateTime<Utc>>,
    #[serde(rename = "clientExtensions")]
    pub client_extensions: Option<ClientExtensions>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TrailingStopLossDetails {
    pub distance: DecimalNumber,
    #[serde(rename = "timeInForce")]
    pub time_in_force: Option<TimeInForce>,
    #[serde(rename = "gtdTime")]
    pub gtd_time: Option<DateTime<Utc>>,
    #[serde(rename = "clientExtensions")]
    pub client_extensions: Option<ClientExtensions>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GuaranteedStopLossDetails {
    pub price: Option<PriceValue>,
    pub distance: Option<DecimalNumber>,
    #[serde(rename = "timeInForce")]
    pub time_in_force: Option<TimeInForce>,
    #[serde(rename = "gtdTime")]
    pub gtd_time: Option<DateTime<Utc>>,
    #[serde(rename = "clientExtensions")]
    pub client_extensions: Option<ClientExtensions>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TradeOpen {
    #[serde(rename = "tradeID")]
    pub trade_id: TradeID,
    pub units: DecimalNumber,
    #[serde(rename = "price")]
    pub price: Option<PriceValue>,
    #[serde(rename = "guaranteedExecutionFee")]
    pub guaranteed_execution_fee: Option<AccountUnits>,
    #[serde(rename = "clientExtensions")]
    pub client_extensions: Option<ClientExtensions>,
    #[serde(rename = "halfSpreadCost")]
    pub half_spread_cost: Option<AccountUnits>,
    #[serde(rename = "initialMarginRequired")]
    pub initial_margin_required: Option<AccountUnits>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TradeReduce {
    #[serde(rename = "tradeID")]
    pub trade_id: TradeID,
    pub units: DecimalNumber,
    #[serde(rename = "price")]
    pub price: Option<PriceValue>,
    #[serde(rename = "realizedPL")]
    pub realized_pl: AccountUnits,
    pub financing: AccountUnits,
    #[serde(rename = "guaranteedExecutionFee")]
    pub guaranteed_execution_fee: Option<AccountUnits>,
    #[serde(rename = "halfSpreadCost")]
    pub half_spread_cost: Option<AccountUnits>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OpenTradeFinancing {
    #[serde(rename = "tradeID")]
    pub trade_id: TradeID,
    pub financing: AccountUnits,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PositionFinancing {
    pub instrument: InstrumentName,
    pub financing: AccountUnits,
    #[serde(rename = "openTradeFinancings")]
    pub open_trade_financings: Option<Vec<OpenTradeFinancing>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OpenTradeDividendAdjustment {
    #[serde(rename = "tradeID")]
    pub trade_id: TradeID,
    #[serde(rename = "dividendAdjustment")]
    pub dividend_adjustment: AccountUnits,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TransactionHeartbeat {
    pub time: DateTime<Utc>,
    #[serde(rename = "lastTransactionID")]
    pub last_transaction_id: TransactionID,
}

// ---------------------------------------------------------------------------
// Client Extensions
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize, Deserialize)]
pub struct ClientExtensions {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<ClientID>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tag: Option<ClientTag>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comment: Option<ClientComment>,
}

impl ClientExtensions {
    pub fn new() -> Self {
        ClientExtensions {
            id: None,
            tag: None,
            comment: None,
        }
    }

    request_option_setter!(id, ClientID);
    request_option_setter!(tag, ClientTag);
    request_option_setter!(comment, ClientComment);
}

// ---------------------------------------------------------------------------
// Request/Response types
// ---------------------------------------------------------------------------

pub struct ListTransactionsRequest {
    pub from: Option<DateTime<Utc>>,
    pub to: Option<DateTime<Utc>>,
    pub page_size: Option<u16>,
    pub transaction_type: Vec<TransactionType>,
}

impl ListTransactionsRequest {
    pub fn new() -> Self {
        ListTransactionsRequest {
            from: None,
            to: None,
            page_size: None,
            transaction_type: Vec::new(),
        }
    }

    pub fn from(mut self, from: DateTime<Utc>) -> Self {
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
        if self.transaction_type.len() > 0 {
            url.query_pairs_mut().append_pair(
                "transactionType",
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

#[derive(Debug, Serialize, Deserialize)]
pub struct ListTransactionsResponse {
    pub count: i64,
    pub from: DateTime<Utc>,
    pub to: DateTime<Utc>,
    #[serde(rename = "pageSize")]
    pub page_size: i64,
    pub pages: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetTransactionDetailsResponse {
    pub transaction: Transaction,
    #[serde(rename = "lastTransactionID")]
    pub last_transaction_id: TransactionID,
}

pub struct GetTransactionsByIDRangeRequest {
    from: TransactionID,
    to: TransactionID,
    filter: Vec<TransactionFilter>,
}

impl GetTransactionsByIDRangeRequest {
    pub fn new(from: TransactionID, to: TransactionID) -> Self {
        GetTransactionsByIDRangeRequest {
            from,
            to,
            filter: Vec::new(),
        }
    }

    pub fn filter(mut self, filter: TransactionFilter) -> Self {
        self.filter.push(filter);
        self
    }

    pub(crate) fn set_params(&self, url: &mut Url) {
        url.query_pairs_mut()
            .append_pair("from", &self.from.to_string())
            .append_pair("to", &self.to.to_string());
        if self.filter.len() > 0 {
            url.query_pairs_mut().append_pair(
                "type",
                &self
                    .filter
                    .iter()
                    .map(|f| f.to_string())
                    .collect::<Vec<String>>()
                    .join(",")
                    .as_str(),
            );
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetTransactionsResponse {
    transactions: Vec<Transaction>,
    #[serde(rename = "lastTransactionID")]
    pub last_transaction_id: TransactionID,
}

pub struct GetTransactionsBySinceIDRequest {
    id: TransactionID,
    filter: Vec<TransactionFilter>,
}

impl GetTransactionsBySinceIDRequest {
    pub fn new(id: TransactionID) -> Self {
        GetTransactionsBySinceIDRequest {
            id,
            filter: Vec::new(),
        }
    }

    pub fn filter(mut self, filter: TransactionFilter) -> Self {
        self.filter.push(filter);
        self
    }

    pub(crate) fn set_params(&self, url: &mut Url) {
        url.query_pairs_mut()
            .append_pair("id", &self.id.to_string());
        if self.filter.len() > 0 {
            url.query_pairs_mut().append_pair(
                "filter",
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

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum TransactionStreamItem {
    HEARTBEAT(TransactionHeartbeat),
    #[serde(untagged)]
    Transaction(Transaction),
}

pub struct TransactionService<'a> {
    client: &'a Client,
}

impl<'a> TransactionService<'a> {
    pub(crate) fn new(client: &'a Client) -> Self {
        TransactionService { client }
    }

    pub async fn list(
        &self,
        req: ListTransactionsRequest,
    ) -> Result<ListTransactionsResponse, APIError> {
        let mut url = self
            .client
            .base_url
            .join(
                format!(
                    "/v3/accounts/{}/transactions",
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
            StatusCode::OK => Ok(http_resp.json::<ListTransactionsResponse>().await?),
            _ => Err(APIError::ErrorResponse(ErrorResponse::CommonError(
                http_resp.json::<CommonErrorResponse>().await?,
            ))),
        }
    }

    pub async fn get_details(
        &self,
        id: TransactionID,
    ) -> Result<GetTransactionDetailsResponse, APIError> {
        let url = self
            .client
            .base_url
            .join(
                format!(
                    "/v3/accounts/{}/transactions/{}",
                    self.client
                        .account_id
                        .as_ref()
                        .expect("Missing account_id in client"),
                    id
                )
                .as_str(),
            )
            .unwrap();
        let http_req = Request::new(reqwest::Method::GET, url);
        let http_resp = self.client.http_client.execute(http_req).await?;
        match http_resp.status() {
            StatusCode::OK => Ok(http_resp.json::<GetTransactionDetailsResponse>().await?),
            _ => Err(APIError::ErrorResponse(ErrorResponse::CommonError(
                http_resp.json::<CommonErrorResponse>().await?,
            ))),
        }
    }

    pub async fn get_by_id_range(
        &self,
        req: GetTransactionsByIDRangeRequest,
    ) -> Result<GetTransactionsResponse, APIError> {
        let mut url = self
            .client
            .base_url
            .join(
                format!(
                    "/v3/accounts/{}/transactions/idrange",
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
            StatusCode::OK => Ok(http_resp.json::<GetTransactionsResponse>().await?),
            _ => Err(APIError::ErrorResponse(ErrorResponse::CommonError(
                http_resp.json::<CommonErrorResponse>().await?,
            ))),
        }
    }

    pub async fn get_by_since_id(
        &self,
        req: GetTransactionsBySinceIDRequest,
    ) -> Result<GetTransactionsResponse, APIError> {
        let mut url = self
            .client
            .base_url
            .join(
                format!(
                    "/v3/accounts/{}/transactions/sinceid",
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
            StatusCode::OK => Ok(http_resp.json::<GetTransactionsResponse>().await?),
            _ => Err(APIError::ErrorResponse(ErrorResponse::CommonError(
                http_resp.json::<CommonErrorResponse>().await?,
            ))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::client::setup_test_client;

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
        let resp = client
            .transaction()
            .get_details("501".to_string())
            .await
            .unwrap();
        println!("{:#?}", resp);
    }

    #[tokio::test]
    async fn test_get_transactions_by_id_range() {
        let client = setup_test_client();
        let req = GetTransactionsByIDRangeRequest::new("500".to_string(), "510".to_string());
        let resp = client.transaction().get_by_id_range(req).await.unwrap();
        println!("{:#?}", resp);
    }

    #[tokio::test]
    async fn test_get_transactions_by_since_id() {
        let client = setup_test_client();
        let req = GetTransactionsBySinceIDRequest::new("520".to_string());
        let resp = client.transaction().get_by_since_id(req).await.unwrap();
        println!("{:#?}", resp);
    }
}
