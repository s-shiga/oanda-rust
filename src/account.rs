use reqwest::{Request, StatusCode};
use crate::Client;
use serde::{Deserialize, Serialize};
use crate::errors::APIError;

#[derive(Debug, Serialize, Deserialize)]
pub struct ListAccountsResponse {
    pub accounts: Vec<AccountProperties>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AccountProperties {
    pub id: String,
    pub mt4account: Option<String>,
    pub tags: Vec<String>,
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
            status => {
                Err(APIError::ApiErrorResponse {status, message: resp.text().await?})
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_list() {
        let api_key = env!("OANDA_API_KEY_DEMO");
        let client = Client::new_practice(api_key);
        let account = client.account().list().await.unwrap();
        println!("{:?}", account);
    }
}
