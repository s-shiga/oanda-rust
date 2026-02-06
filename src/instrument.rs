use crate::primitives::{DecimalNumber, Tag};
use serde::{Deserialize, Serialize};

pub type InstrumentName<'a> = &'a str;

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
    #[serde(rename = "guaranteedStopLossOrderLevelRestriction", skip_serializing_if = "Option::is_none")]
    pub guaranteed_stop_loss_order_level_restriction: Option<GuaranteedStopLossOrderLevelRestriction>,
    pub financing: InstrumentFinancing,
    pub tags: Vec<Tag>,
}
