use crate::client::Client;
use crate::errors::APIError;
use crate::instrument::InstrumentName;
use crate::transaction::{ClientExtensions, ClientID, TransactionID};
use chrono::{DateTime, Utc};
use reqwest::{Request, StatusCode};
use serde::{Deserialize, Serialize};
use std::ops::Not;
use strum_macros::Display;
use url::Url;

type OrderID = u16;

#[derive(Debug, Serialize, Deserialize)]
pub enum Order {
    MarketOrder(MarketOrder),
}

#[derive(Debug, Deserialize, Serialize)]
pub struct MarketOrder {
    pub id: OrderID,
    #[serde(rename = "createTime")]
    pub create_time: DateTime<Utc>,
    pub state: OrderState,
    #[serde(rename = "clientExtensions", skip_serializing_if = "Option::is_none")]
    pub client_extensions: Option<ClientExtensions>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum OrderType {
    Market,
    Limit,
    Stop,
    MarketIfTouched,
    TakeProfit,
    StopLoss,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum OrderState {
    Pending,
    Filled,
    Triggered,
    Cancelled,
}

#[derive(Debug, Serialize, Deserialize, Display)]
#[serde(rename_all = "UPPERCASE")]
pub enum OrderStateFilter {
    Pending,
    Filled,
    Triggered,
    Cancelled,
    All,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OrderIdentifier {
    #[serde(rename = "orderID")]
    pub order_id: OrderID,
    #[serde(rename = "clientOrderID")]
    pub client_order_id: ClientID,
}

pub type OrderSpecifier = String;

#[derive(Debug, Serialize, Deserialize)]
pub enum TimeInForce {
    GTC,
    GTD,
    GFD,
    FOK,
    IOC,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum OrderPositionFill {
    OpenOnly,
    ReduceFirst,
    ReduceOnly,
    Default,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum OrderTriggerCondition {
    Default,
    Inverse,
    Bid,
    Ask,
    Mid,
}

pub struct ListOrdersRequest {
    ids: Vec<OrderID>,
    state: Option<OrderStateFilter>,
    instrument: Option<InstrumentName>,
    count: Option<u16>,
    before_id: Option<OrderID>,
}

impl ListOrdersRequest {
    pub fn new() -> Self {
        ListOrdersRequest {
            ids: Vec::new(),
            state: None,
            instrument: None,
            count: None,
            before_id: None,
        }
    }

    pub fn ids(mut self, id: OrderID) -> Self {
        self.ids.push(id);
        self
    }

    pub fn state(mut self, state: OrderStateFilter) -> Self {
        self.state = Some(state);
        self
    }

    pub fn instrument(mut self, instrument: InstrumentName) -> Self {
        self.instrument = Some(instrument);
        self
    }

    pub fn count(mut self, count: u16) -> Self {
        self.count = Some(count);
        self
    }

    pub fn before_id(mut self, before_id: OrderID) -> Self {
        self.before_id = Some(before_id);
        self
    }

    pub fn set_params(&self, url: &mut Url) {
        self.ids.is_empty().not().then(|| {
            url.query_pairs_mut().append_pair(
                "ids",
                self.ids
                    .iter()
                    .map(|id| id.to_string())
                    .collect::<Vec<String>>()
                    .join(",")
                    .as_str(),
            );
        });
        self.state.is_some().then(|| {
            url.query_pairs_mut()
                .append_pair("state", self.state.as_ref().unwrap().to_string().as_str());
        });
        self.instrument.is_some().then(|| {
            url.query_pairs_mut()
                .append_pair("instrument", self.instrument.as_ref().unwrap().as_str());
        });
        self.count.is_some().then(|| {
            url.query_pairs_mut()
                .append_pair("count", &self.count.unwrap().to_string().as_str());
        });
        self.before_id.is_some().then(|| {
            url.query_pairs_mut()
                .append_pair("beforeID", &self.before_id.unwrap().to_string().as_str());
        });
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ListOrdersResponse {
    pub orders: Vec<Order>,
    #[serde(rename = "lastTransactionID")]
    pub last_transaction_id: TransactionID,
}

pub struct OrderService<'a> {
    client: &'a Client,
}

impl<'a> OrderService<'a> {
    pub fn new(client: &'a Client) -> Self {
        OrderService { client }
    }

    pub async fn list(&self, req: ListOrdersRequest) -> Result<ListOrdersResponse, APIError> {
        let mut url = self
            .client
            .base_url
            .join(
                format!(
                    "/v3/accounts/{}/orders",
                    self.client
                        .account_id
                        .as_ref()
                        .expect("Missing account_id in client")
                )
                .as_str(),
            )
            .unwrap();
        req.set_params(&mut url);
        let http_req = Request::new(reqwest::Method::GET, url);
        let http_resp = self.client.http_client.execute(http_req).await?;
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
mod tests {
    use crate::client::setup_test_client;
    use crate::order::ListOrdersRequest;

    #[tokio::test]
    async fn test_list_orders() {
        let client = setup_test_client();
        let req = ListOrdersRequest::new().instrument(String::from("USD_JPY"));
        let resp = client.order().list(req).await.unwrap();
        println!("{:#?}", resp);
    }
}
