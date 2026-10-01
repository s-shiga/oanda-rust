use crate::account::AccountID;
use crate::errors::APIError;
use crate::http::{self, Connection};
use crate::pricing::{instruments_query, PricingStreamItem};
use crate::transaction::TransactionStreamItem;
use futures_util::stream::{Fuse, StreamExt};
use futures_util::Stream;
use serde::de::DeserializeOwned;
use std::pin::Pin;
use std::time::Duration;
use url::Url;

/// Optional parameters for the pricing stream. OANDA sends a current price on
/// connection unless `snapshot` is disabled.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PricingStreamOptions {
    pub snapshot: bool,
    pub include_home_conversions: bool,
}

impl Default for PricingStreamOptions {
    fn default() -> Self {
        Self {
            snapshot: true,
            include_home_conversions: false,
        }
    }
}

/// HTTP client for OANDA's long-lived streaming endpoints.
///
/// Unlike [`Client`](crate::client::Client), which targets the REST API,
/// `StreamClient` connects to the separate OANDA streaming host and keeps the
/// connection open, delivering newline-delimited JSON messages as they arrive.
/// A stream that receives no data for 30 seconds, several missed heartbeats,
/// ends with an [`APIError::HTTPError`] so the caller can reconnect.
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
    connection: Connection,
}

const FX_TRADE_PRACTICE_STREAMING_URL: &str = "https://stream-fxpractice.oanda.com";
const FX_TRADE_STREAMING_URL: &str = "https://stream-fxtrade.oanda.com";
const STREAM_ACCEPT: &str = "application/octet-stream";
/// OANDA sends a heartbeat every 5 seconds, so a read that waits this long
/// means the connection has stalled.
const STREAM_READ_TIMEOUT: Duration = Duration::from_secs(30);

impl StreamClient {
    /// Creates a `StreamClient` targeting the live trading streaming API.
    ///
    /// The `api_key` is sent as a `Bearer` token on every request.
    /// Call [`with_account_id`](Self::with_account_id) before streaming.
    ///
    /// Returns an error if the token is invalid or the HTTP client cannot be built.
    pub fn new(api_key: &str) -> Result<Self, APIError> {
        Ok(StreamClient {
            connection: Connection::new(
                api_key,
                STREAM_ACCEPT,
                FX_TRADE_STREAMING_URL,
                Some(STREAM_READ_TIMEOUT),
            )?,
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
            connection: Connection::new(
                api_key,
                STREAM_ACCEPT,
                FX_TRADE_PRACTICE_STREAMING_URL,
                Some(STREAM_READ_TIMEOUT),
            )?,
        })
    }

    /// Uses a custom HTTP client while preserving OANDA authentication headers.
    /// The client's own timeouts replace the default 30-second read timeout;
    /// set `read_timeout` on it to keep stalled streams from waiting forever.
    pub fn with_http_client(mut self, client: reqwest::Client) -> Self {
        self.connection.set_http_client(client);
        self
    }

    /// Overrides the streaming endpoint, for example for a local fixture server.
    /// Requests to this endpoint include the configured API token.
    pub fn with_base_url(mut self, url: Url) -> Result<Self, APIError> {
        self.connection.set_base_url(url)?;
        Ok(self)
    }

    /// Sets the account ID used for streaming endpoints and returns `self`.
    pub fn with_account_id(mut self, account_id: AccountID) -> Self {
        self.connection.account_id = Some(account_id);
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
        consume_items(self.transactions_stream().await?, handler).await
    }

    /// Opens the transaction stream and returns its messages as a pull-based
    /// stream. Dropping the returned stream closes the connection. A transport,
    /// framing, or decoding failure is yielded as an error and ends the stream.
    pub async fn transactions_stream(
        &self,
    ) -> Result<
        impl Stream<Item = Result<TransactionStreamItem, APIError>> + Send + 'static,
        APIError,
    > {
        let url = self.connection.account_url(&["transactions", "stream"])?;
        self.open(url).await
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
    /// Returns [`APIError::InvalidRequest`] if no account ID is configured or
    /// the instrument list is empty or contains a blank name.
    pub async fn pricing<F>(&self, instruments: &[&str], handler: F) -> Result<(), APIError>
    where
        F: FnMut(PricingStreamItem) -> Result<(), APIError>,
    {
        consume_items(self.pricing_stream(instruments).await?, handler).await
    }

    /// Opens a pricing stream with OANDA's default options.
    pub async fn pricing_stream(
        &self,
        instruments: &[&str],
    ) -> Result<impl Stream<Item = Result<PricingStreamItem, APIError>> + Send + 'static, APIError>
    {
        self.pricing_stream_with_options(instruments, PricingStreamOptions::default())
            .await
    }

    /// Opens a pricing stream, optionally suppressing the connect snapshot or
    /// requesting home-currency conversion factors.
    pub async fn pricing_stream_with_options(
        &self,
        instruments: &[&str],
        options: PricingStreamOptions,
    ) -> Result<impl Stream<Item = Result<PricingStreamItem, APIError>> + Send + 'static, APIError>
    {
        let mut url = self.connection.account_url(&["pricing", "stream"])?;
        url.query_pairs_mut()
            .append_pair("instruments", &instruments_query(instruments)?);
        if !options.snapshot {
            url.query_pairs_mut().append_pair("snapshot", "false");
        }
        if options.include_home_conversions {
            url.query_pairs_mut()
                .append_pair("includeHomeConversions", "true");
        }
        self.open(url).await
    }

    async fn open<T>(
        &self,
        url: Url,
    ) -> Result<impl Stream<Item = Result<T, APIError>> + Send + 'static, APIError>
    where
        T: DeserializeOwned + Send + 'static,
    {
        let response = self.connection.http_client.get(url).send().await?;
        if response.status() != reqwest::StatusCode::OK {
            return Err(
                http::decode_response::<()>(response, reqwest::StatusCode::OK, None)
                    .await
                    .unwrap_err(),
            );
        }
        Ok(frame_ndjson(
            response
                .bytes_stream()
                .map(|chunk| chunk.map_err(APIError::from)),
        ))
    }
}

async fn consume_items<T, S, F>(stream: S, mut handler: F) -> Result<(), APIError>
where
    S: Stream<Item = Result<T, APIError>>,
    F: FnMut(T) -> Result<(), APIError>,
{
    futures_util::pin_mut!(stream);
    while let Some(item) = stream.next().await {
        handler(item?)?;
    }
    Ok(())
}

/// Largest single stream message accepted. OANDA's pricing and transaction
/// messages are a few kilobytes; the limit bounds memory if a line never ends.
const MAX_MESSAGE_BYTES: usize = 1 << 20;

/// Frames messages independently of HTTP chunk boundaries, including a final
/// message without a newline. Incomplete JSON at EOF, or a message longer than
/// [`MAX_MESSAGE_BYTES`], is reported as an error. The source is fused, so it
/// is not polled again after it ends.
struct FrameState<S, B> {
    stream: Pin<Box<Fuse<S>>>,
    buffer: Vec<u8>,
    chunk: Option<B>,
    offset: usize,
}

fn frame_ndjson<T, S, B>(stream: S) -> impl Stream<Item = Result<T, APIError>> + Send
where
    T: DeserializeOwned + Send,
    S: Stream<Item = Result<B, APIError>> + Send,
    B: AsRef<[u8]> + Send,
{
    futures_util::stream::try_unfold(
        FrameState {
            stream: Box::pin(stream.fuse()),
            buffer: Vec::new(),
            chunk: None,
            offset: 0,
        },
        |mut state: FrameState<S, B>| async move {
            loop {
                let chunk_len = state.chunk.as_ref().map_or(0, |chunk| chunk.as_ref().len());
                if state.offset == chunk_len {
                    match state.stream.next().await {
                        Some(chunk) => {
                            state.chunk = Some(chunk?);
                            state.offset = 0;
                        }
                        None => {
                            let line = state.buffer.trim_ascii();
                            return if line.is_empty() {
                                Ok(None)
                            } else {
                                let item = serde_json::from_slice::<T>(line)?;
                                state.buffer.clear();
                                Ok(Some((item, state)))
                            };
                        }
                    }
                }
                let Some(chunk) = &state.chunk else {
                    continue;
                };
                let remaining = &chunk.as_ref()[state.offset..];
                let end = remaining.iter().position(|byte| *byte == b'\n');
                let length = end.map_or(remaining.len(), |position| position + 1);
                state.buffer.extend_from_slice(&remaining[..length]);
                state.offset += length;
                if state.buffer.len() > MAX_MESSAGE_BYTES {
                    return Err(APIError::JSONError(
                        <serde_json::Error as serde::de::Error>::custom(format!(
                            "stream message exceeds {MAX_MESSAGE_BYTES} bytes"
                        )),
                    ));
                }
                if end.is_some() {
                    let line = state.buffer.trim_ascii();
                    let item = if line.is_empty() {
                        None
                    } else {
                        Some(serde_json::from_slice::<T>(line)?)
                    };
                    state.buffer.clear();
                    if let Some(item) = item {
                        return Ok(Some((item, state)));
                    }
                }
            }
        },
    )
}

#[cfg(test)]
async fn consume_ndjson<T, S, B, F>(stream: S, handler: F) -> Result<(), APIError>
where
    T: DeserializeOwned + Send,
    S: Stream<Item = Result<B, APIError>> + Send,
    B: AsRef<[u8]> + Send,
    F: FnMut(T) -> Result<(), APIError>,
{
    consume_items(frame_ndjson(stream), handler).await
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
    async fn oversized_message_is_an_error_before_dispatch() {
        // A valid JSON string, so only the size limit can reject it.
        let wire = [b"\"".as_slice(), &vec![b'a'; MAX_MESSAGE_BYTES], b"\""].concat();
        let chunks = stream::iter(wire.chunks(64 * 1024).map(Ok::<_, APIError>));
        let mut count = 0;
        let result = consume_ndjson::<Value, _, _, _>(chunks, |_| {
            count += 1;
            Ok(())
        })
        .await;
        assert_eq!(count, 0);
        assert!(
            matches!(result, Err(APIError::JSONError(error)) if error.to_string().contains("exceeds"))
        );
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
