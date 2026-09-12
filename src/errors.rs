use crate::account::ConfigureAccountErrorResponse;
use crate::order::{
    OrderCancelErrorResponse, OrderCreateErrorResponse, UpdateOrderClientExtensionsErrorResponse,
};
use crate::trade::UpdateTradeClientExtensionsErrorResponse;
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Top-level error type returned by all service methods.
///
/// Includes request, transport, decoding, and structured API failures.
/// Response failures retain HTTP status and the server request ID.
#[derive(Error, Debug)]
pub enum APIError {
    /// A response failure with HTTP status and request ID context.
    #[error(transparent)]
    Response(Box<HttpResponseError>),
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
    ///
    /// Boxed to keep `APIError` (and every `Result` carrying it) small.
    #[error(transparent)]
    ErrorResponse(Box<ErrorResponse>),
}

impl From<ErrorResponse> for APIError {
    fn from(err: ErrorResponse) -> Self {
        APIError::ErrorResponse(Box::new(err))
    }
}

/// A generic OANDA error response body containing only a human-readable message.
///
/// Used for endpoints that return a plain `{"errorMessage": "..."}` payload on
/// failure (e.g. authentication errors, rate-limit rejections).
#[derive(Error, Debug, Serialize, Deserialize)]
#[error("{error_message}")]
#[serde(rename_all = "camelCase")]
pub struct CommonErrorResponse {
    /// Human-readable description of the error returned by the OANDA API.
    pub error_message: String,
}

/// A structured error body returned by one of the OANDA API endpoints.
///
/// Each variant corresponds to a specific endpoint's reject/error schema.
/// Constructed by the response decoder when the server returns a
/// non-success status code with a recognised error payload.
#[derive(Error, Debug, Serialize, Deserialize)]
#[error(transparent)]
pub enum ErrorResponse {
    /// A generic error with only an `errorMessage` field.
    #[error(transparent)]
    CommonError(CommonErrorResponse),
    /// An order-create request was rejected by the OANDA risk engine.
    #[error(transparent)]
    OrderCreateError(OrderCreateErrorResponse),
    /// An order-cancel request was rejected.
    #[error(transparent)]
    OrderCancelError(OrderCancelErrorResponse),
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

/// HTTP context retained when reading or decoding a response fails.
#[derive(Debug, Error)]
#[error("HTTP {status} (request ID {request_id:?}): {source}")]
pub struct HttpResponseError {
    pub status: reqwest::StatusCode,
    pub request_id: Option<String>,
    #[source]
    pub source: APIError,
}
