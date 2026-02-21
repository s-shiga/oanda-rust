use crate::client::Client;
use crate::errors::APIError;
use crate::instrument::InstrumentName;
use crate::primitives::DecimalNumber;
use crate::transaction::{AccountUnits, TradeID, TransactionID};
use reqwest::{Method, Request, StatusCode};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Position {
    pub instrument: InstrumentName,
    pub pl: AccountUnits,
    #[serde(rename = "unrealizedPL")]
    pub unrealized_pl: AccountUnits,
    #[serde(rename = "marginUsed")]
    pub margin_used: Option<AccountUnits>,
    #[serde(rename = "resettablePL")]
    pub resettable_pl: AccountUnits,
    pub financing: AccountUnits,
    pub commission: AccountUnits,
    #[serde(rename = "dividendAdjustment")]
    pub dividend_adjustment: AccountUnits,
    #[serde(rename = "guaranteedExecutionFees")]
    pub guaranteed_execution_fees: AccountUnits,
    pub long: PositionSide,
    pub short: PositionSide,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PositionSide {
    pub units: DecimalNumber,
    #[serde(rename = "averagePrice")]
    pub average_price: Option<DecimalNumber>,
    #[serde(rename = "tradeIDs")]
    pub trade_ids: Option<Vec<TradeID>>,
    pub pl: AccountUnits,
    #[serde(rename = "unrealizedPL")]
    pub unrealized_pl: AccountUnits,
    #[serde(rename = "resettablePL")]
    pub resettable_pl: AccountUnits,
    pub financing: AccountUnits,
    #[serde(rename = "dividendAdjustment")]
    pub dividend_adjustment: AccountUnits,
    #[serde(rename = "guaranteedExecutionFees")]
    pub guaranteed_execution_fees: AccountUnits,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CalculatedPositionState {
    pub instrument: InstrumentName,
    #[serde(rename = "netUnrealizedPL")]
    pub net_unrealized_pl: AccountUnits,
    #[serde(rename = "longUnrealizedPL")]
    pub long_unrealized_pl: AccountUnits,
    #[serde(rename = "shortUnrealizedPL")]
    pub short_unrealized_pl: AccountUnits,
    #[serde(rename = "marginUsed")]
    pub margin_used: AccountUnits,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PositionListResponse {
    pub positions: Vec<Position>,
    #[serde(rename = "lastTransactionID")]
    pub last_transaction_id: TransactionID,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PositionDetailsResponse {
    pub position: Position,
    #[serde(rename = "lastTransactionID")]
    pub last_transaction_id: TransactionID,
}

pub struct PositionService<'a> {
    client: &'a Client,
}

impl<'a> PositionService<'a> {
    pub(crate) fn new(client: &'a Client) -> Self {
        Self { client }
    }

    pub async fn list(&self) -> Result<PositionListResponse, APIError> {
        let url = self
            .client
            .base_url
            .join(
                format!(
                    "/v3/accounts/{}/positions",
                    self.client.account_id.as_ref().expect("Missing account_id")
                )
                .as_str(),
            )
            .unwrap();
        let http_req = Request::new(Method::GET, url);
        let http_resp = self.client.http_client.execute(http_req).await?;
        match http_resp.status() {
            StatusCode::OK => {
                let resp = http_resp.json::<PositionListResponse>().await?;
                Ok(resp)
            }
            status => Err(APIError::ApiErrorResponse {
                status,
                message: http_resp.text().await?,
            }),
        }
    }

    pub async fn list_open(&self) -> Result<PositionListResponse, APIError> {
        let url = self
            .client
            .base_url
            .join(
                format!(
                    "/v3/accounts/{}/openPositions",
                    self.client.account_id.as_ref().expect("Missing account_id")
                )
                .as_str(),
            )
            .unwrap();
        let http_req = Request::new(Method::GET, url);
        let http_resp = self.client.http_client.execute(http_req).await?;
        match http_resp.status() {
            StatusCode::OK => {
                let resp = http_resp.json::<PositionListResponse>().await?;
                Ok(resp)
            }
            status => Err(APIError::ApiErrorResponse {
                status,
                message: http_resp.text().await?,
            }),
        }
    }

    pub async fn details(
        &self,
        instrument: InstrumentName,
    ) -> Result<PositionDetailsResponse, APIError> {
        let url = self
            .client
            .base_url
            .join(
                format!(
                    "/v3/accounts/{}/positions/{}",
                    self.client.account_id.as_ref().expect("Missing account_id"),
                    instrument
                )
                .as_str(),
            )
            .unwrap();
        let http_req = Request::new(Method::GET, url);
        let http_resp = self.client.http_client.execute(http_req).await?;
        match http_resp.status() {
            StatusCode::OK => {
                let resp = http_resp.json::<PositionDetailsResponse>().await?;
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
    async fn test_position_list() {
        let client = setup_test_client();
        let positions = client.position().list().await.unwrap();
        println!("{:#?}", positions);
    }

    #[tokio::test]
    async fn test_position_list_open() {
        let client = setup_test_client();
        let positions = client.position().list_open().await.unwrap();
        println!("{:#?}", positions);
    }

    #[tokio::test]
    async fn test_position_details() {
        let client = setup_test_client();
        let details = client
            .position()
            .details(String::from("USD_JPY"))
            .await
            .unwrap();
        println!("{:#?}", details);
    }
}
