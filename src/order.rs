use crate::client::{request_option_setter, request_setter};
use crate::errors::{APIError, ErrorResponse};
use crate::http::{decode_reject, decode_response, Connection};
use crate::instrument::InstrumentName;
use crate::pricing::PriceValue;
use crate::primitives::DecimalNumber;
use crate::transaction::{
    ClientExtensions, ClientID, GuaranteedStopLossDetails, MarketOrderDelayedTradeClose,
    MarketOrderMarginCloseout, MarketOrderPositionCloseout, MarketOrderTradeClose,
    OrderCancelRejectTransaction, OrderCancelTransaction,
    OrderClientExtensionsModifyRejectTransaction, OrderClientExtensionsModifyTransaction,
    OrderCreateRejectTransaction, OrderCreateTransaction, OrderFillTransaction, OrderID,
    StopLossDetails, TakeProfitDetails, TradeID, TrailingStopLossDetails, TransactionID,
};
use chrono::{DateTime, Utc};
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use strum_macros::Display;
use thiserror::Error;
use url::Url;

mod model;
mod request;
mod response;
mod service;

pub use model::*;
pub use request::*;
pub use response::*;
pub use service::*;
