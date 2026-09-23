use super::*;

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
/// Consumed as part of [`TransactionStreamItem::Heartbeat`].
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
