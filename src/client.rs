use crate::account::{AccountID, ListAccountsResponse, ListInstrumentsResponse};
use crate::errors::APIError;
use reqwest::header::{ACCEPT, AUTHORIZATION, HeaderMap, HeaderValue};
use reqwest::{Request, StatusCode};
use url::Url;

const FX_TRADE_PRACTICE_URL: &str = "https://api-fxpractice.oanda.com";
const FX_TRADE_URL: &str = "https://api-fxtrade.oanda.com";
const FX_TRADE_PRACTICE_STREAMING_URL: &str = "https://stream-fxpractice.oanda.com";
const FX_TRADE_STREAMING_URL: &str = "https://stream-fxtrade.oanda.com";

pub struct Client {
    pub(crate) base_url: Url,
    #[allow(unused)]
    pub(crate) base_streaming_url: Url,
    pub(crate) http_client: reqwest::Client,
    pub(crate) account_id: Option<AccountID>,
}

impl Client {
    #[allow(unused)]
    pub fn new(api_key: &'static str) -> Client {
        Client {
            base_url: Url::parse(FX_TRADE_URL).unwrap(),
            base_streaming_url: Url::parse(FX_TRADE_STREAMING_URL).unwrap(),
            http_client: Client::build_client(api_key),
            account_id: None,
        }
    }

    #[allow(unused)]
    pub fn new_practice(api_key: &'static str) -> Client {
        Client {
            base_url: Url::parse(FX_TRADE_PRACTICE_URL).unwrap(),
            base_streaming_url: Url::parse(FX_TRADE_PRACTICE_STREAMING_URL).unwrap(),
            http_client: Client::build_client(api_key),
            account_id: None,
        }
    }

    fn build_client(api_key: &str) -> reqwest::Client {
        let mut headers = HeaderMap::new();
        headers.insert(ACCEPT, "application/json".parse().unwrap());
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

    pub async fn list_accounts(&self) -> Result<ListAccountsResponse, APIError> {
        let url = self.base_url.join("/v3/accounts").unwrap();
        let request = Request::new(reqwest::Method::GET, url);
        let resp = self.http_client.execute(request).await?;
        match resp.status() {
            StatusCode::OK => {
                let list_account_response = resp.json::<ListAccountsResponse>().await?;
                Ok(list_account_response)
            }
            status => Err(APIError::ApiErrorResponse {
                status,
                message: resp.text().await?,
            }),
        }
    }

    pub async fn list_instruments(&self) -> Result<ListInstrumentsResponse, APIError> {
        let url = self
            .base_url
            .join(
                format!(
                    "/v3/accounts/{}/instruments",
                    self.account_id.as_ref().expect("Missing client account_id")
                )
                .as_str(),
            )
            .unwrap();
        let request = Request::new(reqwest::Method::GET, url);
        let resp = self.http_client.execute(request).await?;
        match resp.status() {
            StatusCode::OK => {
                let list_instruments_response = resp.json::<ListInstrumentsResponse>().await?;
                Ok(list_instruments_response)
            }
            status => Err(APIError::ApiErrorResponse {
                status,
                message: resp.text().await?,
            }),
        }
    }
}
