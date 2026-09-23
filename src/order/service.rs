use super::*;

/// Provides access to the OANDA Order endpoints (`/v3/accounts/{id}/orders/...`).
///
/// Obtain an instance via [`Client::order`](crate::client::Client::order).
pub struct OrderService<'a> {
    connection: &'a Connection,
}

impl<'a> OrderService<'a> {
    /// Creates a new `OrderService` bound to the given client.
    pub(crate) fn new(connection: &'a Connection) -> Self {
        OrderService { connection }
    }

    /// Creates a new order on the account.
    ///
    /// Calls `POST /v3/accounts/{accountID}/orders`. On success (HTTP 201)
    /// returns [`CreateOrderResponse`].
    ///
    /// Returns [`APIError::InvalidRequest`] if no account ID is configured.
    pub async fn create(&self, order: OrderRequest) -> Result<CreateOrderResponse, APIError> {
        let url = self.connection.account_url(&["orders"])?;
        let body = CreateOrderRequest { order };
        let http_resp = self
            .connection
            .http_client
            .post(url)
            .json(&body)
            .send()
            .await?;
        decode_response::<CreateOrderResponse>(
            http_resp,
            StatusCode::CREATED,
            Some(|status, body| match status {
                StatusCode::BAD_REQUEST | StatusCode::NOT_FOUND => {
                    decode_reject(body, ErrorResponse::OrderCreateError)
                }
                _ => None,
            }),
        )
        .await
    }

    /// Lists orders on the account, optionally filtered by the parameters in `req`.
    ///
    /// Calls `GET /v3/accounts/{accountID}/orders`.
    ///
    /// Returns [`APIError::InvalidRequest`] if no account ID is configured.
    pub async fn list(&self, req: ListOrdersRequest) -> Result<ListOrdersResponse, APIError> {
        let mut url = self.connection.account_url(&["orders"])?;
        req.set_params(&mut url);
        self.connection.http_client.get_json(url).await
    }

    /// Returns all pending (not yet filled or cancelled) orders on the account.
    ///
    /// Calls `GET /v3/accounts/{accountID}/pendingOrders`.
    ///
    /// Returns [`APIError::InvalidRequest`] if no account ID is configured.
    pub async fn list_pending(&self) -> Result<ListOrdersResponse, APIError> {
        let url = self.connection.account_url(&["pendingOrders"])?;
        self.connection.http_client.get_json(url).await
    }

    /// Returns the details of a single order identified by `specifier`.
    ///
    /// Calls `GET /v3/accounts/{accountID}/orders/{orderSpecifier}`.
    ///
    /// Returns [`APIError::InvalidRequest`] if no account ID is configured.
    pub async fn get_details(
        &self,
        specifier: OrderSpecifier,
    ) -> Result<GetOrderDetailsResponse, APIError> {
        let url = self.connection.account_url(&["orders", &specifier])?;
        self.connection.http_client.get_json(url).await
    }

    /// Replaces the order identified by `specifier` with a new `order`.
    ///
    /// Calls `PUT /v3/accounts/{accountID}/orders/{orderSpecifier}`.
    /// The original order is cancelled and a new one is created atomically.
    ///
    /// Returns [`APIError::InvalidRequest`] if no account ID is configured.
    pub async fn replace(
        &self,
        specifier: OrderSpecifier,
        req: OrderRequest,
    ) -> Result<ReplaceOrderResponse, APIError> {
        let url = self.connection.account_url(&["orders", &specifier])?;
        let body = CreateOrderRequest { order: req };
        let http_resp = self
            .connection
            .http_client
            .put(url)
            .json(&body)
            .send()
            .await?;
        decode_response::<ReplaceOrderResponse>(
            http_resp,
            StatusCode::CREATED,
            Some(|status, body| match status {
                StatusCode::BAD_REQUEST => decode_reject(body, ErrorResponse::OrderCreateError),
                StatusCode::NOT_FOUND => decode_reject(body, ErrorResponse::OrderCancelError),
                _ => None,
            }),
        )
        .await
    }

    /// Cancels the pending order identified by `specifier`.
    ///
    /// Calls `PUT /v3/accounts/{accountID}/orders/{orderSpecifier}/cancel`.
    ///
    /// Returns [`APIError::InvalidRequest`] if no account ID is configured.
    pub async fn cancel(&self, specifier: OrderSpecifier) -> Result<CancelOrderResponse, APIError> {
        let url = self
            .connection
            .account_url(&["orders", &specifier, "cancel"])?;
        let http_resp = self.connection.http_client.put(url).send().await?;
        decode_response::<CancelOrderResponse>(
            http_resp,
            StatusCode::OK,
            Some(|status, body| match status {
                StatusCode::NOT_FOUND => decode_reject(body, ErrorResponse::OrderCancelError),
                _ => None,
            }),
        )
        .await
    }

    /// Updates the client extensions on the order and/or its associated trade.
    ///
    /// Calls `PUT /v3/accounts/{accountID}/orders/{orderSpecifier}/clientExtensions`.
    ///
    /// Returns [`APIError::InvalidRequest`] if no account ID is configured.
    pub async fn update_client_extensions(
        &self,
        specifier: OrderSpecifier,
        req: UpdateOrderClientExtensionsRequest,
    ) -> Result<UpdateOrderClientExtensionsResponse, APIError> {
        let url = self
            .connection
            .account_url(&["orders", &specifier, "clientExtensions"])?;
        let http_resp = self
            .connection
            .http_client
            .put(url)
            .json(&req)
            .send()
            .await?;
        decode_response::<UpdateOrderClientExtensionsResponse>(
            http_resp,
            StatusCode::OK,
            Some(|status, body| match status {
                StatusCode::BAD_REQUEST | StatusCode::NOT_FOUND => {
                    decode_reject(body, ErrorResponse::UpdateOrderClientExtensionsError)
                }
                _ => None,
            }),
        )
        .await
    }
}
