use super::*;

/// Provides access to the OANDA Transaction endpoints
/// (`/v3/accounts/{id}/transactions/...`).
///
/// Obtain an instance via [`Client::transaction`](crate::client::Client::transaction).
pub struct TransactionService<'a> {
    connection: &'a Connection,
}

impl<'a> TransactionService<'a> {
    pub(crate) fn new(connection: &'a Connection) -> Self {
        TransactionService { connection }
    }

    /// Lists transactions on the account, optionally filtered by time range or type.
    ///
    /// Calls `GET /v3/accounts/{accountID}/transactions`.
    ///
    /// Returns page URLs rather than inline transactions; follow each URL in
    /// [`ListTransactionsResponse::pages`] to retrieve the actual transaction data.
    ///
    /// Returns [`APIError::InvalidRequest`] if no account ID is configured.
    pub async fn list(
        &self,
        req: ListTransactionsRequest,
    ) -> Result<ListTransactionsResponse, APIError> {
        let mut url = self.connection.account_url("transactions")?;
        req.set_params(&mut url);
        self.connection.http_client.get_json(url).await
    }

    /// Returns the details of the transaction identified by `id`.
    ///
    /// Calls `GET /v3/accounts/{accountID}/transactions/{transactionID}`.
    ///
    /// Returns [`APIError::InvalidRequest`] if no account ID is configured.
    pub async fn get_details(
        &self,
        id: TransactionID,
    ) -> Result<GetTransactionDetailsResponse, APIError> {
        let url = self
            .connection
            .account_url(&format!("transactions/{}", id))?;
        self.connection.http_client.get_json(url).await
    }

    /// Returns all transactions with IDs in the inclusive range specified by `req`.
    ///
    /// Calls `GET /v3/accounts/{accountID}/transactions/idrange`.
    ///
    /// Returns [`APIError::InvalidRequest`] if no account ID is configured.
    pub async fn get_by_id_range(
        &self,
        req: GetTransactionsByIDRangeRequest,
    ) -> Result<GetTransactionsResponse, APIError> {
        let mut url = self.connection.account_url("transactions/idrange")?;
        req.set_params(&mut url);
        self.connection.http_client.get_json(url).await
    }

    /// Returns all transactions with IDs greater than the one specified in `req`.
    ///
    /// Calls `GET /v3/accounts/{accountID}/transactions/sinceid`.
    ///
    /// Returns [`APIError::InvalidRequest`] if no account ID is configured.
    pub async fn get_by_since_id(
        &self,
        req: GetTransactionsBySinceIDRequest,
    ) -> Result<GetTransactionsResponse, APIError> {
        let mut url = self.connection.account_url("transactions/sinceid")?;
        req.set_params(&mut url);
        self.connection.http_client.get_json(url).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn regression_transaction_filter_query_parameters() {
        use std::collections::BTreeMap;
        let mut url = Url::parse("https://example.com/transactions").unwrap();
        ListTransactionsRequest::new()
            .transaction_type(TransactionFilter::Order)
            .transaction_type(TransactionFilter::GuaranteedStopLossOrder)
            .transaction_type(TransactionFilter::ResetResettablePL)
            .set_params(&mut url);
        let params: BTreeMap<_, _> = url.query_pairs().into_owned().collect();
        assert_eq!(params.len(), 1);
        assert_eq!(
            params["type"],
            "ORDER,GUARANTEED_STOP_LOSS_ORDER,RESET_RESETTABLE_PL"
        );

        let mut url = Url::parse("https://example.com/transactions/idrange").unwrap();
        GetTransactionsByIDRangeRequest::new("1".into(), "5".into())
            .filter(TransactionFilter::OrderFill)
            .filter(TransactionFilter::Funding)
            .set_params(&mut url);
        let params: BTreeMap<_, _> = url.query_pairs().into_owned().collect();
        assert_eq!(params.len(), 3);
        assert_eq!(params["from"], "1");
        assert_eq!(params["to"], "5");
        assert_eq!(params["type"], "ORDER_FILL,FUNDING");

        let mut url = Url::parse("https://example.com/transactions/sinceid").unwrap();
        GetTransactionsBySinceIDRequest::new("5".into())
            .filter(TransactionFilter::OrderFill)
            .filter(TransactionFilter::Funding)
            .set_params(&mut url);
        let params: BTreeMap<_, _> = url.query_pairs().into_owned().collect();
        assert_eq!(params.len(), 2);
        assert_eq!(params["id"], "5");
        assert_eq!(params["type"], "ORDER_FILL,FUNDING");
    }

    #[test]
    fn regression_transaction_date_range_query_parameters() {
        use std::collections::BTreeMap;
        let from: DateTime<Utc> = "2026-01-01T00:00:00.123456789Z".parse().unwrap();
        let to: DateTime<Utc> = "2026-01-02T00:00:00Z".parse().unwrap();
        let mut url = Url::parse("https://example.com/transactions").unwrap();
        ListTransactionsRequest::new()
            .from_time(from)
            .to(to)
            .page_size(25)
            .set_params(&mut url);
        let params: BTreeMap<_, _> = url.query_pairs().into_owned().collect();
        assert_eq!(params.len(), 3);
        assert_eq!(params["from"], from.to_rfc3339());
        assert_eq!(params["to"], to.to_rfc3339());
        assert_eq!(params["pageSize"], "25");
    }
}
