use crate::client::Client;
use crate::errors::APIError;
use crate::instrument::Instrument;
use crate::transaction::TransactionID;
use reqwest::{Request, StatusCode};
use serde::{Deserialize, Serialize};

pub type AccountID = String;

#[derive(Serialize, Deserialize, Debug)]
pub struct AccountProperties {
    pub id: String,
    pub mt4account: Option<String>,
    pub tags: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ListAccountsResponse {
    pub accounts: Vec<AccountProperties>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ListInstrumentsResponse {
    pub instruments: Vec<Instrument>,
    #[serde(rename = "lastTransactionID")]
    pub last_transaction_id: TransactionID,
}

pub struct AccountService<'a> {
    client: &'a Client,
}

impl<'a> AccountService<'a> {
    pub fn new(client: &'a Client) -> Self {
        Self { client }
    }

    pub async fn list(&self) -> Result<ListAccountsResponse, APIError> {
        let url = self.client.base_url.join("/v3/accounts").unwrap();
        let http_req = Request::new(reqwest::Method::GET, url);
        let http_resp = self.client.http_client.execute(http_req).await?;
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
            .client
            .base_url
            .join(
                format!(
                    "/v3/accounts/{}/instruments",
                    self.client
                        .account_id
                        .as_ref()
                        .expect("Missing account_id in client")
                )
                .as_str(),
            )
            .unwrap();
        let http_req = Request::new(reqwest::Method::GET, url);
        let http_resp = self.client.http_client.execute(http_req).await?;
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
}

#[cfg(test)]
mod tests {
    use crate::client::setup_test_client;

    #[tokio::test]
    async fn test_list() {
        let client = setup_test_client();
        let account = client.account().list().await.unwrap();
        println!("{:#?}", account);
    }

    #[tokio::test]
    async fn test_list_instruments() {
        let client = setup_test_client();
        let resp = client.account().list_instruments().await.unwrap();
        println!("{:#?}", resp);
    }
}
