use crate::order::{
    OrderCancelRejectResponse, OrderCreateRejectResponse, UpdateOrderClientExtensionsErrorResponse,
};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use crate::trade::UpdateTradeClientExtensionsErrorResponse;

#[derive(Error, Debug)]
pub enum APIError {
    #[error("Invalid request: {0}")]
    InvalidRequest(String),
    #[error("HTTP request failed: {0}")]
    HTTPError(#[from] reqwest::Error),
    #[error("JSON error: {0}")]
    JSONError(#[from] serde_json::Error),
    #[error(transparent)]
    ErrorResponse(#[from] ErrorResponse),
}

#[derive(Error, Debug, Serialize, Deserialize)]
#[error("{error_message}")]
pub struct CommonErrorResponse {
    #[serde(rename = "errorMessage")]
    pub error_message: String,
}

#[derive(Error, Debug, Serialize, Deserialize)]
#[error(transparent)]
pub enum ErrorResponse {
    #[error(transparent)]
    CommonError(CommonErrorResponse),
    #[error(transparent)]
    OrderCreateError(OrderCreateRejectResponse),
    #[error(transparent)]
    OrderCancelError(OrderCancelRejectResponse),
    #[error(transparent)]
    UpdateOrderClientExtensionsError(#[from] UpdateOrderClientExtensionsErrorResponse),
    #[error(transparent)]
    UpdateTradeClientExtensionsError(#[from] UpdateTradeClientExtensionsErrorResponse),
}
