use crate::account::AccountID;
use crate::errors::APIError;
use crate::http::{self, HttpClient};
use crate::pricing::PricingStreamItem;
use crate::transaction::TransactionStreamItem;
use futures_util::stream::StreamExt;
use futures_util::Stream;
use reqwest::Request;
use serde::de::DeserializeOwned;
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
    pub async fn transactions<F>(&self, handler: F) -> Result<(), APIError>
    where
        F: FnMut(TransactionStreamItem) -> Result<(), APIError>,
    {
        let url = http::account_url(
            &self.base_url,
            self.account_id.as_ref(),
            "transactions/stream",
        )?;
        self.consume(url, handler).await
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
    pub async fn pricing<F>(&self, instruments: &[&str], handler: F) -> Result<(), APIError>
    where
        F: FnMut(PricingStreamItem) -> Result<(), APIError>,
    {
        let mut url =
            http::account_url(&self.base_url, self.account_id.as_ref(), "pricing/stream")?;
        url.query_pairs_mut()
            .append_pair("instruments", &instruments.join(","));
        self.consume(url, handler).await
    }

    async fn consume<T, F>(&self, url: Url, handler: F) -> Result<(), APIError>
    where
        T: DeserializeOwned,
        F: FnMut(T) -> Result<(), APIError>,
    {
        let response = self
            .http_client
            .execute(Request::new(reqwest::Method::GET, url))
            .await?;
        if response.status() != reqwest::StatusCode::OK {
            return http::decode_response::<()>(response, reqwest::StatusCode::OK, None).await;
        }
        consume_ndjson(
            response
                .bytes_stream()
                .map(|chunk| chunk.map_err(APIError::from)),
            handler,
        )
        .await
    }
}

/// Frames messages independently of HTTP chunk boundaries, including a final
/// message without a newline. Incomplete JSON at EOF is reported as an error.
async fn consume_ndjson<T, S, B, F>(stream: S, mut handler: F) -> Result<(), APIError>
where
    T: DeserializeOwned,
    S: Stream<Item = Result<B, APIError>>,
    B: AsRef<[u8]>,
    F: FnMut(T) -> Result<(), APIError>,
{
    futures_util::pin_mut!(stream);
    let mut buffer = Vec::new();
    while let Some(chunk) = stream.next().await {
        let chunk = chunk?;
        for part in chunk.as_ref().split_inclusive(|byte| *byte == b'\n') {
            buffer.extend_from_slice(part);
            if part.last() == Some(&b'\n') {
                let line = buffer.trim_ascii();
                if !line.is_empty() {
                    handler(serde_json::from_slice(line)?)?;
                }
                buffer.clear();
            }
        }
    }
    let line = buffer.trim_ascii();
    if !line.is_empty() {
        handler(serde_json::from_slice(line)?)?;
    }
    Ok(())
}

#[cfg(test)]
mod framing_tests {
    use super::*;
    use futures_util::stream;
    use serde_json::Value;

    #[tokio::test]
    async fn frames_split_utf8_blank_lines_and_final_message() {
        let wire = "\n{\"name\":\"円\"}\r\n \n{\"n\":2}".as_bytes();
        // Every UTF-8 code unit and delimiter can arrive in a separate chunk.
        let chunks = stream::iter(wire.chunks(1).map(Ok::<_, APIError>));
        let mut messages = Vec::<Value>::new();
        consume_ndjson(chunks, |value| {
            messages.push(value);
            Ok(())
        })
        .await
        .unwrap();
        assert_eq!(
            messages,
            vec![serde_json::json!({"name":"円"}), serde_json::json!({"n":2})]
        );
        let mut count = 0;
        consume_ndjson::<Value, _, _, _>(stream::iter([Ok(wire)]), |_| {
            count += 1;
            Ok(())
        })
        .await
        .unwrap();
        assert_eq!(count, 2);
    }

    #[tokio::test]
    async fn malformed_or_truncated_messages_return_errors() {
        for wire in [b"{bad}\n".as_slice(), b"{\"n\":".as_slice()] {
            let result =
                consume_ndjson::<Value, _, _, _>(stream::iter([Ok(wire)]), |_| Ok(())).await;
            assert!(matches!(result, Err(APIError::JSONError(_))));
        }
    }

    #[tokio::test]
    async fn handler_error_stops_before_later_messages() {
        let mut count = 0;
        let result = consume_ndjson::<Value, _, _, _>(stream::iter([Ok(b"1\n2\n")]), |_| {
            count += 1;
            Err(APIError::InvalidRequest("handler stopped".into()))
        })
        .await;
        assert_eq!(count, 1);
        assert!(
            matches!(result, Err(APIError::InvalidRequest(message)) if message == "handler stopped")
        );
    }

    #[tokio::test]
    async fn chunk_error_is_propagated_without_dispatching_partial_json() {
        let chunks = stream::iter([
            Ok(b"{\"n\":".as_slice()),
            Err(APIError::InvalidRequest("transport stopped".into())),
        ]);
        let mut count = 0;
        let result = consume_ndjson::<Value, _, _, _>(chunks, |_| {
            count += 1;
            Ok(())
        })
        .await;
        assert_eq!(count, 0);
        assert!(
            matches!(result, Err(APIError::InvalidRequest(message)) if message == "transport stopped")
        );
    }
}
