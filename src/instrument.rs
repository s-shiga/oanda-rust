use crate::client::Client;
use crate::errors::{APIError, CommonErrorResponse, ErrorResponse};
use crate::pricing::{PriceValue, PricingComponent};
use crate::primitives::{DecimalNumber, Tag};
use crate::transaction::TransactionID;
use chrono::{DateTime, Local};
use reqwest::{Request, StatusCode};
use serde::{Deserialize, Serialize};
use std::ops::Not;
use strum_macros::{Display, EnumString};
use url::Url;
use crate::handle_response;

/// An instrument identifier string in `"BASE_QUOTE"` format (e.g. `"EUR_USD"`).
pub type InstrumentName = String;

/// The asset class of a tradeable instrument.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum InstrumentType {
    /// A foreign-exchange currency pair.
    Currency,
    /// A contract for difference on an index, commodity, or other underlying.
    CFD,
    /// A precious metal spot instrument.
    Metal,
}

/// Commission structure applied to trades on a specific instrument.
#[derive(Debug, Serialize, Deserialize)]
pub struct InstrumentCommission {
    /// Commission charged per `units_traded` units, expressed in home currency.
    pub commission: DecimalNumber,
    /// The number of units of the instrument to which `commission` applies.
    #[serde(rename = "unitsTraded")]
    pub units_traded: DecimalNumber,
    /// The minimum commission charged per trade, regardless of size.
    #[serde(rename = "minimumCommission")]
    pub min_commission: DecimalNumber,
}

/// Whether Guaranteed Stop Loss Orders (GSLOs) are available for a specific instrument.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum GuaranteedStopLossOrderModeForInstrument {
    /// GSLOs are not available for this instrument.
    Disabled,
    /// GSLOs are available but not required for this instrument.
    Allowed,
    /// Every trade on this instrument must have a GSLO attached.
    Required,
}

/// Restricts where a Guaranteed Stop Loss Order may be placed relative to the
/// current price, based on overall position volume.
#[derive(Debug, Serialize, Deserialize)]
pub struct GuaranteedStopLossOrderLevelRestriction {
    /// Total position volume (in units) above which the price range restriction applies.
    pub volume: DecimalNumber,
    /// Minimum distance (in price units) from the current price that a GSLO must maintain
    /// when the position volume exceeds `volume`.
    #[serde(rename = "priceRange")]
    pub price_range: DecimalNumber,
}

/// Day of the week, used when configuring financing schedules.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum DayOfWeek {
    Sunday,
    Monday,
    Tuesday,
    Wednesday,
    Thursday,
    Friday,
    Saturday,
}

/// Specifies how many days' worth of financing are charged on a given day of the week.
///
/// Some instruments charge multiple days of financing on a single day (e.g. triple
/// swap on Wednesdays for most spot FX pairs).
#[derive(Debug, Serialize, Deserialize)]
pub struct FinancingDayOfWeek {
    /// The day on which the financing charge is applied.
    #[serde(rename = "dayOfWeek")]
    pub day_of_week: DayOfWeek,
    /// Number of days of financing charged on this day (typically 1, but 3 on rollover days).
    #[serde(rename = "daysCharged")]
    pub days_charged: i8,
}

/// Financing (swap/rollover) rates applied to long and short positions on an instrument.
#[derive(Debug, Serialize, Deserialize)]
pub struct InstrumentFinancing {
    /// Daily financing rate applied to long positions (expressed as a decimal fraction).
    #[serde(rename = "longRate")]
    pub long_rate: DecimalNumber,
    /// Daily financing rate applied to short positions (expressed as a decimal fraction).
    #[serde(rename = "shortRate")]
    pub short_rate: DecimalNumber,
}

/// Full specification of a tradeable instrument, including margin, precision,
/// financing, and GSLO rules.
///
/// Returned as part of [`ListInstrumentsResponse`] by
/// `GET /v3/accounts/{accountID}/instruments`.
#[derive(Debug, Serialize, Deserialize)]
pub struct Instrument {
    /// The instrument name in `"BASE_QUOTE"` format (e.g. `"EUR_USD"`).
    pub name: String,
    /// The asset class of this instrument.
    #[serde(rename = "type")]
    pub instrument_type: InstrumentType,
    /// A human-readable display name (e.g. `"EUR/USD"`).
    #[serde(rename = "displayName")]
    pub display_name: String,
    /// The exponent of the pip value: a pip is `10^pip_location` price units.
    /// For example, `-4` means a pip is 0.0001.
    #[serde(rename = "pipLocation")]
    pub pip_location: i8,
    /// Number of decimal places to display for prices of this instrument.
    #[serde(rename = "displayPrecision")]
    pub display_precision: i8,
    /// Number of decimal places to which trade unit sizes must be rounded.
    #[serde(rename = "tradeUnitsPrecision")]
    pub trade_units_precision: i8,
    /// Minimum number of units that can be traded in a single order.
    #[serde(rename = "minimumTradeSize")]
    pub min_trade_size: DecimalNumber,
    /// Maximum allowed trailing stop distance, in price units.
    #[serde(rename = "maximumTrailingStopDistance")]
    pub max_trailing_stop_distance: DecimalNumber,
    /// Minimum allowed distance for a Guaranteed Stop Loss Order, in price units.
    /// Only present when GSLOs are `Allowed` or `Required` for this instrument.
    #[serde(
        rename = "minimumGuaranteedStopLossDistance",
        skip_serializing_if = "Option::is_none"
    )]
    pub min_guaranteed_stop_loss_distance: Option<DecimalNumber>,
    /// Minimum allowed trailing stop distance, in price units.
    #[serde(rename = "minimumTrailingStopDistance")]
    pub min_trailing_stop_distance: DecimalNumber,
    /// Maximum aggregate position size allowed for this instrument, in units.
    /// `0` means no limit.
    #[serde(rename = "maximumPositionSize")]
    pub max_position_size: DecimalNumber,
    /// Maximum number of units that can be specified in a single order.
    #[serde(rename = "maximumOrderUnits")]
    pub max_order_units: DecimalNumber,
    /// Margin rate expressed as a decimal (e.g. `0.05` for 5 % margin / 20:1 leverage).
    #[serde(rename = "marginRate")]
    pub margin_rate: DecimalNumber,
    /// Commission structure for this instrument, if applicable.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub commission: Option<InstrumentCommission>,
    /// Whether GSLOs are disabled, allowed, or required for this instrument.
    #[serde(rename = "guaranteedStopLossOrderMode")]
    pub guaranteed_stop_loss_order_mode: GuaranteedStopLossOrderModeForInstrument,
    /// Additional premium (in price units) charged when a GSLO is executed,
    /// above the normal spread. Only present when GSLOs are available.
    #[serde(
        rename = "guaranteedStopLossOrderExecutionPremium",
        skip_serializing_if = "Option::is_none"
    )]
    pub guaranteed_stop_loss_order_execution_premium: Option<DecimalNumber>,
    /// Volume-based restriction on where GSLOs may be placed. Only present
    /// when GSLOs are available.
    #[serde(
        rename = "guaranteedStopLossOrderLevelRestriction",
        skip_serializing_if = "Option::is_none"
    )]
    pub guaranteed_stop_loss_order_level_restriction:
        Option<GuaranteedStopLossOrderLevelRestriction>,
    /// Financing rates and schedule for this instrument.
    pub financing: InstrumentFinancing,
    /// Arbitrary metadata tags associated with this instrument.
    pub tags: Vec<Tag>,
}

/// Time granularity (bar size) of a candlestick series.
///
/// Variants follow the OANDA naming convention:
/// `S` = seconds, `M` = minutes, `H` = hours, `D` = day, `W` = week, `M` (last) = month.
#[derive(Debug, Serialize, Deserialize, Display, EnumString)]
pub enum CandlestickGranularity {
    /// 5-second bars.
    S5,
    /// 10-second bars.
    S10,
    /// 15-second bars.
    S15,
    /// 30-second bars.
    S30,
    /// 1-minute bars.
    M1,
    /// 2-minute bars.
    M2,
    /// 4-minute bars.
    M4,
    /// 5-minute bars.
    M5,
    /// 10-minute bars.
    M10,
    /// 15-minute bars.
    M15,
    /// 30-minute bars.
    M30,
    /// 1-hour bars.
    H1,
    /// 2-hour bars.
    H2,
    /// 3-hour bars.
    H3,
    /// 4-hour bars.
    H4,
    /// 6-hour bars.
    H6,
    /// 8-hour bars.
    H8,
    /// 12-hour bars.
    H12,
    /// Daily bars.
    D,
    /// Weekly bars.
    W,
    /// Monthly bars.
    M,
}

/// The day on which weekly candlestick bars begin.
#[derive(Debug, Serialize, Deserialize, Display, EnumString)]
pub enum WeeklyAlignment {
    Monday,
    Tuesday,
    Wednesday,
    Thursday,
    Friday,
    Saturday,
    Sunday,
}

/// A single OHLCV candlestick bar.
#[derive(Debug, Serialize, Deserialize)]
pub struct Candlestick {
    /// The timestamp of the bar open (start of the period).
    pub time: DateTime<Local>,
    /// Bid-side OHLC data, present when `"B"` is included in the price component.
    pub bid: Option<CandlestickData>,
    /// Ask-side OHLC data, present when `"A"` is included in the price component.
    pub ask: Option<CandlestickData>,
    /// Mid-point OHLC data, present when `"M"` is included in the price component.
    pub mid: Option<CandlestickData>,
    /// Number of ticks that contributed to this bar.
    pub volume: i16,
    /// `true` if the bar is complete (its period has fully elapsed).
    pub complete: bool,
}

/// OHLC (open, high, low, close) price data for one side of a candlestick.
#[derive(Debug, Serialize, Deserialize)]
pub struct CandlestickData {
    /// Opening price of the period.
    #[serde(rename = "o")]
    pub open: PriceValue,
    /// Highest price reached during the period.
    #[serde(rename = "h")]
    pub high: PriceValue,
    /// Lowest price reached during the period.
    #[serde(rename = "l")]
    pub low: PriceValue,
    /// Closing price of the period.
    #[serde(rename = "c")]
    pub close: PriceValue,
}

/// Response body for `GET /v3/accounts/{accountID}/instruments`.
#[derive(Debug, Serialize, Deserialize)]
pub struct ListInstrumentsResponse {
    /// The list of instruments available to the account.
    pub instruments: Vec<Instrument>,
    /// The ID of the most recent transaction on the account.
    #[serde(rename = "lastTransactionID")]
    pub last_transaction_id: TransactionID,
}

/// Builder for a `GET /v3/instruments/{instrument}/candles` request.
///
/// Construct via [`FetchCandlesticksRequest::new`], configure with the
/// builder methods, then pass to [`InstrumentService::fetch_candlesticks`].
///
/// # Example
///
/// ```no_run
/// use oanda_rust::instrument::{CandlestickGranularity, FetchCandlesticksRequest};
///
/// let req = FetchCandlesticksRequest::new("EUR_USD".to_string())
///     .mid()
///     .granularity(CandlestickGranularity::H1)
///     .count(100)
///     .unwrap();
/// ```
pub struct FetchCandlesticksRequest {
    /// The instrument to fetch candles for.
    pub instrument: InstrumentName,
    /// Which price components to include (`"B"`, `"A"`, `"M"`, or any combination).
    price: PricingComponent,
    /// The granularity (bar size) of the candles.
    granularity: Option<CandlestickGranularity>,
    /// Maximum number of candles to return (1–5000).
    count: Option<usize>,
    /// Start of the requested time range (inclusive).
    from: Option<DateTime<Local>>,
    /// End of the requested time range (exclusive).
    to: Option<DateTime<Local>>,
    /// When `true`, each bar's open equals the previous bar's close (smooth candles).
    smooth: Option<bool>,
    /// When `true`, the first candle in the range is included even if it predates `from`.
    include_first: Option<bool>,
    /// Hour of the day (0–23) used as the alignment point for daily candles.
    daily_alignment: Option<u8>,
    /// IANA timezone name used with `daily_alignment` (e.g. `"America/New_York"`).
    alignment_timezone: Option<String>,
    /// Day of the week on which weekly bars begin.
    weekly_alignment: Option<WeeklyAlignment>,
}

impl<'a> FetchCandlesticksRequest {
    /// Creates a new request for the given instrument with no parameters set.
    ///
    /// Call the builder methods (`bid`, `ask`, `mid`, `granularity`, etc.) to
    /// configure the request before passing it to
    /// [`InstrumentService::fetch_candlesticks`].
    pub fn new(instrument: InstrumentName) -> Self {
        FetchCandlesticksRequest {
            instrument,
            price: "".into(),
            granularity: None,
            count: None,
            from: None,
            to: None,
            smooth: None,
            include_first: None,
            daily_alignment: None,
            alignment_timezone: None,
            weekly_alignment: None,
        }
    }

    /// Includes bid-side (`"B"`) OHLC data in the response.
    pub fn bid(mut self) -> Self {
        self.price.push('B');
        self
    }

    /// Includes ask-side (`"A"`) OHLC data in the response.
    pub fn ask(mut self) -> Self {
        self.price.push('A');
        self
    }

    /// Includes mid-point (`"M"`) OHLC data in the response.
    pub fn mid(mut self) -> Self {
        self.price.push('M');
        self
    }

    /// Sets the bar size for the returned candles.
    pub fn granularity(mut self, granularity: CandlestickGranularity) -> Self {
        self.granularity = Some(granularity);
        self
    }

    /// Sets the maximum number of candles to return.
    ///
    /// # Errors
    ///
    /// Returns [`APIError::InvalidRequest`] if `count` is greater than 5000.
    pub fn count(mut self, count: usize) -> Result<Self, APIError> {
        (count <= 5000)
            .then(|| {
                self.count = Some(count);
                self
            })
            .ok_or_else(|| APIError::InvalidRequest("count must be <= 5000".to_string()))
    }

    /// Sets the start of the requested time range (inclusive).
    pub fn from(mut self, from: DateTime<Local>) -> Self {
        self.from = Some(from);
        self
    }

    /// Sets the end of the requested time range (exclusive).
    pub fn to(mut self, to: DateTime<Local>) -> Self {
        self.to = Some(to);
        self
    }

    /// When set to `true`, each bar's open price equals the previous bar's close
    /// (smooth/continuous candles). Defaults to `false` (standard OHLC).
    pub fn smooth(mut self, smooth: bool) -> Self {
        self.smooth = Some(smooth);
        self
    }

    /// When set to `true`, the first candle in the range is included in the
    /// response even if its open time is earlier than `from`.
    pub fn include_first(mut self, include_first: bool) -> Self {
        self.include_first = Some(include_first);
        self
    }

    /// Sets the hour of day (0–23, server-local time) used as the alignment
    /// boundary for daily candles.
    pub fn daily_alignment(mut self, daily_alignment: u8) -> Self {
        self.daily_alignment = Some(daily_alignment);
        self
    }

    /// Sets the IANA timezone used in conjunction with `daily_alignment`
    /// (e.g. `"America/New_York"`).
    pub fn alignment_timezone(mut self, alignment_timezone: String) -> Self {
        self.alignment_timezone = Some(alignment_timezone);
        self
    }

    /// Sets the day of the week on which weekly bars begin.
    pub fn weekly_alignment(mut self, weekly_alignment: WeeklyAlignment) -> Self {
        self.weekly_alignment = Some(weekly_alignment);
        self
    }

    /// Appends all configured query parameters to `url`.
    ///
    /// Called internally by [`InstrumentService::fetch_candlesticks`] before
    /// dispatching the HTTP request.
    pub fn set_params(&self, url: &mut Url) {
        self.price.is_empty().not().then(|| {
            url.query_pairs_mut()
                .append_pair("price", self.price.as_str());
        });
        self.granularity.is_some().then(|| {
            url.query_pairs_mut().append_pair(
                "granularity",
                &self.granularity.as_ref().unwrap().to_string(),
            );
        });
        self.count.is_some().then(|| {
            url.query_pairs_mut()
                .append_pair("count", &self.count.unwrap().to_string());
        });
        self.from.is_some().then(|| {
            url.query_pairs_mut()
                .append_pair("from", &self.from.unwrap().to_string());
        });
        self.to.is_some().then(|| {
            url.query_pairs_mut()
                .append_pair("to", &self.to.unwrap().to_string());
        });
        self.smooth.is_some().then(|| {
            url.query_pairs_mut()
                .append_pair("smooth", &self.smooth.unwrap().to_string());
        });
        self.include_first.is_some().then(|| {
            url.query_pairs_mut()
                .append_pair("includeFirst", &self.include_first.unwrap().to_string());
        });
        self.daily_alignment.is_some().then(|| {
            url.query_pairs_mut()
                .append_pair("dailyAlignment", &self.daily_alignment.unwrap().to_string());
        });
        self.alignment_timezone.is_some().then(|| {
            url.query_pairs_mut().append_pair(
                "alignmentTimezone",
                &self.alignment_timezone.as_ref().unwrap().to_string(),
            );
        });
        self.weekly_alignment.is_some().then(|| {
            url.query_pairs_mut().append_pair(
                "weeklyAlignment",
                &self.weekly_alignment.as_ref().unwrap().to_string(),
            );
        });
    }
}

/// Response body for `GET /v3/instruments/{instrument}/candles`.
#[derive(Debug, Serialize, Deserialize)]
pub struct FetchCandlesticksResponse {
    /// The instrument the candles belong to.
    pub instrument: InstrumentName,
    /// The granularity of the returned candles.
    granularity: CandlestickGranularity,
    /// The candlestick bars, ordered chronologically.
    candles: Vec<Candlestick>,
}

/// Provides access to the OANDA Instrument endpoints.
///
/// Obtain an instance via [`Client::instrument`](crate::client::Client::instrument).
pub struct InstrumentService<'a> {
    client: &'a Client,
}

impl<'a> InstrumentService<'a> {
    /// Creates a new `InstrumentService` bound to the given client.
    pub fn new(client: &'a Client) -> Self {
        InstrumentService { client }
    }

    /// Lists all instruments available to the configured account.
    ///
    /// Calls `GET /v3/accounts/{accountID}/instruments`.
    ///
    /// # Panics
    ///
    /// Panics if no `account_id` has been set on the client.
    pub async fn list(&self) -> Result<ListInstrumentsResponse, APIError> {
        let url = self
            .client
            .base_url
            .join(
                format!(
                    "/v3/accounts/{}/instruments",
                    self.client
                        .account_id
                        .as_ref()
                        .expect("Missing account_id in client")
                )
                .as_str(),
            )
            .unwrap();
        let http_req = Request::new(reqwest::Method::GET, url);
        let http_resp = self.client.http_client.execute(http_req).await?;
        handle_response!(
            http_resp,
            success: StatusCode::OK => ListInstrumentsResponse,
            errors: [ ]
        )
    }

    /// Fetches historical candlestick (OHLCV) data for an instrument.
    ///
    /// Calls `GET /v3/instruments/{instrument}/candles` with the parameters
    /// encoded in `req`. Build the request with [`FetchCandlesticksRequest`].
    pub async fn fetch_candlesticks(
        &self,
        req: FetchCandlesticksRequest,
    ) -> Result<FetchCandlesticksResponse, APIError> {
        let mut url = self
            .client
            .base_url
            .join(format!("/v3/instruments/{}/candles", req.instrument).as_str())
            .unwrap();
        req.set_params(&mut url);
        let http_req = Request::new(reqwest::Method::GET, url);
        let http_resp = self.client.http_client.execute(http_req).await?;
        handle_response!(
            http_resp,
            success: StatusCode::OK => FetchCandlesticksResponse,
            errors: [ ]
        )
    }
}

#[cfg(test)]
mod tests {
    use crate::client::setup_test_client;
    use crate::instrument::{CandlestickGranularity, FetchCandlesticksRequest};

    #[tokio::test]
    async fn test_list_instruments() {
        let client = setup_test_client();
        let resp = client.instrument().list().await.unwrap();
        println!("{:#?}", resp);
    }

    #[tokio::test]
    async fn test_fetch_candlestick_data() {
        let client = setup_test_client();
        let req = FetchCandlesticksRequest::new("USD_JPY".to_string())
            .granularity(CandlestickGranularity::M1)
            .count(50)
            .unwrap();
        let resp = client.instrument().fetch_candlesticks(req).await.unwrap();
        println!("{:#?}", resp);
    }
}
