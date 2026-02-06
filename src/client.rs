use url::Url;
use reqwest::header::{HeaderMap, HeaderValue, ACCEPT, AUTHORIZATION};
use crate::account::{AccountID, AccountService};

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
        headers.insert(AUTHORIZATION, HeaderValue::from_str(format!("Bearer {}", api_key).as_str()).unwrap());
        reqwest::ClientBuilder::new().default_headers(headers).build().unwrap()
    }
    
    pub fn with_account_id(mut self, account_id: AccountID) -> Self {
        self.account_id = Some(account_id);
        self
    }

    pub fn account(&self) -> AccountService<'_> {
        AccountService {client: self}
    }
}
