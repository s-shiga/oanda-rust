use super::*;

// ---------------------------------------------------------------------------
// Request/Response types
// ---------------------------------------------------------------------------

/// Request parameters for `GET /v3/accounts/{accountID}/transactions`.
///
/// All fields are optional. Call builder methods to set filters before passing
/// to [`TransactionService::list`].
#[derive(Default)]
pub struct ListTransactionsRequest {
    /// Return only transactions at or after this timestamp.
    pub from: Option<DateTime<Utc>>,
    /// Return only transactions at or before this timestamp.
    pub to: Option<DateTime<Utc>>,
    /// Maximum number of transactions per page (server default applies when `None`).
    pub page_size: Option<u16>,
    /// Restrict results to these transaction filters. Empty = all types.
    pub transaction_type: Vec<TransactionFilter>,
}

impl ListTransactionsRequest {
    /// Creates a new request with no filters applied.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the earliest timestamp for transactions to include.
    pub fn from_time(mut self, from: DateTime<Utc>) -> Self {
        self.from = Some(from);
        self
    }

    pub fn to(mut self, to: DateTime<Utc>) -> Self {
        self.to = Some(to);
        self
    }

    pub fn page_size(mut self, page_size: u16) -> Self {
        self.page_size = Some(page_size);
        self
    }

    pub fn transaction_type(mut self, transaction_type: TransactionFilter) -> Self {
        self.transaction_type.push(transaction_type);
        self
    }

    pub(crate) fn set_params(&self, url: &mut Url) {
        if let Some(from) = self.from {
            url.query_pairs_mut()
                .append_pair("from", &from.to_rfc3339());
        }
        if let Some(to) = self.to {
            url.query_pairs_mut().append_pair("to", &to.to_rfc3339());
        }
        if let Some(page_size) = self.page_size {
            url.query_pairs_mut()
                .append_pair("pageSize", page_size.to_string().as_str());
        }
        if !self.transaction_type.is_empty() {
            url.query_pairs_mut().append_pair(
                "type",
                self.transaction_type
                    .iter()
                    .map(|t| t.to_string())
                    .collect::<Vec<String>>()
                    .join(",")
                    .as_str(),
            );
        }
    }
}

/// Request parameters for
/// `GET /v3/accounts/{accountID}/transactions/idrange`.
///
/// Returns transactions with IDs in the inclusive range `[from, to]`.
pub struct GetTransactionsByIDRangeRequest {
    from: TransactionID,
    to: TransactionID,
    filter: Vec<TransactionFilter>,
}

impl GetTransactionsByIDRangeRequest {
    /// Creates a new request for the inclusive ID range `[from, to]`.
    pub fn new(from: TransactionID, to: TransactionID) -> Self {
        GetTransactionsByIDRangeRequest {
            from,
            to,
            filter: Vec::new(),
        }
    }

    /// Restricts the results to transactions matching `filter`.
    /// Call multiple times to include several filter categories.
    pub fn filter(mut self, filter: TransactionFilter) -> Self {
        self.filter.push(filter);
        self
    }

    pub(crate) fn set_params(&self, url: &mut Url) {
        url.query_pairs_mut()
            .append_pair("from", &self.from.to_string())
            .append_pair("to", &self.to.to_string());
        if !self.filter.is_empty() {
            url.query_pairs_mut().append_pair(
                "type",
                self.filter
                    .iter()
                    .map(|f| f.to_string())
                    .collect::<Vec<String>>()
                    .join(",")
                    .as_str(),
            );
        }
    }
}

/// Request parameters for
/// `GET /v3/accounts/{accountID}/transactions/sinceid`.
///
/// Returns all transactions with IDs greater than `id`.
pub struct GetTransactionsBySinceIDRequest {
    id: TransactionID,
    filter: Vec<TransactionFilter>,
}

impl GetTransactionsBySinceIDRequest {
    /// Creates a new request that returns transactions since (exclusive of) `id`.
    pub fn new(id: TransactionID) -> Self {
        GetTransactionsBySinceIDRequest {
            id,
            filter: Vec::new(),
        }
    }

    /// Restricts the results to transactions matching `filter`.
    /// Call multiple times to include several filter categories.
    pub fn filter(mut self, filter: TransactionFilter) -> Self {
        self.filter.push(filter);
        self
    }

    pub(crate) fn set_params(&self, url: &mut Url) {
        url.query_pairs_mut()
            .append_pair("id", &self.id.to_string());
        if !self.filter.is_empty() {
            url.query_pairs_mut().append_pair(
                "type",
                &self
                    .filter
                    .iter()
                    .map(|f| f.to_string())
                    .collect::<Vec<String>>()
                    .join(","),
            );
        }
    }
}
