use crate::account::AccountID;
use crate::instrument::InstrumentName;
use crate::order::{OrderPositionFill, OrderTriggerCondition, TimeInForce};
use crate::pricing::PriceValue;
use crate::primitives::{Currency, DecimalNumber};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

pub type TransactionID = String;
pub type ClientID = String;
pub type ClientTag = String;
pub type ClientComment = String;
pub type RequestID = String;
pub type AccountUnits = String;
pub type TradeID = String;
pub type OrderID = String;

// ---------------------------------------------------------------------------
// Client Extensions
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize, Deserialize)]
pub struct ClientExtensions {
    pub id: ClientID,
    pub tag: ClientTag,
    pub comment: ClientComment,
}

// ---------------------------------------------------------------------------
// Helper Structs
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize, Deserialize)]
pub struct MarketOrderTradeClose {
    #[serde(rename = "tradeID")]
    pub trade_id: TradeID,
    pub units: Option<DecimalNumber>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MarketOrderPositionCloseout {
    pub instrument: InstrumentName,
    pub units: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MarketOrderMarginCloseout {
    pub reason: MarketOrderMarginCloseoutReason,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MarketOrderDelayedTradeClose {
    #[serde(rename = "tradeID")]
    pub trade_id: TradeID,
    #[serde(rename = "sourceTransactionID")]
    pub source_transaction_id: TransactionID,
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
pub struct HomeConversionFactors {
    #[serde(rename = "gainQuoteHomeConversionFactor")]
    pub gain_quote_home_conversion_factor: DecimalNumber,
    #[serde(rename = "lossQuoteHomeConversionFactor")]
    pub loss_quote_home_conversion_factor: DecimalNumber,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TransactionHeartbeat {
    #[serde(rename = "type")]
    pub heartbeat_type: String,
    pub time: DateTime<Utc>,
    #[serde(rename = "lastTransactionID")]
    pub last_transaction_id: TransactionID,
}

// ---------------------------------------------------------------------------
// Enums
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize, Deserialize)]
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
pub enum MarketOrderMarginCloseoutReason {
    MarginCheckViolation,
    RegulatoryMarginCallViolation,
    RegulatoryMarginCheckViolation,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AccountFinancingMode {
    NoFinancing,
    SecondBySecond,
    Daily,
}

#[derive(Debug, Serialize, Deserialize)]
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
    UnitsMimimumNotMet,
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

// ---------------------------------------------------------------------------
// Transaction enum (tagged union)
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize, Deserialize)]
pub enum Transaction {}

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
    #[serde(rename = "type")]
    pub transaction_type: Option<TransactionType>,
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
    #[serde(rename = "type")]
    pub transaction_type: Option<TransactionType>,
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
    #[serde(rename = "type")]
    pub transaction_type: Option<TransactionType>,
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
    #[serde(rename = "type")]
    pub transaction_type: Option<TransactionType>,
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
    #[serde(rename = "type")]
    pub transaction_type: Option<TransactionType>,
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
    #[serde(rename = "type")]
    pub transaction_type: Option<TransactionType>,
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
    #[serde(rename = "type")]
    pub transaction_type: Option<TransactionType>,
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
    #[serde(rename = "type")]
    pub transaction_type: Option<TransactionType>,
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
    pub request_id: Option<RequestID>,
    #[serde(rename = "type")]
    pub transaction_type: Option<TransactionType>,
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
    pub request_id: Option<RequestID>,
    #[serde(rename = "type")]
    pub transaction_type: Option<TransactionType>,
    pub instrument: Option<InstrumentName>,
    pub units: Option<DecimalNumber>,
    pub price: Option<PriceValue>,
    #[serde(rename = "positionFill")]
    pub position_fill: Option<OrderPositionFill>,
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
    pub request_id: Option<RequestID>,
    #[serde(rename = "type")]
    pub transaction_type: Option<TransactionType>,
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
    pub reason: Option<LimitOrderReason>,
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
    pub request_id: Option<RequestID>,
    #[serde(rename = "type")]
    pub transaction_type: Option<TransactionType>,
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
    pub reason: Option<LimitOrderReason>,
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
    pub reject_reason: Option<TransactionRejectReason>,
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
    pub request_id: Option<RequestID>,
    #[serde(rename = "type")]
    pub transaction_type: Option<TransactionType>,
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
    pub reason: Option<StopOrderReason>,
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
    pub request_id: Option<RequestID>,
    #[serde(rename = "type")]
    pub transaction_type: Option<TransactionType>,
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
    pub reason: Option<StopOrderReason>,
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
    pub reject_reason: Option<TransactionRejectReason>,
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
    pub request_id: Option<RequestID>,
    #[serde(rename = "type")]
    pub transaction_type: Option<TransactionType>,
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
    pub reason: Option<MarketIfTouchedOrderReason>,
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
    pub request_id: Option<RequestID>,
    #[serde(rename = "type")]
    pub transaction_type: Option<TransactionType>,
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
    pub reason: Option<MarketIfTouchedOrderReason>,
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
    pub reject_reason: Option<TransactionRejectReason>,
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
    pub request_id: Option<RequestID>,
    #[serde(rename = "type")]
    pub transaction_type: Option<TransactionType>,
    #[serde(rename = "tradeID")]
    pub trade_id: Option<TradeID>,
    #[serde(rename = "clientTradeID")]
    pub client_trade_id: Option<ClientID>,
    pub price: Option<PriceValue>,
    #[serde(rename = "timeInForce")]
    pub time_in_force: Option<TimeInForce>,
    #[serde(rename = "gtdTime")]
    pub gtd_time: Option<DateTime<Utc>>,
    #[serde(rename = "triggerCondition")]
    pub trigger_condition: Option<OrderTriggerCondition>,
    pub reason: Option<TakeProfitOrderReason>,
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
    pub request_id: Option<RequestID>,
    #[serde(rename = "type")]
    pub transaction_type: Option<TransactionType>,
    #[serde(rename = "tradeID")]
    pub trade_id: Option<TradeID>,
    #[serde(rename = "clientTradeID")]
    pub client_trade_id: Option<ClientID>,
    pub price: Option<PriceValue>,
    #[serde(rename = "timeInForce")]
    pub time_in_force: Option<TimeInForce>,
    #[serde(rename = "gtdTime")]
    pub gtd_time: Option<DateTime<Utc>>,
    #[serde(rename = "triggerCondition")]
    pub trigger_condition: Option<OrderTriggerCondition>,
    pub reason: Option<TakeProfitOrderReason>,
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
    pub reject_reason: Option<TransactionRejectReason>,
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
    pub request_id: Option<RequestID>,
    #[serde(rename = "type")]
    pub transaction_type: Option<TransactionType>,
    #[serde(rename = "tradeID")]
    pub trade_id: Option<TradeID>,
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
    pub reason: Option<StopLossOrderReason>,
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
    pub request_id: Option<RequestID>,
    #[serde(rename = "type")]
    pub transaction_type: Option<TransactionType>,
    #[serde(rename = "tradeID")]
    pub trade_id: Option<TradeID>,
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
    pub reason: Option<StopLossOrderReason>,
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
    pub reject_reason: Option<TransactionRejectReason>,
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
    pub request_id: Option<RequestID>,
    #[serde(rename = "type")]
    pub transaction_type: Option<TransactionType>,
    #[serde(rename = "tradeID")]
    pub trade_id: Option<TradeID>,
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
    pub reason: Option<GuaranteedStopLossOrderReason>,
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
    pub request_id: Option<RequestID>,
    #[serde(rename = "type")]
    pub transaction_type: Option<TransactionType>,
    #[serde(rename = "tradeID")]
    pub trade_id: Option<TradeID>,
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
    pub reason: Option<GuaranteedStopLossOrderReason>,
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
    pub reject_reason: Option<TransactionRejectReason>,
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
    pub request_id: Option<RequestID>,
    #[serde(rename = "type")]
    pub transaction_type: Option<TransactionType>,
    #[serde(rename = "tradeID")]
    pub trade_id: Option<TradeID>,
    #[serde(rename = "clientTradeID")]
    pub client_trade_id: Option<ClientID>,
    pub distance: Option<DecimalNumber>,
    #[serde(rename = "timeInForce")]
    pub time_in_force: Option<TimeInForce>,
    #[serde(rename = "gtdTime")]
    pub gtd_time: Option<DateTime<Utc>>,
    #[serde(rename = "triggerCondition")]
    pub trigger_condition: Option<OrderTriggerCondition>,
    pub reason: Option<TrailingStopLossOrderReason>,
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
    pub request_id: Option<RequestID>,
    #[serde(rename = "type")]
    pub transaction_type: Option<TransactionType>,
    #[serde(rename = "tradeID")]
    pub trade_id: Option<TradeID>,
    #[serde(rename = "clientTradeID")]
    pub client_trade_id: Option<ClientID>,
    pub distance: Option<DecimalNumber>,
    #[serde(rename = "timeInForce")]
    pub time_in_force: Option<TimeInForce>,
    #[serde(rename = "gtdTime")]
    pub gtd_time: Option<DateTime<Utc>>,
    #[serde(rename = "triggerCondition")]
    pub trigger_condition: Option<OrderTriggerCondition>,
    pub reason: Option<TrailingStopLossOrderReason>,
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
    pub reject_reason: Option<TransactionRejectReason>,
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
    pub request_id: Option<RequestID>,
    #[serde(rename = "type")]
    pub transaction_type: Option<TransactionType>,
    #[serde(rename = "orderID")]
    pub order_id: Option<OrderID>,
    #[serde(rename = "clientOrderID")]
    pub client_order_id: Option<ClientID>,
    pub instrument: Option<InstrumentName>,
    pub units: Option<DecimalNumber>,
    #[serde(rename = "gainQuoteHomeConversionFactor")]
    pub gain_quote_home_conversion_factor: Option<DecimalNumber>,
    #[serde(rename = "lossQuoteHomeConversionFactor")]
    pub loss_quote_home_conversion_factor: Option<DecimalNumber>,
    #[serde(rename = "homeConversionFactors")]
    pub home_conversion_factors: Option<HomeConversionFactors>,
    pub price: Option<PriceValue>,
    #[serde(rename = "fullVWAP")]
    pub full_vwap: Option<PriceValue>,
    #[serde(rename = "fullPrice")]
    pub full_price: Option<PriceValue>,
    pub reason: Option<OrderFillReason>,
    pub pl: Option<AccountUnits>,
    pub financing: Option<AccountUnits>,
    pub commission: Option<AccountUnits>,
    #[serde(rename = "guaranteedExecutionFee")]
    pub guaranteed_execution_fee: Option<AccountUnits>,
    #[serde(rename = "halfSpreadCost")]
    pub half_spread_cost: Option<AccountUnits>,
    #[serde(rename = "accountBalance")]
    pub account_balance: Option<AccountUnits>,
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
    pub request_id: Option<RequestID>,
    #[serde(rename = "type")]
    pub transaction_type: Option<TransactionType>,
    #[serde(rename = "orderID")]
    pub order_id: Option<OrderID>,
    #[serde(rename = "clientOrderID")]
    pub client_order_id: Option<ClientID>,
    pub reason: Option<OrderCancelReason>,
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
    pub request_id: Option<RequestID>,
    #[serde(rename = "type")]
    pub transaction_type: Option<TransactionType>,
    #[serde(rename = "orderID")]
    pub order_id: Option<OrderID>,
    #[serde(rename = "clientOrderID")]
    pub client_order_id: Option<ClientID>,
    #[serde(rename = "rejectReason")]
    pub reject_reason: Option<TransactionRejectReason>,
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
    pub request_id: Option<RequestID>,
    #[serde(rename = "type")]
    pub transaction_type: Option<TransactionType>,
    #[serde(rename = "orderID")]
    pub order_id: Option<OrderID>,
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
    pub request_id: Option<RequestID>,
    #[serde(rename = "type")]
    pub transaction_type: Option<TransactionType>,
    #[serde(rename = "orderID")]
    pub order_id: Option<OrderID>,
    #[serde(rename = "clientOrderID")]
    pub client_order_id: Option<ClientID>,
    #[serde(rename = "clientExtensionsModify")]
    pub client_extensions_modify: Option<ClientExtensions>,
    #[serde(rename = "tradeClientExtensionsModify")]
    pub trade_client_extensions_modify: Option<ClientExtensions>,
    #[serde(rename = "rejectReason")]
    pub reject_reason: Option<TransactionRejectReason>,
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
    pub request_id: Option<RequestID>,
    #[serde(rename = "type")]
    pub transaction_type: Option<TransactionType>,
    #[serde(rename = "tradeID")]
    pub trade_id: Option<TradeID>,
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
    pub request_id: Option<RequestID>,
    #[serde(rename = "type")]
    pub transaction_type: Option<TransactionType>,
    #[serde(rename = "tradeID")]
    pub trade_id: Option<TradeID>,
    #[serde(rename = "clientTradeID")]
    pub client_trade_id: Option<ClientID>,
    #[serde(rename = "tradeClientExtensionsModify")]
    pub trade_client_extensions_modify: Option<ClientExtensions>,
    #[serde(rename = "rejectReason")]
    pub reject_reason: Option<TransactionRejectReason>,
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
    pub request_id: Option<RequestID>,
    #[serde(rename = "type")]
    pub transaction_type: Option<TransactionType>,
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
    pub request_id: Option<RequestID>,
    #[serde(rename = "type")]
    pub transaction_type: Option<TransactionType>,
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
    pub request_id: Option<RequestID>,
    #[serde(rename = "type")]
    pub transaction_type: Option<TransactionType>,
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
    pub request_id: Option<RequestID>,
    #[serde(rename = "type")]
    pub transaction_type: Option<TransactionType>,
    pub reason: Option<MarketOrderReason>,
    #[serde(rename = "tradeIDs")]
    pub trade_ids: Option<Vec<TradeID>>,
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
    pub request_id: Option<RequestID>,
    #[serde(rename = "type")]
    pub transaction_type: Option<TransactionType>,
    pub financing: Option<AccountUnits>,
    #[serde(rename = "accountBalance")]
    pub account_balance: Option<AccountUnits>,
    #[serde(rename = "accountFinancingMode")]
    pub account_financing_mode: Option<AccountFinancingMode>,
    #[serde(rename = "positionFinancings")]
    pub position_financings: Option<Vec<PositionFinancing>>,
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
    pub request_id: Option<RequestID>,
    #[serde(rename = "type")]
    pub transaction_type: Option<TransactionType>,
    pub instrument: Option<InstrumentName>,
    #[serde(rename = "dividendAdjustment")]
    pub dividend_adjustment: Option<AccountUnits>,
    #[serde(rename = "quoteDividendAdjustment")]
    pub quote_dividend_adjustment: Option<AccountUnits>,
    #[serde(rename = "homeConversionFactors")]
    pub home_conversion_factors: Option<HomeConversionFactors>,
    #[serde(rename = "accountBalance")]
    pub account_balance: Option<AccountUnits>,
    #[serde(rename = "openTradeDividendAdjustments")]
    pub open_trade_dividend_adjustments: Option<Vec<OpenTradeDividendAdjustment>>,
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
    pub request_id: Option<RequestID>,
    #[serde(rename = "type")]
    pub transaction_type: Option<TransactionType>,
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

#[derive(Debug, Serialize, Deserialize)]
pub struct ListTransactionsResponse {
    pub count: Option<i64>,
    pub from: Option<DateTime<Utc>>,
    pub to: Option<DateTime<Utc>>,
    #[serde(rename = "pageSize")]
    pub page_size: Option<i64>,
    pub pages: Option<Vec<String>>,
}
