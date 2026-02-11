use crate::account::{AccountID, AccountService};
use crate::errors::APIError;
use crate::instrument::InstrumentService;
use crate::order::{ListOrdersRequest, ListOrdersResponse};
use crate::position::PositionService;
use crate::transaction::TransactionService;
use reqwest::header::{HeaderMap, HeaderValue, ACCEPT, AUTHORIZATION};
use reqwest::{Request, StatusCode};
use url::Url;

const FX_TRADE_PRACTICE_URL: &str = "https://api-fxpractice.oanda.com";
const FX_TRADE_URL: &str = "https://api-fxtrade.oanda.com";

pub struct Client {
    pub(crate) base_url: Url,
    #[allow(unused)]
    pub(crate) http_client: reqwest::Client,
    pub(crate) account_id: Option<AccountID>,
}

impl<'a> Client {
    #[allow(unused)]
    pub fn new(api_key: &str) -> Client {
        Client {
            base_url: Url::parse(FX_TRADE_URL).unwrap(),
            http_client: Client::build_client(api_key),
            account_id: None,
        }
    }

    #[allow(unused)]
    pub fn new_practice(api_key: &str) -> Client {
        Client {
            base_url: Url::parse(FX_TRADE_PRACTICE_URL).unwrap(),
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

    pub fn account(&'a self) -> AccountService<'a> {
        AccountService::new(self)
    }

    pub fn instrument(&'a self) -> InstrumentService<'a> {
        InstrumentService::new(self)
    }

    pub fn transaction(&'a self) -> TransactionService<'a> {
        TransactionService::new(self)
    }

    pub fn position(&'a self) -> PositionService<'a> {
        PositionService::new(self)
    }

    pub async fn list_orders(
        &self,
        req: ListOrdersRequest,
    ) -> Result<ListOrdersResponse, APIError> {
        let mut url = self
            .base_url
            .join(
                format!(
                    "/v3/accounts/{}/orders",
                    self.account_id
                        .as_ref()
                        .expect("Missing account_id in client")
                )
                .as_str(),
            )
            .unwrap();
        req.set_params(&mut url);
        let http_req = Request::new(reqwest::Method::GET, url);
        let http_resp = self.http_client.execute(http_req).await?;
        match http_resp.status() {
            StatusCode::OK => {
                let resp = http_resp.json::<ListOrdersResponse>().await?;
                Ok(resp)
            }
            status => Err(APIError::ApiErrorResponse {
                status,
                message: http_resp.text().await?,
            }),
        }
    }
}

#[cfg(test)]
pub(crate) fn setup_test_client() -> Client {
    let api_key = env!("OANDA_API_KEY_DEMO");
    let account_id = env!("OANDA_ACCOUNT_ID_DEMO").to_string();
    Client::new_practice(api_key).with_account_id(account_id)
}

mod tests {
    use super::*;

    #[tokio::test]
    async fn test_list_orders() {
        let client = setup_test_client();
        let req = ListOrdersRequest::new().instrument(String::from("USD_JPY"));
        let resp = client.list_orders(req).await.unwrap();
        println!("{:#?}", resp);
    }
}
