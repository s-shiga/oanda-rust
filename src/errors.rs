use thiserror::Error;

#[derive(Error, Debug)]
pub enum APIError {
    #[error("Invalid parameter")]
    InvalidParameter(String),

    #[error("HTTP request failed: {0}")]
    RequestFailed(#[from] reqwest::Error),

    #[error("API returned error {status}: {message}")]
    ApiErrorResponse {
        status: reqwest::StatusCode,
        message: String,
    },

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
}
