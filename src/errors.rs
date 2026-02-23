use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use crate::order::UpdateClientExtensionsErrorResponse;

#[derive(Error, Debug)]
pub enum APIError {
    #[error("Invalid request: {0}")]
    InvalidRequest(String),

    #[error("HTTP request failed: {0}")]
    HTTPError(#[from] reqwest::Error),

    #[error("JSON error: {0}")]
    JSONError(#[from] serde_json::Error),

    #[error("API returned error {status}: {message}")]
    ErrorResponse {
        status: StatusCode,
        message: String,
    },

    #[error(transparent)]
    DetailedErrorResponse(#[from] DetailedErrorResponse),
}

#[derive(Error, Debug, Serialize, Deserialize)]
#[error("{error_message}")]
pub struct ErrorResponse {
    #[serde(rename = "errorMessage")]
    pub error_message: String,
}

#[derive(Error, Debug, Serialize, Deserialize)]
#[error(transparent)]
pub enum DetailedErrorResponse {
    #[error(transparent)]
    UpdateClientExtensionsErrorResponse(#[from] UpdateClientExtensionsErrorResponse),
}
