use crate::errors::APIError;
use crate::pricing::{PriceValue, PricingComponent};
use crate::primitives::{DecimalNumber, Tag};
use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};
use std::ops::Not;
use strum_macros::{Display, EnumString};
use url::Url;

pub type InstrumentName = String;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum InstrumentType {
    Currency,
    CFD,
    Metal,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InstrumentCommission {
    pub commission: DecimalNumber,
    #[serde(rename = "unitsTraded")]
    pub units_traded: DecimalNumber,
    #[serde(rename = "minimumCommission")]
    pub min_commission: DecimalNumber,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum GuaranteedStopLossOrderModeForInstrument {
    Disabled,
    Allowed,
    Required,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GuaranteedStopLossOrderLevelRestriction {
    pub volume: DecimalNumber,
    #[serde(rename = "priceRange")]
    pub price_range: DecimalNumber,
}

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

#[derive(Debug, Serialize, Deserialize)]
pub struct FinancingDayOfWeek {
    #[serde(rename = "dayOfWeek")]
    pub day_of_week: DayOfWeek,
    #[serde(rename = "daysCharged")]
    pub days_charged: i8,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InstrumentFinancing {
    #[serde(rename = "longRate")]
    pub long_rate: DecimalNumber,
    #[serde(rename = "shortRate")]
    pub short_rate: DecimalNumber,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Instrument {
    pub name: String,
    #[serde(rename = "type")]
    pub instrument_type: InstrumentType,
    #[serde(rename = "displayName")]
    pub display_name: String,
    #[serde(rename = "pipLocation")]
    pub pip_location: i8,
    #[serde(rename = "displayPrecision")]
    pub display_precision: i8,
    #[serde(rename = "tradeUnitsPrecision")]
    pub trade_units_precision: i8,
    #[serde(rename = "minimumTradeSize")]
    pub min_trade_size: DecimalNumber,
    #[serde(rename = "maximumTrailingStopDistance")]
    pub max_trailing_stop_distance: DecimalNumber,
    #[serde(
        rename = "minimumGuaranteedStopLossDistance",
        skip_serializing_if = "Option::is_none"
    )]
    pub min_guaranteed_stop_loss_distance: Option<DecimalNumber>,
    #[serde(rename = "minimumTrailingStopDistance")]
    pub min_trailing_stop_distance: DecimalNumber,
    #[serde(rename = "maximumPositionSize")]
    pub max_position_size: DecimalNumber,
    #[serde(rename = "maximumOrderUnits")]
    pub max_order_units: DecimalNumber,
    #[serde(rename = "marginRate")]
    pub margin_rate: DecimalNumber,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub commission: Option<InstrumentCommission>,
    #[serde(rename = "guaranteedStopLossOrderMode")]
    pub guaranteed_stop_loss_order_mode: GuaranteedStopLossOrderModeForInstrument,
    #[serde(
        rename = "guaranteedStopLossOrderExecutionPremium",
        skip_serializing_if = "Option::is_none"
    )]
    pub guaranteed_stop_loss_order_execution_premium: Option<DecimalNumber>,
    #[serde(
        rename = "guaranteedStopLossOrderLevelRestriction",
        skip_serializing_if = "Option::is_none"
    )]
    pub guaranteed_stop_loss_order_level_restriction:
        Option<GuaranteedStopLossOrderLevelRestriction>,
    pub financing: InstrumentFinancing,
    pub tags: Vec<Tag>,
}

#[derive(Debug, Serialize, Deserialize, Display, EnumString)]
pub enum CandlestickGranularity {
    S5,
    S10,
    S15,
    S30,
    M1,
    M2,
    M4,
    M5,
    M10,
    M15,
    M30,
    H1,
    H2,
    H3,
    H4,
    H6,
    H8,
    H12,
    D,
    W,
    M,
}

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

#[derive(Debug, Serialize, Deserialize)]
pub struct Candlestick {
    pub time: DateTime<Local>,
    pub bid: Option<CandlestickData>,
    pub ask: Option<CandlestickData>,
    pub mid: Option<CandlestickData>,
    pub volume: i16, // check
    pub complete: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CandlestickData {
    #[serde(rename = "o")]
    pub open: PriceValue,
    #[serde(rename = "h")]
    pub high: PriceValue,
    #[serde(rename = "l")]
    pub low: PriceValue,
    #[serde(rename = "c")]
    pub close: PriceValue,
}

pub struct FetchCandlestickDataRequest {
    pub instrument: InstrumentName,
    price: PricingComponent,
    granularity: Option<CandlestickGranularity>,
    count: Option<usize>,
    from: Option<DateTime<Local>>,
    to: Option<DateTime<Local>>,
    smooth: Option<bool>,
    include_first: Option<bool>,
    daily_alignment: Option<u8>,
    alignment_timezone: Option<String>,
    weekly_alignment: Option<WeeklyAlignment>,
}

impl<'a> FetchCandlestickDataRequest {
    pub fn new(instrument: InstrumentName) -> Self {
        FetchCandlestickDataRequest {
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

    pub fn bid(mut self) -> Self {
        self.price.push('B');
        self
    }

    pub fn ask(mut self) -> Self {
        self.price.push('A');
        self
    }

    pub fn mid(mut self) -> Self {
        self.price.push('M');
        self
    }

    pub fn granularity(mut self, granularity: CandlestickGranularity) -> Self {
        self.granularity = Some(granularity);
        self
    }

    pub fn count(mut self, count: usize) -> Result<Self, APIError> {
        (count <= 5000)
            .then(|| {
                self.count = Some(count);
                self
            })
            .ok_or_else(|| APIError::InvalidParameter("count must be <= 5000".to_string()))
    }

    pub fn from(mut self, from: DateTime<Local>) -> Self {
        self.from = Some(from);
        self
    }

    pub fn to(mut self, to: DateTime<Local>) -> Self {
        self.to = Some(to);
        self
    }

    pub fn smooth(mut self, smooth: bool) -> Self {
        self.smooth = Some(smooth);
        self
    }

    pub fn include_first(mut self, include_first: bool) -> Self {
        self.include_first = Some(include_first);
        self
    }

    pub fn daily_alignment(mut self, daily_alignment: u8) -> Self {
        self.daily_alignment = Some(daily_alignment);
        self
    }

    pub fn alignment_timezone(mut self, alignment_timezone: String) -> Self {
        self.alignment_timezone = Some(alignment_timezone);
        self
    }

    pub fn weekly_alignment(mut self, weekly_alignment: WeeklyAlignment) -> Self {
        self.weekly_alignment = Some(weekly_alignment);
        self
    }

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

#[derive(Debug, Serialize, Deserialize)]
pub struct FetchCandlestickDataResponse {
    pub instrument: InstrumentName,
    granularity: CandlestickGranularity,
    candles: Vec<Candlestick>,
}
