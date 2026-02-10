use crate::account::{AccountID, ListAccountsResponse, ListInstrumentsResponse};
use crate::errors::APIError;
use crate::instrument::{FetchCandlestickDataRequest, FetchCandlestickDataResponse};
use crate::order::{ListOrdersRequest, ListOrdersResponse};
use crate::position::PositionService;
use crate::transaction::{
    GetTransactionDetailsResponse, GetTransactionsByIDRangeRequest,
    GetTransactionsBySinceIDRequest, GetTransactionsResponse, ListTransactionsRequest,
    ListTransactionsResponse, TransactionID,
};
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

    pub fn position(&'a self) -> PositionService<'a> {
        PositionService::new(self)
    }

    pub async fn list_accounts(&self) -> Result<ListAccountsResponse, APIError> {
        let url = self.base_url.join("/v3/accounts").unwrap();
        let http_req = Request::new(reqwest::Method::GET, url);
        let http_resp = self.http_client.execute(http_req).await?;
        match http_resp.status() {
            StatusCode::OK => {
                let resp = http_resp.json::<ListAccountsResponse>().await?;
                Ok(resp)
            }
            status => Err(APIError::ApiErrorResponse {
                status,
                message: http_resp.text().await?,
            }),
        }
    }

    pub async fn list_instruments(&self) -> Result<ListInstrumentsResponse, APIError> {
        let url = self
            .base_url
            .join(
                format!(
                    "/v3/accounts/{}/instruments",
                    self.account_id
                        .as_ref()
                        .expect("Missing account_id in client")
                )
                .as_str(),
            )
            .unwrap();
        let http_req = Request::new(reqwest::Method::GET, url);
        let http_resp = self.http_client.execute(http_req).await?;
        match http_resp.status() {
            StatusCode::OK => {
                let resp = http_resp.json::<ListInstrumentsResponse>().await?;
                Ok(resp)
            }
            status => Err(APIError::ApiErrorResponse {
                status,
                message: http_resp.text().await?,
            }),
        }
    }
    pub async fn fetch_candlestick_data(
        &self,
        req: FetchCandlestickDataRequest,
    ) -> Result<FetchCandlestickDataResponse, APIError> {
        let mut url = self
            .base_url
            .join(format!("/v3/instruments/{}/candles", req.instrument).as_str())
            .unwrap();
        req.set_params(&mut url);
        let http_req = Request::new(reqwest::Method::GET, url);
        let http_resp = self.http_client.execute(http_req).await?;
        match http_resp.status() {
            StatusCode::OK => {
                let resp = http_resp.json::<FetchCandlestickDataResponse>().await?;
                Ok(resp)
            }
            status => Err(APIError::ApiErrorResponse {
                status,
                message: http_resp.text().await?,
            }),
        }
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

    pub async fn list_transactions(
        &self,
        req: ListTransactionsRequest,
    ) -> Result<ListTransactionsResponse, APIError> {
        let mut url = self
            .base_url
            .join(
                format!(
                    "/v3/accounts/{}/transactions",
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
                let resp = http_resp.json::<ListTransactionsResponse>().await?;
                Ok(resp)
            }
            status => Err(APIError::ApiErrorResponse {
                status,
                message: http_resp.text().await?,
            }),
        }
    }

    pub async fn get_transaction_details(
        &self,
        id: TransactionID,
    ) -> Result<GetTransactionDetailsResponse, APIError> {
        let url = self
            .base_url
            .join(
                format!(
                    "/v3/accounts/{}/transactions/{}",
                    self.account_id
                        .as_ref()
                        .expect("Missing account_id in client"),
                    id
                )
                .as_str(),
            )
            .unwrap();
        let http_req = Request::new(reqwest::Method::GET, url);
        let http_resp = self.http_client.execute(http_req).await?;
        match http_resp.status() {
            StatusCode::OK => {
                let resp = http_resp.json::<GetTransactionDetailsResponse>().await?;
                Ok(resp)
            }
            status => Err(APIError::ApiErrorResponse {
                status,
                message: http_resp.text().await?,
            }),
        }
    }

    pub async fn get_transactions_by_id_range(
        &self,
        req: GetTransactionsByIDRangeRequest,
    ) -> Result<GetTransactionsResponse, APIError> {
        let mut url = self
            .base_url
            .join(
                format!(
                    "/v3/accounts/{}/transactions/idrange",
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
                let resp = http_resp.json::<GetTransactionsResponse>().await?;
                Ok(resp)
            }
            status => Err(APIError::ApiErrorResponse {
                status,
                message: http_resp.text().await?,
            }),
        }
    }

    pub async fn get_transactions_by_since_id(
        &self,
        req: GetTransactionsBySinceIDRequest,
    ) -> Result<GetTransactionsResponse, APIError> {
        let mut url = self
            .base_url
            .join(
                format!(
                    "/v3/accounts/{}/transactions/sinceid",
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
                let resp = http_resp.json::<GetTransactionsResponse>().await?;
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
    use crate::instrument::CandlestickGranularity;

    #[tokio::test]
    async fn test_list() {
        let client = setup_test_client();
        let account = client.list_accounts().await.unwrap();
        println!("{:#?}", account);
    }

    #[tokio::test]
    async fn test_list_instruments() {
        let client = setup_test_client();
        let resp = client.list_instruments().await.unwrap();
        println!("{:#?}", resp);
    }

    #[tokio::test]
    async fn test_fetch_candlestick_data() {
        let client = setup_test_client();
        let req = FetchCandlestickDataRequest::new("USD_JPY".to_string())
            .granularity(CandlestickGranularity::M1)
            .count(50)
            .unwrap();
        let resp = client.fetch_candlestick_data(req).await.unwrap();
        println!("{:#?}", resp);
    }

    #[tokio::test]
    async fn test_list_orders() {
        let client = setup_test_client();
        let req = ListOrdersRequest::new().instrument(String::from("USD_JPY"));
        let resp = client.list_orders(req).await.unwrap();
        println!("{:#?}", resp);
    }

    #[tokio::test]
    async fn test_list_transactions() {
        let client = setup_test_client();
        let req = ListTransactionsRequest::new();
        let resp = client.list_transactions(req).await.unwrap();
        println!("{:#?}", resp);
    }

    #[tokio::test]
    async fn test_get_transaction_details() {
        let client = setup_test_client();
        let resp = client
            .get_transaction_details("456".to_string())
            .await
            .unwrap();
        println!("{:#?}", resp);
    }

    #[tokio::test]
    async fn test_get_transactions_by_id_range() {
        let client = setup_test_client();
        let req = GetTransactionsByIDRangeRequest::new("500".to_string(), "510".to_string());
        let resp = client.get_transactions_by_id_range(req).await.unwrap();
        println!("{:#?}", resp);
    }

    #[tokio::test]
    async fn test_get_transactions_by_since_id() {
        let client = setup_test_client();
        let req = GetTransactionsBySinceIDRequest::new("500".to_string());
        let resp = client.get_transactions_by_since_id(req).await.unwrap();
        println!("{:#?}", resp);
    }
}
