use super::*;

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
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum OrderRequest {
    Market(MarketOrderRequest),
    Limit(LimitOrderRequest),
    Stop(StopOrderRequest),
    MarketIfTouched(MarketIfTouchedOrderRequest),
    TakeProfit(TakeProfitOrderRequest),
    StopLoss(StopLossOrderRequest),
    GuaranteedStopLoss(GuaranteedStopLossOrderRequest),
    TrailingStopLoss(TrailingStopLossOrderRequest),
}

/// Request body for creating a market order.
///
/// Build with [`MarketOrderRequest::new`] then pass to [`OrderService::create`]
/// wrapped in [`OrderRequest::Market`].
///
/// Defaults: `time_in_force = FOK`, `position_fill = Default`.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MarketOrderRequest {
    /// The instrument to trade.
    pub instrument: InstrumentName,
    /// Units to trade. Positive = buy (long), negative = sell (short).
    pub units: DecimalNumber,
    /// Time-in-force (`FOK` or `IOC` for market orders).
    pub time_in_force: TimeInForce,
    /// Worst acceptable fill price. Omitted if `None`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub price_bound: Option<PriceValue>,
    /// How the order interacts with an existing position.
    pub position_fill: OrderPositionFill,
    /// Client metadata to attach to the order. Omitted if `None`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_extensions: Option<ClientExtensions>,
    /// Take-profit to attach to any resulting trade. Omitted if `None`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub take_profit_on_fill: Option<TakeProfitDetails>,
    /// Stop-loss to attach to any resulting trade. Omitted if `None`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop_loss_on_fill: Option<StopLossDetails>,
    /// Guaranteed stop-loss to attach to any resulting trade. Omitted if `None`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub guaranteed_stop_loss_on_fill: Option<GuaranteedStopLossDetails>,
    /// Trailing stop-loss to attach to any resulting trade. Omitted if `None`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trailing_stop_loss_on_fill: Option<TrailingStopLossDetails>,
    /// Client extensions to apply to the resulting trade. Omitted if `None`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trade_client_extensions: Option<ClientExtensions>,
}

impl MarketOrderRequest {
    /// Creates a new market order request for `instrument` with the given `units`.
    ///
    /// Defaults: `time_in_force = FOK`, `position_fill = Default`.
    pub fn new(instrument: InstrumentName, units: DecimalNumber) -> MarketOrderRequest {
        MarketOrderRequest {
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
#[serde(rename_all = "camelCase")]
pub struct LimitOrderRequest {
    /// The instrument to trade.
    pub instrument: InstrumentName,
    /// Units to trade. Positive = buy (long), negative = sell (short).
    pub units: DecimalNumber,
    /// The limit price. The order fills only at this price or better.
    pub price: PriceValue,
    /// How long the order remains active.
    pub time_in_force: TimeInForce,
    /// Expiry timestamp used when `time_in_force` is changed to `GTD` via [`Self::gtd`].
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gtd_time: Option<DateTime<Utc>>,
    /// How the order interacts with an existing position.
    pub position_fill: OrderPositionFill,
    /// Which price stream triggers the order.
    pub trigger_condition: OrderTriggerCondition,
    /// Client metadata to attach to the order. Omitted if `None`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_extensions: Option<ClientExtensions>,
    /// Take-profit to attach to any resulting trade. Omitted if `None`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub take_profit_on_fill: Option<TakeProfitDetails>,
    /// Stop-loss to attach to any resulting trade. Omitted if `None`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop_loss_on_fill: Option<StopLossDetails>,
    /// Guaranteed stop-loss to attach to any resulting trade. Omitted if `None`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub guaranteed_stop_loss_on_fill: Option<GuaranteedStopLossDetails>,
    /// Trailing stop-loss to attach to any resulting trade. Omitted if `None`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trailing_stop_loss_on_fill: Option<TrailingStopLossDetails>,
    /// Client extensions to apply to the resulting trade. Omitted if `None`.
    #[serde(skip_serializing_if = "Option::is_none")]
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
#[serde(rename_all = "camelCase")]
pub struct StopOrderRequest {
    /// The instrument to trade.
    pub instrument: InstrumentName,
    /// Units to trade. Positive = buy (long), negative = sell (short).
    pub units: DecimalNumber,
    /// The stop trigger price.
    pub price: PriceValue,
    /// Worst acceptable fill price after the stop triggers. Omitted if `None`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub price_bound: Option<PriceValue>,
    /// How long the order remains active.
    pub time_in_force: TimeInForce,
    /// Expiry timestamp used when `time_in_force` is `GTD`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gtd_time: Option<DateTime<Utc>>,
    /// How the order interacts with an existing position.
    pub position_fill: OrderPositionFill,
    /// Which price stream triggers the order.
    pub trigger_condition: OrderTriggerCondition,
    /// Client metadata to attach to the order. Omitted if `None`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_extensions: Option<ClientExtensions>,
    /// Take-profit to attach to any resulting trade. Omitted if `None`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub take_profit_on_fill: Option<TakeProfitDetails>,
    /// Stop-loss to attach to any resulting trade. Omitted if `None`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop_loss_on_fill: Option<StopLossDetails>,
    /// Guaranteed stop-loss to attach to any resulting trade. Omitted if `None`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub guaranteed_stop_loss_on_fill: Option<GuaranteedStopLossDetails>,
    /// Trailing stop-loss to attach to any resulting trade. Omitted if `None`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trailing_stop_loss_on_fill: Option<TrailingStopLossDetails>,
    /// Client extensions to apply to the resulting trade. Omitted if `None`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trade_client_extensions: Option<ClientExtensions>,
}

impl StopOrderRequest {
    /// Creates a new stop order request.
    ///
    /// Defaults: `time_in_force = GTC`, `position_fill = Default`,
    /// `trigger_condition = Default`.
    pub fn new(instrument: InstrumentName, units: DecimalNumber, price: PriceValue) -> Self {
        StopOrderRequest {
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
#[serde(rename_all = "camelCase")]
pub struct MarketIfTouchedOrderRequest {
    /// The instrument to trade.
    pub instrument: InstrumentName,
    /// Units to trade. Positive = buy (long), negative = sell (short).
    pub units: DecimalNumber,
    /// The trigger price. When touched, the order converts to a market order.
    pub price: PriceValue,
    /// Worst acceptable fill price after triggering. Omitted if `None`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub price_bound: Option<PriceValue>,
    /// How long the order remains active.
    pub time_in_force: TimeInForce,
    /// Expiry timestamp used when `time_in_force` is `GTD`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gtd_time: Option<DateTime<Utc>>,
    /// How the order interacts with an existing position.
    pub position_fill: OrderPositionFill,
    /// Which price stream triggers the order.
    pub trigger_condition: OrderTriggerCondition,
    /// Client metadata to attach to the order. Omitted if `None`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_extensions: Option<ClientExtensions>,
    /// Take-profit to attach to any resulting trade. Omitted if `None`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub take_profit_on_fill: Option<TakeProfitDetails>,
    /// Stop-loss to attach to any resulting trade. Omitted if `None`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop_loss_on_fill: Option<StopLossDetails>,
    /// Guaranteed stop-loss to attach to any resulting trade. Omitted if `None`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub guaranteed_stop_loss_on_fill: Option<GuaranteedStopLossDetails>,
    /// Trailing stop-loss to attach to any resulting trade. Omitted if `None`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trailing_stop_loss_on_fill: Option<TrailingStopLossDetails>,
    /// Client extensions to apply to the resulting trade. Omitted if `None`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trade_client_extensions: Option<ClientExtensions>,
}

impl MarketIfTouchedOrderRequest {
    /// Creates a new market-if-touched order request.
    ///
    /// Defaults: `time_in_force = GTC`, `position_fill = Default`,
    /// `trigger_condition = Default`.
    pub fn new(instrument: InstrumentName, units: DecimalNumber, price: PriceValue) -> Self {
        MarketIfTouchedOrderRequest {
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
#[serde(rename_all = "camelCase")]
pub struct TakeProfitOrderRequest {
    /// The ID of the trade to attach the take-profit to.
    #[serde(rename = "tradeID")]
    pub trade_id: TradeID,
    /// Client-provided trade ID, if any. Omitted if `None`.
    #[serde(rename = "clientTradeID", skip_serializing_if = "Option::is_none")]
    pub client_trade_id: Option<ClientID>,
    /// The price at which the trade will be closed to take profit.
    pub price: PriceValue,
    /// How long the order remains active.
    pub time_in_force: TimeInForce,
    /// Expiry timestamp used when `time_in_force` is `GTD`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gtd_time: Option<DateTime<Utc>>,
    /// Which price stream triggers the order.
    pub trigger_condition: OrderTriggerCondition,
    /// Client metadata to attach to the order. Omitted if `None`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_extensions: Option<ClientExtensions>,
}

impl TakeProfitOrderRequest {
    /// Creates a new take-profit order request for `trade_id` at the given `price`.
    ///
    /// Defaults: `time_in_force = GTC`, `trigger_condition = Default`.
    pub fn new(trade_id: TradeID, price: PriceValue) -> Self {
        TakeProfitOrderRequest {
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
#[serde(rename_all = "camelCase")]
pub struct StopLossOrderRequest {
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
    pub time_in_force: TimeInForce,
    /// Expiry timestamp used when `time_in_force` is `GTD`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gtd_time: Option<DateTime<Utc>>,
    /// Which price stream triggers the order.
    pub trigger_condition: OrderTriggerCondition,
    /// Client metadata to attach to the order. Omitted if `None`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_extensions: Option<ClientExtensions>,
}

impl StopLossOrderRequest {
    /// Creates a new stop-loss order request for `trade_id` at the given `price`.
    ///
    /// Defaults: `time_in_force = GTC`, `trigger_condition = Default`.
    pub fn new(trade_id: TradeID, price: PriceValue) -> Self {
        StopLossOrderRequest {
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
#[serde(rename_all = "camelCase")]
pub struct GuaranteedStopLossOrderRequest {
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
    pub time_in_force: TimeInForce,
    /// Expiry timestamp used when `time_in_force` is `GTD`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gtd_time: Option<DateTime<Utc>>,
    /// Which price stream triggers the order.
    pub trigger_condition: OrderTriggerCondition,
    /// Client metadata to attach to the order. Omitted if `None`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_extensions: Option<ClientExtensions>,
}

impl GuaranteedStopLossOrderRequest {
    /// Creates a new guaranteed stop-loss order request for `trade_id` at `price`.
    ///
    /// Defaults: `time_in_force = GTC`, `trigger_condition = Default`.
    pub fn new(trade_id: TradeID, price: PriceValue) -> Self {
        GuaranteedStopLossOrderRequest {
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
#[serde(rename_all = "camelCase")]
pub struct TrailingStopLossOrderRequest {
    /// The ID of the trade to attach the trailing stop-loss to.
    #[serde(rename = "tradeID")]
    pub trade_id: TradeID,
    /// Client-provided trade ID, if any. Omitted if `None`.
    #[serde(rename = "clientTradeID", skip_serializing_if = "Option::is_none")]
    pub client_trade_id: Option<ClientID>,
    /// The trailing distance kept between market price and the stop.
    pub distance: DecimalNumber,
    /// How long the order remains active.
    pub time_in_force: TimeInForce,
    /// Expiry timestamp used when `time_in_force` is `GTD`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gtd_time: Option<DateTime<Utc>>,
    /// Which price stream triggers the order.
    pub trigger_condition: OrderTriggerCondition,
    /// Client metadata to attach to the order. Omitted if `None`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_extensions: Option<ClientExtensions>,
}

impl TrailingStopLossOrderRequest {
    /// Creates a new trailing stop-loss order request for `trade_id` with the
    /// given trailing `distance`.
    ///
    /// Defaults: `time_in_force = GTC`, `trigger_condition = Default`.
    pub fn new(trade_id: TradeID, distance: DecimalNumber) -> Self {
        TrailingStopLossOrderRequest {
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

/// Builder for a `GET /v3/accounts/{accountID}/orders` request.
///
/// Construct via [`ListOrdersRequest::new`], configure with the builder
/// methods, then pass to [`OrderService::list`].
#[derive(Default)]
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
        Self::default()
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
        if !self.ids.is_empty() {
            url.query_pairs_mut().append_pair(
                "ids",
                self.ids
                    .iter()
                    .map(|id| id.to_string())
                    .collect::<Vec<String>>()
                    .join(",")
                    .as_str(),
            );
        }
        if let Some(state) = &self.state {
            url.query_pairs_mut()
                .append_pair("state", state.to_string().as_str());
        }
        if let Some(instrument) = &self.instrument {
            url.query_pairs_mut()
                .append_pair("instrument", instrument.as_str());
        }
        if let Some(count) = self.count {
            url.query_pairs_mut()
                .append_pair("count", count.to_string().as_str());
        }
        if let Some(id) = self.before_id.as_ref() {
            url.query_pairs_mut().append_pair("beforeID", id.as_str());
        }
    }
}

/// JSON request body sent to `POST /v3/accounts/{accountID}/orders`.
#[derive(Debug, Serialize)]
pub struct CreateOrderRequest {
    /// The order to create.
    pub order: OrderRequest,
}

/// Request body for
/// `PUT /v3/accounts/{accountID}/orders/{orderSpecifier}/clientExtensions`.
///
/// At least one of `client_extensions` or `trade_client_extensions` must be set.
#[derive(Debug, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateOrderClientExtensionsRequest {
    /// New client extensions for the order itself. Omitted if `None`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_extensions: Option<ClientExtensions>,
    /// New client extensions for the trade that would result from filling this
    /// order. Omitted if `None`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trade_client_extensions: Option<ClientExtensions>,
}

impl UpdateOrderClientExtensionsRequest {
    /// Creates a new request with no extensions set.
    ///
    /// Call [`client_extensions`](Self::client_extensions) and/or
    /// [`trade_client_extensions`](Self::trade_client_extensions) before passing
    /// this to [`OrderService::update_client_extensions`].
    pub fn new() -> Self {
        Self::default()
    }

    request_option_setter!(client_extensions, ClientExtensions);
    request_option_setter!(trade_client_extensions, ClientExtensions);
}
