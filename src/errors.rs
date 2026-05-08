use crate::account::ConfigureAccountErrorResponse;
use crate::order::{
    OrderCancelRejectResponse, OrderCreateRejectResponse, UpdateOrderClientExtensionsErrorResponse,
};
use crate::trade::UpdateTradeClientExtensionsErrorResponse;
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Top-level error type returned by all service methods.
///
/// Variants cover the four failure modes: a malformed request caught before
/// sending, a transport-level HTTP failure, a JSON deserialisation failure,
/// and a well-formed API error response from the OANDA server.
#[derive(Error, Debug)]
pub enum APIError {
    /// The request could not be constructed (e.g. a required field is missing).
    #[error("Invalid request: {0}")]
    InvalidRequest(String),
    /// The underlying HTTP request failed (connection error, timeout, etc.).
    #[error("HTTP request failed: {0}")]
    HTTPError(#[from] reqwest::Error),
    /// The response body could not be deserialised as the expected JSON type.
    #[error("JSON error: {0}")]
    JSONError(#[from] serde_json::Error),
    /// The OANDA API returned a non-success response with a structured error body.
    #[error(transparent)]
    ErrorResponse(#[from] ErrorResponse),
}

/// A generic OANDA error response body containing only a human-readable message.
///
/// Used for endpoints that return a plain `{"errorMessage": "..."}` payload on
/// failure (e.g. authentication errors, rate-limit rejections).
#[derive(Error, Debug, Serialize, Deserialize)]
#[error("{error_message}")]
pub struct CommonErrorResponse {
    /// Human-readable description of the error returned by the OANDA API.
    #[serde(rename = "errorMessage")]
    pub error_message: String,
}

/// A structured error body returned by one of the OANDA API endpoints.
///
/// Each variant corresponds to a specific endpoint's reject/error schema.
/// Constructed by the `handle_response!` macro when the server returns a
/// non-success status code with a recognised error payload.
#[derive(Error, Debug, Serialize, Deserialize)]
#[error(transparent)]
pub enum ErrorResponse {
    /// A generic error with only an `errorMessage` field.
    #[error(transparent)]
    CommonError(CommonErrorResponse),
    /// An order-create request was rejected by the OANDA risk engine.
    #[error(transparent)]
    OrderCreateError(OrderCreateRejectResponse),
    /// An order-cancel request was rejected.
    #[error(transparent)]
    OrderCancelError(OrderCancelRejectResponse),
    /// An order client-extensions update was rejected.
    #[error(transparent)]
    UpdateOrderClientExtensionsError(#[from] UpdateOrderClientExtensionsErrorResponse),
    /// A trade client-extensions update was rejected.
    #[error(transparent)]
    UpdateTradeClientExtensionsError(#[from] UpdateTradeClientExtensionsErrorResponse),
    /// An account configuration update was rejected.
    #[error(transparent)]
    ConfigureAccountError(#[from] ConfigureAccountErrorResponse),
}
