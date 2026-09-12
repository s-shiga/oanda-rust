use crate::account::AccountID;
use crate::errors::APIError;
use crate::http::{self, HttpClient};
use crate::pricing::PricingStreamItem;
use crate::transaction::TransactionStreamItem;
use futures_util::stream::StreamExt;
use reqwest::Request;
use url::Url;

/// HTTP client for OANDA's long-lived streaming endpoints.
///
/// Unlike [`Client`](crate::client::Client), which targets the REST API,
/// `StreamClient` connects to the separate OANDA streaming host and keeps the
/// connection open, delivering newline-delimited JSON messages as they arrive.
///
/// # Example
///
/// ```no_run
/// use oanda_rust::stream::StreamClient;
///
/// # #[tokio::main]
/// # async fn main() {
/// let client = StreamClient::new_practice("my-api-key").unwrap()
///     .with_account_id("001-001-1234567-001".to_string());
///
/// client.transactions(|item| {
///     println!("{:?}", item);
///     Ok(())
/// }).await.unwrap();
/// # }
/// ```
pub struct StreamClient {
    pub(crate) base_url: Url,
    pub(crate) http_client: HttpClient,
    pub(crate) account_id: Option<AccountID>,
}

const FX_TRADE_PRACTICE_STREAMING_URL: &str = "https://stream-fxpractice.oanda.com";
const FX_TRADE_STREAMING_URL: &str = "https://stream-fxtrade.oanda.com";

impl StreamClient {
    /// Creates a `StreamClient` targeting the live trading streaming API.
    ///
    /// The `api_key` is sent as a `Bearer` token on every request.
    /// Call [`with_account_id`](Self::with_account_id) before streaming.
    ///
    /// Returns an error if the token is invalid or the HTTP client cannot be built.
    pub fn new(api_key: &str) -> Result<Self, APIError> {
        Ok(StreamClient {
            base_url: Url::parse(FX_TRADE_STREAMING_URL).unwrap(),
            http_client: HttpClient::new(api_key, "application/octet-stream")?,
            account_id: None,
        })
    }

    /// Creates a `StreamClient` targeting the practice (demo) streaming API.
    ///
    /// The `api_key` is sent as a `Bearer` token on every request.
    /// Call [`with_account_id`](Self::with_account_id) before streaming.
    ///
    /// Returns an error if the token is invalid or the HTTP client cannot be built.
    pub fn new_practice(api_key: &str) -> Result<Self, APIError> {
        Ok(StreamClient {
            base_url: Url::parse(FX_TRADE_PRACTICE_STREAMING_URL).unwrap(),
            http_client: HttpClient::new(api_key, "application/octet-stream")?,
            account_id: None,
        })
    }

    /// Uses a custom HTTP client while preserving OANDA authentication headers.
    pub fn with_http_client(mut self, client: reqwest::Client) -> Self {
        self.http_client.client = client;
        self
    }

    /// Overrides the streaming endpoint, for example for a local fixture server.
    /// Requests to this endpoint include the configured API token.
    pub fn with_base_url(mut self, url: Url) -> Result<Self, APIError> {
        http::validate_base_url(&url)?;
        self.base_url = url;
        Ok(self)
    }

    /// Sets the account ID used for streaming endpoints and returns `self`.
    pub fn with_account_id(mut self, account_id: AccountID) -> Self {
        self.account_id = Some(account_id);
        self
    }

    /// Opens a persistent connection to the transaction stream and invokes
    /// `handler` for each message received.
    ///
    /// Calls `GET /v3/accounts/{accountID}/transactions/stream`. The connection
    /// stays open until the server closes it, `handler` returns an `Err`, or a
    /// network error occurs. Messages are newline-delimited JSON and may be
    /// heartbeats or transaction events — see [`TransactionStreamItem`].
    ///
    /// # Errors
    ///
    /// Returns [`APIError`] if the initial HTTP request fails, the server
    /// returns a non-2xx status, a chunk cannot be read, a message cannot be
    /// deserialised, or `handler` itself returns an error.
    ///
    /// Returns [`APIError::InvalidRequest`] if no account ID is configured.
    pub async fn transactions(
        &self,
        handler: fn(TransactionStreamItem) -> Result<(), APIError>,
    ) -> Result<(), APIError> {
        let url = http::account_url(
            &self.base_url,
            self.account_id.as_ref(),
            "transactions/stream",
        )?;
        let http_req = Request::new(reqwest::Method::GET, url);
        let http_resp = self.http_client.execute(http_req).await?;
        if http_resp.status() != reqwest::StatusCode::OK {
            return http::decode_response::<()>(http_resp, reqwest::StatusCode::OK, None).await;
        }
        let mut stream = http_resp.bytes_stream();
        let mut buffer = Vec::new();
        while let Some(result) = stream.next().await {
            match result {
                Ok(chunk) => {
                    buffer.extend_from_slice(&chunk);
                    while let Some(pos) = buffer.iter().position(|&b| b == b'\n') {
                        let line: Vec<u8> = buffer.drain(..=pos).collect();
                        let trimmed = line.trim_ascii();
                        if trimmed.is_empty() {
                            continue;
                        }
                        let transaction = serde_json::from_slice::<TransactionStreamItem>(trimmed)?;
                        handler(transaction)?;
                    }
                }
                Err(error) => return Err(APIError::from(error)),
            }
        }
        Ok(())
    }

    /// Opens a persistent connection to the pricing stream and invokes
    /// `handler` for each message received.
    ///
    /// Calls `GET /v3/accounts/{accountID}/pricing/stream`. The `instruments`
    /// slice must contain at least one instrument name (e.g. `"EUR_USD"`).
    /// The connection stays open until the server closes it, `handler` returns
    /// an `Err`, or a network error occurs. Messages are newline-delimited JSON
    /// and may be price updates or heartbeats — see [`PricingStreamItem`].
    ///
    /// # Errors
    ///
    /// Returns [`APIError`] if the initial HTTP request fails, the server
    /// returns a non-2xx status, a chunk cannot be read, a message cannot be
    /// deserialised, or `handler` itself returns an error.
    ///
    /// Returns [`APIError::InvalidRequest`] if no account ID is configured.
    pub async fn pricing(
        &self,
        instruments: &[&str],
        handler: fn(PricingStreamItem) -> Result<(), APIError>,
    ) -> Result<(), APIError> {
        let mut url =
            http::account_url(&self.base_url, self.account_id.as_ref(), "pricing/stream")?;
        url.query_pairs_mut()
            .append_pair("instruments", &instruments.join(","));
        let http_req = Request::new(reqwest::Method::GET, url);
        let http_resp = self.http_client.execute(http_req).await?;
        if http_resp.status() != reqwest::StatusCode::OK {
            return http::decode_response::<()>(http_resp, reqwest::StatusCode::OK, None).await;
        }
        let mut stream = http_resp.bytes_stream();
        let mut buffer = Vec::new();
        while let Some(result) = stream.next().await {
            match result {
                Ok(chunk) => {
                    buffer.extend_from_slice(&chunk);
                    while let Some(pos) = buffer.iter().position(|&b| b == b'\n') {
                        let line: Vec<u8> = buffer.drain(..=pos).collect();
                        let trimmed = line.trim_ascii();
                        if trimmed.is_empty() {
                            continue;
                        }
                        let item = serde_json::from_slice::<PricingStreamItem>(trimmed)?;
                        handler(item)?;
                    }
                }
                Err(error) => return Err(APIError::from(error)),
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::stream::StreamClient;
    use std::time::Duration;
    use tokio::time::timeout;

    fn setup() -> StreamClient {
        let api_key = std::env::var("OANDA_API_KEY_DEMO").expect("OANDA_API_KEY_DEMO must be set");
        let account_id =
            std::env::var("OANDA_ACCOUNT_ID_DEMO").expect("OANDA_ACCOUNT_ID_DEMO must be set");
        StreamClient::new_practice(&api_key)
            .unwrap()
            .with_account_id(account_id)
    }

    #[tokio::test]
    #[ignore = "requires OANDA demo credentials; run explicitly with --ignored"]
    async fn test_pricing() {
        let client = setup();
        let _ = timeout(
            Duration::from_secs(10),
            client.pricing(&["EUR_USD"], |item| {
                println!("{:#?}", item);
                Ok(())
            }),
        )
        .await;
    }

    #[tokio::test]
    #[ignore = "requires OANDA demo credentials; run explicitly with --ignored"]
    async fn test_stream_transactions() {
        let client = setup();
        let _ = timeout(
            Duration::from_secs(10),
            client.transactions(|item| {
                println!("{:#?}", item);
                Ok(())
            }),
        )
        .await;
    }
}
