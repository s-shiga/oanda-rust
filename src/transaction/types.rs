use super::*;

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
    #[serde(rename = "RESET_RESETTABLE_PL")]
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
    DividendAdjustmentReplaced,
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
    GuaranteedStopLossOnFillGtdTimestampInPast,
    GuaranteedStopLossOnFillLoss,
    GuaranteedStopLossOnFillPriceDistanceMaximumExceeded,
    GuaranteedStopLossOnFillRequired,
    GuaranteedStopLossOnFillNotAllowed,
    GuaranteedStopLossOnFillMinimumDistanceNotMet,
    GuaranteedStopLossOnFillLevelRestrictionVolumeExceeded,
    GuaranteedStopLossOnFillLevelRestrictionPriceRangeExceeded,
    GuaranteedStopLossOnFillHedgingNotAllowed,
    GuaranteedStopLossOnFillTimeInForceInvalid,
    GuaranteedStopLossOnFillTriggerConditionInvalid,
    TakeProfitOnFillPriceDistanceMaximumExceeded,
    TrailingStopLossOnFillGtdTimestampInPast,
    ClientTradeIdAlreadyExists,
    PositionCloseoutFailed,
    OpenTradesAllowedExceeded,
    PendingOrdersAllowedExceeded,
    TakeProfitOnFillClientOrderIdAlreadyExists,
    StopLossOnFillClientOrderIdAlreadyExists,
    GuaranteedStopLossOnFillClientOrderIdAlreadyExists,
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
    GuaranteedStopLossOnFillBidHalted,
    GuaranteedStopLossOnFillAskHalted,
    FifoViolationSafeguardViolation,
    FifoViolationSafeguardPartialCloseViolation,
    OrdersOnFillRmoMutualExclusivityMutuallyExclusiveViolation,
    /// A value not known to this version of the crate. OANDA adds new
    /// values over time; this keeps such transactions decodable.
    #[serde(other)]
    Unknown,
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
/// Used with [`ListTransactionsRequest`], [`GetTransactionsByIDRangeRequest`], and
/// [`GetTransactionsBySinceIDRequest`] to narrow results to a subset of transaction
/// types. Variants like [`Order`](Self::Order) and [`Funding`](Self::Funding)
/// aggregate multiple [`TransactionType`] values.
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
    GuaranteedStopLossOrder,
    GuaranteedStopLossOrderReject,
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
    #[serde(rename = "RESET_RESETTABLE_PL")]
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
    OrdersOnFillRmoMutualExclusivityMutuallyExclusiveViolation,
    OrdersOnFillRmoMutualExclusivityGsloExcludesOthersViolation,
    TakeProfitOrderAlreadyExists,
    TakeProfitOrderWouldViolateFifoViolationSafeguard,
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
    StopLossOrderWouldViolateFifoViolationSafeguard,
    StopLossOrderRmoMutualExclusivityMutuallyExclusiveViolation,
    StopLossOrderRmoMutualExclusivityGsloExcludesOthersViolation,
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
    GuaranteedStopLossOrderAlreadyExists,
    GuaranteedStopLossOrderRequired,
    GuaranteedStopLossOrderPriceWithinSpread,
    GuaranteedStopLossOrderNotAllowed,
    GuaranteedStopLossOrderHaltedCreateViolation,
    GuaranteedStopLossOrderCreateViolation,
    GuaranteedStopLossOrderHaltedTightenViolation,
    GuaranteedStopLossOrderTightenViolation,
    GuaranteedStopLossOrderHedgingNotAllowed,
    GuaranteedStopLossOrderMinimumDistanceNotMet,
    GuaranteedStopLossOrderNotCancelable,
    GuaranteedStopLossOrderHaltedNotCancelable,
    GuaranteedStopLossOrderNotReplaceable,
    GuaranteedStopLossOrderHaltedNotReplaceable,
    GuaranteedStopLossOrderLevelRestrictionVolumeExceeded,
    GuaranteedStopLossOrderLevelRestrictionPriceRangeExceeded,
    GuaranteedStopLossOrderPriceAndDistanceBothSpecified,
    GuaranteedStopLossOrderPriceAndDistanceBothMissing,
    GuaranteedStopLossOrderWouldViolateFifoViolationSafeguard,
    GuaranteedStopLossOrderRmoMutualExclusivityMutuallyExclusiveViolation,
    GuaranteedStopLossOrderRmoMutualExclusivityGsloExcludesOthersViolation,
    GuaranteedStopLossOnFillRequiredForPendingOrder,
    GuaranteedStopLossOnFillNotAllowed,
    GuaranteedStopLossOnFillRequired,
    GuaranteedStopLossOnFillPriceMissing,
    GuaranteedStopLossOnFillPriceInvalid,
    GuaranteedStopLossOnFillPricePrecisionExceeded,
    GuaranteedStopLossOnFillMinimumDistanceNotMet,
    GuaranteedStopLossOnFillLevelRestrictionVolumeExceeded,
    GuaranteedStopLossOnFillLevelRestrictionPriceRangeExceeded,
    GuaranteedStopLossOnFillDistanceInvalid,
    GuaranteedStopLossOnFillPriceDistanceMaximumExceeded,
    GuaranteedStopLossOnFillDistancePrecisionExceeded,
    GuaranteedStopLossOnFillPriceAndDistanceBothSpecified,
    GuaranteedStopLossOnFillPriceAndDistanceBothMissing,
    GuaranteedStopLossOnFillTimeInForceMissing,
    GuaranteedStopLossOnFillTimeInForceInvalid,
    GuaranteedStopLossOnFillGtdTimestampMissing,
    GuaranteedStopLossOnFillGtdTimestampInPast,
    GuaranteedStopLossOnFillClientOrderIdInvalid,
    GuaranteedStopLossOnFillClientOrderTagInvalid,
    GuaranteedStopLossOnFillClientOrderCommentInvalid,
    GuaranteedStopLossOnFillTriggerConditionMissing,
    GuaranteedStopLossOnFillTriggerConditionInvalid,
    TrailingStopLossOrderAlreadyExists,
    TrailingStopLossOrderWouldViolateFifoViolationSafeguard,
    TrailingStopLossOrderRmoMutualExclusivityMutuallyExclusiveViolation,
    TrailingStopLossOrderRmoMutualExclusivityGsloExcludesOthersViolation,
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
    OcaOrderIdsStopLossNotAllowed,
    ClientExtensionsDataMissing,
    ReplacingOrderInvalid,
    ReplacingTradeIdInvalid,
    OrderCancelWouldTriggerCloseout,
    /// A value not known to this version of the crate. OANDA adds new
    /// values over time; this keeps such transactions decodable.
    #[serde(other)]
    Unknown,
}
