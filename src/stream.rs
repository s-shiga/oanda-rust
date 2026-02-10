use crate::account::AccountID;
use crate::errors::APIError;
use crate::transaction::TransactionStreamItem;
use futures_util::stream::StreamExt;
use reqwest::header::{HeaderMap, HeaderValue, ACCEPT, AUTHORIZATION};
use reqwest::Request;
use url::Url;

pub struct StreamClient {
    pub(crate) base_url: Url,
    pub(crate) http_client: reqwest::Client,
    pub(crate) account_id: Option<AccountID>,
}

const FX_TRADE_PRACTICE_STREAMING_URL: &str = "https://stream-fxpractice.oanda.com";
const FX_TRADE_STREAMING_URL: &str = "https://stream-fxtrade.oanda.com";

impl<'a> StreamClient {
    #[allow(unused)]
    pub fn new(api_key: &str) -> Self {
        StreamClient {
            base_url: Url::parse(FX_TRADE_STREAMING_URL).unwrap(),
            http_client: StreamClient::build_client(api_key),
            account_id: None,
        }
    }

    #[allow(unused)]
    pub fn new_practice(api_key: &str) -> Self {
        StreamClient {
            base_url: Url::parse(FX_TRADE_PRACTICE_STREAMING_URL).unwrap(),
            http_client: StreamClient::build_client(api_key),
            account_id: None,
        }
    }

    fn build_client(api_key: &str) -> reqwest::Client {
        let mut headers = HeaderMap::new();
        headers.insert(ACCEPT, "application/octet-stream".parse().unwrap());
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(format!("Bearer {}", api_key).as_str()).unwrap(),
        );
        reqwest::ClientBuilder::new()
            .default_headers(headers)
            .build()
            .unwrap()
    }

    pub fn with_account_id(mut self, account_id: AccountID) -> Self {
        self.account_id = Some(account_id);
        self
    }

    pub async fn stream_transactions(
        &self,
        handler: fn(TransactionStreamItem) -> Result<(), APIError>,
    ) -> Result<(), APIError> {
        let url = self
            .base_url
            .join(
                format!(
                    "/v3/accounts/{}/transactions/stream",
                    self.account_id
                        .as_ref()
                        .expect("Missing account_id in client")
                )
                .as_str(),
            )
            .unwrap();
        let http_req = Request::new(reqwest::Method::GET, url);
        let http_resp = self
            .http_client
            .execute(http_req)
            .await?
            .error_for_status()?;
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
}

#[cfg(test)]
mod tests {
    use crate::stream::StreamClient;
    use std::time::Duration;
    use tokio::time::timeout;

    fn setup() -> StreamClient {
        let api_key = env!("OANDA_API_KEY_DEMO");
        let account_id = env!("OANDA_ACCOUNT_ID_DEMO").to_string();
        StreamClient::new_practice(api_key).with_account_id(account_id)
    }

    #[tokio::test]
    async fn test_stream_transactions() {
        let client = setup();
        let _ = timeout(
            Duration::from_secs(10),
            client.stream_transactions(|item| {
                println!("{:#?}", item);
                Ok(())
            }),
        )
        .await;
    }
}
