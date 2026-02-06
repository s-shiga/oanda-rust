use crate::Client;
use crate::errors::APIError;
use crate::instrument::Instrument;
use crate::transaction::TransactionId;
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
    pub last_transaction_id: TransactionId,
}

pub struct AccountService<'a> {
    pub(crate) client: &'a Client,
}

impl<'a> AccountService<'a> {
    pub async fn list(&self) -> Result<ListAccountsResponse, APIError> {
        let url = self.client.base_url.join("/v3/accounts").unwrap();
        let request = Request::new(reqwest::Method::GET, url);
        let resp = self.client.http_client.execute(request).await?;
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
            .client
            .base_url
            .join(
                format!(
                    "/v3/accounts/{}/instruments",
                    self.client.account_id.as_ref().unwrap()
                )
                .as_str(),
            )
            .unwrap();
        let request = Request::new(reqwest::Method::GET, url);
        let resp = self.client.http_client.execute(request).await?;
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

#[cfg(test)]
mod tests {
    use super::*;

    fn setup() -> Client {
        let api_key = env!("OANDA_API_KEY_DEMO");
        let account_id = env!("OANDA_ACCOUNT_ID_DEMO").to_string();
        Client::new_practice(api_key).with_account_id(account_id)
    }

    #[tokio::test]
    async fn test_list() {
        let client = setup();
        let account = client.account().list().await.unwrap();
        println!("{:#?}", account);
    }

    #[tokio::test]
    async fn test_list_instruments() {
        let client = setup();
        let resp = client.account().list_instruments().await.unwrap();
        println!("{:#?}", resp);
    }
}
