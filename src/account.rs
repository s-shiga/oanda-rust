use crate::instrument::Instrument;
use crate::transaction::TransactionID;
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
