use crate::account;
use crate::errors::APIError;
use url::Url;
use reqwest::{Request, StatusCode};
use reqwest::header::{HeaderMap, HeaderValue, ACCEPT, AUTHORIZATION};
use crate::account::ListAccountsResponse;

const FX_TRADE_PRACTICE_URL: &str = "https://api-fxpractice.oanda.com";
const FX_TRADE_URL: &str = "https://api-fxtrade.oanda.com";
const FX_TRADE_PRACTICE_STREAMING_URL: &str = "https://stream-fxpractice.oanda.com";
const FX_TRADE_STREAMING_URL: &str = "https://stream-fxtrade.oanda.com";

pub struct Client {
    #[allow(unused)]
    base_url: Url,
    #[allow(unused)]
    base_streaming_url: Url,
    client: reqwest::Client,
}

impl Client {
    #[allow(unused)]
    pub fn new(api_key: &'static str) -> Client {
        Client {
            base_url: Url::parse(FX_TRADE_URL).unwrap(),
            base_streaming_url: Url::parse(FX_TRADE_STREAMING_URL).unwrap(),
            client: Client::build_client(api_key),
        }
    }

    #[allow(unused)]
    pub fn new_practice(api_key: &'static str) -> Client {
        Client {
            base_url: Url::parse(FX_TRADE_PRACTICE_URL).unwrap(),
            base_streaming_url: Url::parse(FX_TRADE_PRACTICE_STREAMING_URL).unwrap(),
            client: Client::build_client(api_key),
        }
    }
    
    fn build_client(api_key: &str) -> reqwest::Client {
        let mut headers = HeaderMap::new();
        headers.insert(ACCEPT, "application/json".parse().unwrap());
        headers.insert(AUTHORIZATION, HeaderValue::from_str(format!("Bearer {}", api_key).as_str()).unwrap());
        reqwest::ClientBuilder::new().default_headers(headers).build().unwrap()
    }

    pub async fn list_accounts(&self) -> Result<Vec<account::AccountProperties>, APIError> {
        let url = self.base_url.join("/v3/accounts").unwrap();
        let request = Request::new(reqwest::Method::GET, url);
        let resp = self.client.execute(request).await?;
        match resp.status() {
            StatusCode::OK => {
                let list_account_response = resp.json::<ListAccountsResponse>().await?;
                Ok(list_account_response.accounts)
            }
            status => {
                Err(APIError::ApiErrorResponse {status, message: resp.text().await?})
            }
        }
    }
}
