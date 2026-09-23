use super::*;

/// Response body for `GET /v3/accounts/{accountID}/transactions` (HTTP 200).
///
/// The actual transactions are not embedded here; instead, `pages` contains
/// URLs for fetching each page of transactions individually.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListTransactionsResponse {
    /// Total number of transactions matching the query.
    pub count: i64,
    /// Start of the time range covered by this response.
    pub from: DateTime<Utc>,
    /// End of the time range covered by this response.
    pub to: DateTime<Utc>,
    /// Maximum number of transactions per page used for this response.
    pub page_size: i64,
    /// URLs for each page of results. Fetch each URL to retrieve the transactions.
    pub pages: Vec<String>,
    /// ID of the most recent transaction on the account.
    #[serde(rename = "lastTransactionID")]
    pub last_transaction_id: TransactionID,
}

/// Response body for `GET /v3/accounts/{accountID}/transactions/{transactionID}` (HTTP 200).
#[derive(Debug, Serialize, Deserialize)]
pub struct GetTransactionDetailsResponse {
    /// The requested transaction.
    pub transaction: Transaction,
    /// ID of the most recent transaction on the account.
    #[serde(rename = "lastTransactionID")]
    pub last_transaction_id: TransactionID,
}

/// Response body for `GET /v3/accounts/{accountID}/transactions/idrange` and
/// `GET /v3/accounts/{accountID}/transactions/sinceid` (HTTP 200).
#[derive(Debug, Serialize, Deserialize)]
pub struct GetTransactionsResponse {
    /// The transactions matching the request.
    pub transactions: Vec<Transaction>,
    /// ID of the most recent transaction on the account.
    #[serde(rename = "lastTransactionID")]
    pub last_transaction_id: TransactionID,
}
