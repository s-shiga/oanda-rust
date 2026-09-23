use super::*;

/// Response body for a successful `POST /v3/accounts/{accountID}/orders` (HTTP 201).
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateOrderResponse {
    /// The transaction that recorded the order creation.
    pub order_create_transaction: Option<OrderCreateTransaction>,
    /// The transaction that filled the order, if it was immediately filled
    /// (e.g. a market order).
    pub order_fill_transaction: Option<Box<OrderFillTransaction>>,
    /// The transaction that cancelled the order, if it was immediately cancelled
    /// (e.g. a FOK order that could not be filled).
    pub order_cancel_transaction: Option<OrderCancelTransaction>,
    /// The transaction that re-issued the order (e.g. an IOC order partially filled).
    pub order_reissue_transaction: Option<OrderCreateTransaction>,
    /// The transaction that rejected the re-issued order, if applicable.
    pub order_reissue_reject_transaction: Option<OrderCreateRejectTransaction>,
    /// IDs of all transactions related to this request.
    #[serde(rename = "relatedTransactionIDs")]
    pub related_transaction_ids: Vec<TransactionID>,
    /// ID of the most recent transaction on the account after this request.
    #[serde(rename = "lastTransactionID")]
    pub last_transaction_id: TransactionID,
}

/// Response body for a successful `PUT /v3/accounts/{accountID}/orders/{orderSpecifier}`
/// (HTTP 201).
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReplaceOrderResponse {
    /// The transaction that cancelled the replaced order.
    pub order_cancel_transaction: Option<OrderCancelTransaction>,
    /// The transaction that created the replacement order.
    pub order_create_transaction: Option<OrderCreateTransaction>,
    /// The transaction that filled the replacement order, if immediately filled.
    pub order_fill_transaction: Option<Box<OrderFillTransaction>>,
    /// The transaction that re-issued the order, if applicable.
    pub order_reissue_transaction: Option<OrderCreateTransaction>,
    /// The transaction that rejected the re-issue, if applicable.
    pub order_reissue_reject_transaction: Option<OrderCreateRejectTransaction>,
    /// The transaction that cancelled the replacement order, present only when
    /// the replacement was cancelled immediately.
    pub replacing_order_cancel_transaction: Option<OrderCancelTransaction>,
    /// IDs of all transactions related to this request.
    #[serde(rename = "relatedTransactionIDs")]
    pub related_transaction_ids: Option<Vec<TransactionID>>,
    /// ID of the most recent transaction on the account after this request.
    #[serde(rename = "lastTransactionID")]
    pub last_transaction_id: Option<TransactionID>,
}

/// Error response body for `POST /v3/accounts/{accountID}/orders` (HTTP 400 or
/// 404) and for a rejected order replacement (HTTP 400).
///
/// Returned when an order creation request is rejected by OANDA.
#[derive(Debug, Error, Serialize, Deserialize)]
#[error(
    "Order creation was rejected{}: {error_message}",
    crate::errors::code_suffix(.error_code.as_deref())
)]
#[serde(rename_all = "camelCase")]
pub struct OrderCreateErrorResponse {
    /// The transaction that recorded the rejection reason. `None` when OANDA
    /// omits it.
    pub order_reject_transaction: Option<OrderCreateRejectTransaction>,
    /// IDs of all transactions related to this request. `None` when OANDA
    /// omits them.
    #[serde(rename = "relatedTransactionIDs")]
    pub related_transaction_ids: Option<Vec<TransactionID>>,
    /// ID of the most recent transaction on the account. `None` when OANDA
    /// omits it.
    #[serde(rename = "lastTransactionID")]
    pub last_transaction_id: Option<TransactionID>,
    /// Machine-readable error code. `None` when OANDA omits it.
    pub error_code: Option<String>,
    /// Human-readable description of why the order was rejected.
    pub error_message: String,
}

/// Error response body for order cancellation and replacement requests (HTTP 404).
///
/// Returned when the target order cannot be found or the cancellation is rejected
/// (e.g. the order was already filled).
#[derive(Debug, Error, Serialize, Deserialize)]
#[error(
    "Order cancellation was rejected{}: {error_message}",
    crate::errors::code_suffix(.error_code.as_deref())
)]
#[serde(rename_all = "camelCase")]
pub struct OrderCancelErrorResponse {
    /// The transaction that recorded the rejection reason, if one was created.
    pub order_cancel_reject_transaction: Option<OrderCancelRejectTransaction>,
    /// IDs of all transactions related to this request. `None` when OANDA
    /// omits them.
    #[serde(rename = "relatedTransactionIDs")]
    pub related_transaction_ids: Option<Vec<TransactionID>>,
    /// ID of the most recent transaction on the account. `None` when OANDA
    /// omits it.
    #[serde(rename = "lastTransactionID")]
    pub last_transaction_id: Option<TransactionID>,
    /// Machine-readable error code. `None` when OANDA omits it.
    pub error_code: Option<String>,
    /// Human-readable description of why the cancellation was rejected.
    pub error_message: String,
}

/// Response body for a successful
/// `PUT /v3/accounts/{accountID}/orders/{orderSpecifier}/cancel` (HTTP 200).
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CancelOrderResponse {
    /// The transaction that cancelled the order.
    pub order_cancel_transaction: Option<OrderCancelTransaction>,
    /// IDs of all transactions related to this request.
    #[serde(rename = "relatedTransactionIDs")]
    pub related_transaction_ids: Option<Vec<TransactionID>>,
    /// ID of the most recent transaction on the account after this request.
    #[serde(rename = "lastTransactionID")]
    pub last_transaction_id: Option<TransactionID>,
}

/// Response body for a successful client-extensions update (HTTP 200).
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateOrderClientExtensionsResponse {
    /// The transaction recording the modification.
    pub order_client_extensions_modify_transaction: OrderClientExtensionsModifyTransaction,
    /// IDs of all transactions related to this request.
    #[serde(rename = "relatedTransactionIDs")]
    pub related_transaction_ids: Vec<TransactionID>,
    /// ID of the most recent transaction on the account after this request.
    #[serde(rename = "lastTransactionID")]
    pub last_transaction_id: TransactionID,
}

/// Error response body for a failed order client-extensions update (HTTP 400 or 404).
///
/// Returned when the modification is rejected — for example, if the order
/// specifier does not match any order on the account.
#[derive(Debug, Serialize, Deserialize, Error)]
#[error(
    "Order client extensions update error{}: {error_message}",
    crate::errors::code_suffix(.error_code.as_deref())
)]
#[serde(rename_all = "camelCase")]
pub struct UpdateOrderClientExtensionsErrorResponse {
    /// The reject transaction that recorded why the modification was refused.
    /// `None` when OANDA omits it.
    pub order_client_extensions_modify_reject_transaction:
        Option<OrderClientExtensionsModifyRejectTransaction>,
    /// ID of the most recent transaction on the account. `None` when OANDA
    /// omits it.
    #[serde(rename = "lastTransactionID")]
    pub last_transaction_id: Option<TransactionID>,
    /// IDs of all transactions related to this (failed) request. `None` when
    /// OANDA omits them.
    #[serde(rename = "relatedTransactionIDs")]
    pub related_transaction_ids: Option<Vec<TransactionID>>,
    /// Machine-readable error code. `None` when OANDA omits it.
    pub error_code: Option<String>,
    /// Human-readable description of the error.
    pub error_message: String,
}

/// Response body for `GET /v3/accounts/{accountID}/orders` and
/// `GET /v3/accounts/{accountID}/pendingOrders`.
#[derive(Debug, Serialize, Deserialize)]
pub struct ListOrdersResponse {
    /// The list of orders matching the request filters.
    pub orders: Vec<Order>,
    /// ID of the most recent transaction on the account.
    #[serde(rename = "lastTransactionID")]
    pub last_transaction_id: TransactionID,
}

/// Response body for `GET /v3/accounts/{accountID}/orders/{orderSpecifier}`.
#[derive(Debug, Serialize, Deserialize)]
pub struct GetOrderDetailsResponse {
    /// The requested order.
    pub order: Order,
    /// ID of the most recent transaction on the account.
    #[serde(rename = "lastTransactionID")]
    pub last_transaction_id: TransactionID,
}
