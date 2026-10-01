use futures_util::StreamExt;
use oanda_rust::account::ConfigureAccountRequest;
use oanda_rust::client::Client;
use oanda_rust::errors::{APIError, ErrorResponse};
use oanda_rust::instrument::{CandlestickGranularity, CandlesticksRequest, WeeklyAlignment};
use oanda_rust::order::{MarketOrderRequest, OrderRequest, UpdateOrderClientExtensionsRequest};
use oanda_rust::position::ClosePositionRequest;
use oanda_rust::pricing::{AccountCandlesticksRequest, LatestCandlesRequest};
use oanda_rust::stream::{PricingStreamOptions, StreamClient};
use oanda_rust::trade::{CloseTradeRequest, ListTradesRequest, TradeState, TradeStateFilter};
use oanda_rust::transaction::{ClientExtensions, OrderCreateRejectTransaction};
use serde_json::json;
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use tokio::task::JoinHandle;
use url::Url;

async fn fixture(status: &str, body: &str) -> (Url, JoinHandle<String>) {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let url = Url::parse(&format!("http://{}", listener.local_addr().unwrap())).unwrap();
    let response = format!("HTTP/1.1 {status}\r\nContent-Type: application/json\r\nRequestID: fixture-123\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}", body.len());
    let task = tokio::spawn(async move {
        tokio::time::timeout(Duration::from_secs(5), async move {
            let (mut socket, _) = listener.accept().await.unwrap();
            let mut request = Vec::new();
            let mut buffer = [0; 1024];
            loop {
                let read = socket.read(&mut buffer).await.unwrap();
                assert_ne!(read, 0, "client closed before completing request");
                request.extend_from_slice(&buffer[..read]);
                if let Some(end) = request.windows(4).position(|bytes| bytes == b"\r\n\r\n") {
                    let headers = String::from_utf8_lossy(&request[..end]).to_ascii_lowercase();
                    let length = headers
                        .lines()
                        .find_map(|line| line.strip_prefix("content-length: "))
                        .map(|length| length.parse::<usize>().unwrap())
                        .unwrap_or(0);
                    if request.len() >= end + 4 + length {
                        break;
                    }
                }
            }
            socket.write_all(response.as_bytes()).await.unwrap();
            String::from_utf8(request).unwrap()
        })
        .await
        .unwrap()
    });
    (url, task)
}

fn custom_http_client() -> reqwest::Client {
    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert("x-fixture", "injected".parse().unwrap());
    reqwest::Client::builder()
        .no_proxy()
        .timeout(Duration::from_secs(3))
        .default_headers(headers)
        .build()
        .unwrap()
}

fn client(url: Url) -> Client {
    Client::new_practice("fixture-token")
        .unwrap()
        .with_http_client(custom_http_client())
        .with_base_url(url)
        .unwrap()
        .with_account_id("account".into())
}

fn request_url(request: &str) -> Url {
    Url::parse(&format!(
        "http://fixture{}",
        request.split_whitespace().nth(1).unwrap()
    ))
    .unwrap()
}

#[tokio::test]
async fn latest_candles_encodes_all_options_and_decodes_multiple_series() {
    let body = r#"{"latestCandles":[{"instrument":"EUR_USD","granularity":"H1","candles":[{"time":"2026-09-12T00:00:00Z","mid":{"o":"1.1","h":"1.2","l":"1.0","c":"1.15"},"volume":100000,"complete":true}]},{"instrument":"USD_JPY","granularity":"M5","candles":[]}]}"#;
    let (url, request) = fixture("200 OK", body).await;
    let response = client(url.join("gateway/").unwrap())
        .pricing()
        .candles_latest(
            LatestCandlesRequest::new(vec![" EUR_USD:H1:M ".into(), "USD_JPY:M5:BA".into()])
                .units("1000.5".into())
                .smooth(false)
                .daily_alignment(23)
                .alignment_timezone("America/New_York".into())
                .weekly_alignment(WeeklyAlignment::Monday),
        )
        .await
        .unwrap();
    assert_eq!(response.latest_candles.len(), 2);
    assert!(response.latest_candles[0].candles[0].complete);
    assert_eq!(response.latest_candles[0].candles[0].volume, 100000);
    let request = request.await.unwrap();
    assert!(request.starts_with("GET "));
    assert!(request
        .to_ascii_lowercase()
        .contains("authorization: bearer fixture-token\r\n"));
    let url = request_url(&request);
    assert_eq!(url.path(), "/gateway/v3/accounts/account/candles/latest");
    let query: std::collections::HashMap<_, _> = url.query_pairs().collect();
    assert_eq!(query["candleSpecifications"], "EUR_USD:H1:M,USD_JPY:M5:BA");
    assert_eq!(query["units"], "1000.5");
    assert_eq!(query["smooth"], "false");
    assert_eq!(query["dailyAlignment"], "23");
    assert_eq!(query["alignmentTimezone"], "America/New_York");
    assert_eq!(query["weeklyAlignment"], "Monday");

    let (url, request) = fixture("200 OK", r#"{"latestCandles":[]}"#).await;
    client(url)
        .pricing()
        .candles_latest(LatestCandlesRequest::new(vec!["EUR_USD:D:M".into()]))
        .await
        .unwrap();
    assert_eq!(
        request_url(&request.await.unwrap()).query_pairs().count(),
        1
    );
}

#[tokio::test]
async fn account_candles_share_standard_options_and_add_units() {
    let body = r#"{"instrument":"EUR_USD","granularity":"H1","candles":[{"time":"2026-09-12T00:00:00Z","bid":{"o":"1.1","h":"1.2","l":"1.0","c":"1.15"},"ask":{"o":"1.2","h":"1.3","l":"1.1","c":"1.25"},"volume":42,"complete":false}]}"#;
    let (url, request) = fixture("200 OK", body).await;
    let from = chrono::DateTime::parse_from_rfc3339("2026-09-12T00:00:00+09:00")
        .unwrap()
        .with_timezone(&chrono::Local);
    let to = from + chrono::Duration::hours(2);
    let response = client(url.join("gateway/").unwrap())
        .pricing()
        .candlesticks(
            AccountCandlesticksRequest::new(
                CandlesticksRequest::new(" EUR_USD ".into())
                    .bid()
                    .ask()
                    .granularity(CandlestickGranularity::H1)
                    .from(from)
                    .to(to)
                    .smooth(true)
                    .include_first(false)
                    .daily_alignment(17)
                    .alignment_timezone("Asia/Tokyo".into())
                    .weekly_alignment(WeeklyAlignment::Friday),
            )
            .units("500".into()),
        )
        .await
        .unwrap();
    assert!(response.candles[0].bid.is_some());
    assert!(response.candles[0].ask.is_some());
    assert!(!response.candles[0].complete);
    let url = request_url(&request.await.unwrap());
    assert_eq!(
        url.path(),
        "/gateway/v3/accounts/account/instruments/EUR_USD/candles"
    );
    let query: std::collections::HashMap<_, _> = url.query_pairs().collect();
    assert_eq!(query["price"], "BA");
    assert_eq!(query["granularity"], "H1");
    assert_eq!(query["units"], "500");
    assert_eq!(
        chrono::DateTime::parse_from_rfc3339(&query["from"]).unwrap(),
        from
    );
    assert_eq!(
        chrono::DateTime::parse_from_rfc3339(&query["to"]).unwrap(),
        to
    );
    assert_eq!(query["smooth"], "true");
    assert_eq!(query["includeFirst"], "false");
    assert_eq!(query["dailyAlignment"], "17");
    assert_eq!(query["alignmentTimezone"], "Asia/Tokyo");
    assert_eq!(query["weeklyAlignment"], "Friday");

    let (url, request) = fixture("200 OK", body).await;
    client(url)
        .pricing()
        .candlesticks(AccountCandlesticksRequest::new(
            CandlesticksRequest::new("EUR_USD".into()).count(2).unwrap(),
        ))
        .await
        .unwrap();
    assert_eq!(
        request_url(&request.await.unwrap()).query(),
        Some("count=2")
    );
}

#[tokio::test]
async fn pricing_rejects_empty_instrument_lists_before_dispatch() {
    let rest = Client::new_practice("fixture-token")
        .unwrap()
        .with_account_id("account".into());
    assert!(matches!(
        rest.pricing().get(vec![]).await,
        Err(APIError::InvalidRequest(_))
    ));
    assert!(matches!(
        rest.pricing().get(vec!["  ".into()]).await,
        Err(APIError::InvalidRequest(_))
    ));

    let stream = StreamClient::new_practice("fixture-token")
        .unwrap()
        .with_account_id("account".into());
    assert!(matches!(
        stream.pricing(&[], |_| Ok(())).await,
        Err(APIError::InvalidRequest(_))
    ));
    assert!(matches!(
        stream.pricing(&[""], |_| Ok(())).await,
        Err(APIError::InvalidRequest(_))
    ));
}

#[tokio::test]
async fn get_uses_custom_transport_and_preserves_authentication() {
    let (url, request) = fixture("200 OK", r#"{"accounts":[]}"#).await;
    assert!(client(url)
        .account()
        .list()
        .await
        .unwrap()
        .accounts
        .is_empty());
    let request = request.await.unwrap().to_ascii_lowercase();
    assert!(request.starts_with("get /v3/accounts http/1.1"));
    assert!(request.contains("authorization: bearer fixture-token\r\n"));
    assert!(request.contains("accept: application/json\r\n"));
    assert!(request.contains("x-fixture: injected\r\n"));
}

#[tokio::test]
async fn trade_list_defaults_and_id_selection_are_sent() {
    let (url, request) = fixture("200 OK", include_str!("fixtures/trades.json")).await;
    let response = client(url)
        .trade()
        .list(ListTradesRequest::new())
        .await
        .unwrap();
    assert_eq!(response.trades[0].id, "6397");
    assert!(request
        .await
        .unwrap()
        .starts_with("GET /v3/accounts/account/trades HTTP/1.1"));

    let (url, request) = fixture("200 OK", include_str!("fixtures/trades.json")).await;
    client(url)
        .trade()
        .list(
            ListTradesRequest::new()
                .ids("6397".into())
                .ids("6387".into())
                .state(TradeStateFilter::All)
                .count(500),
        )
        .await
        .unwrap();
    assert!(request.await.unwrap().starts_with(
        "GET /v3/accounts/account/trades?ids=6397%2C6387&state=ALL&count=500 HTTP/1.1"
    ));
}

#[tokio::test]
async fn trade_list_can_retrieve_closed_trades_on_later_pages() {
    let mut before_id: Option<String> = None;
    for id in [Some("6397"), Some("6387"), None] {
        let mut body: serde_json::Value =
            serde_json::from_str(include_str!("fixtures/trades.json")).unwrap();
        if let Some(id) = id {
            body["trades"][0]["id"] = json!(id);
            body["trades"][0]["state"] = json!("CLOSED");
            body["trades"][0]["currentUnits"] = json!("0");
        } else {
            body["trades"] = json!([]);
        }
        let (url, request) = fixture("200 OK", &body.to_string()).await;
        let mut query = ListTradesRequest::new()
            .state(TradeStateFilter::Closed)
            .instrument("USD_CAD".into())
            .count(1);
        if let Some(id) = &before_id {
            query = query.before_id(id.clone());
        }
        let page = client(url).trade().list(query).await.unwrap();
        let request = request.await.unwrap();
        let target = request.split_whitespace().nth(1).unwrap();
        let url = Url::parse(&format!("http://fixture{target}")).unwrap();
        let params: std::collections::BTreeMap<_, _> = url.query_pairs().into_owned().collect();
        assert_eq!(params["state"], "CLOSED");
        assert_eq!(params["instrument"], "USD_CAD");
        assert_eq!(params["count"], "1");
        assert_eq!(params.get("beforeID"), before_id.as_ref());
        if let Some(id) = id {
            assert_eq!(page.trades[0].id, id);
            assert!(matches!(page.trades[0].state, TradeState::Closed));
            before_id = Some(page.trades[0].id.clone());
        } else {
            assert!(page.trades.is_empty());
        }
    }
}

#[tokio::test]
async fn trade_list_rejects_invalid_page_sizes_before_dispatch() {
    let client = Client::new_practice("fixture-token")
        .unwrap()
        .with_account_id("account".into());
    for count in [0, 501] {
        assert!(matches!(
            client
                .trade()
                .list(ListTradesRequest::new().count(count))
                .await,
            Err(APIError::InvalidRequest(_))
        ));
    }
}

#[tokio::test]
async fn post_keeps_json_body_and_accepts_created_status() {
    for (wire, units, fill_id) in [
        (
            include_str!("fixtures/order_create_market_buy.json"),
            "100",
            "6368",
        ),
        (
            include_str!("fixtures/order_create_market_sell.json"),
            "-100",
            "647",
        ),
    ] {
        let (url, request) = fixture("201 Created", wire).await;
        let result = client(url)
            .order()
            .create(OrderRequest::Market(MarketOrderRequest::new(
                "EUR_USD".into(),
                units.into(),
            )))
            .await
            .unwrap();
        assert_eq!(result.last_transaction_id, fill_id);
        let fill = result.order_fill_transaction.unwrap();
        assert_eq!(fill.id, fill_id);
        assert_eq!(fill.units, units);
        assert_eq!(fill.trade_opened.unwrap().trade_id, fill_id);
        let request = request.await.unwrap();
        assert!(request.starts_with("POST /v3/accounts/account/orders HTTP/1.1"));
        assert!(request
            .to_ascii_lowercase()
            .contains("authorization: bearer fixture-token\r\n"));
        let body: serde_json::Value =
            serde_json::from_str(request.split("\r\n\r\n").nth(1).unwrap()).unwrap();
        assert_eq!(body["order"]["type"], "MARKET");
        assert_eq!(body["order"]["units"], units);
    }
}

#[tokio::test]
async fn structured_rejections_and_common_fallback_keep_http_context() {
    // A documented reject status decodes to the endpoint's type even when OANDA
    // sends only errorMessage; other statuses fall back to CommonError.
    for (status, code, body, typed) in [
        (
            "404 Not Found",
            404,
            r#"{"errorCode":"ORDER_DOESNT_EXIST","errorMessage":"missing order","relatedTransactionIDs":[],"lastTransactionID":"2"}"#,
            true,
        ),
        (
            "404 Not Found",
            404,
            r#"{"errorMessage":"missing account"}"#,
            true,
        ),
        (
            "401 Unauthorized",
            401,
            r#"{"errorMessage":"Insufficient authorization"}"#,
            false,
        ),
    ] {
        let (url, request) = fixture(status, body).await;
        let error = client(url).order().cancel("1".into()).await.unwrap_err();
        let APIError::Response(context) = error else {
            panic!("missing HTTP context")
        };
        assert_eq!(context.status.as_u16(), code);
        assert_eq!(context.request_id.as_deref(), Some("fixture-123"));
        let APIError::ErrorResponse(error) = context.source else {
            panic!("missing API error")
        };
        if typed {
            assert!(matches!(*error, ErrorResponse::OrderCancelError(_)));
        } else {
            assert!(matches!(*error, ErrorResponse::CommonError(_)));
        }
        assert!(request.await.unwrap().starts_with("PUT "));
    }
}

fn structured_error(error: APIError) -> ErrorResponse {
    let APIError::Response(context) = error else {
        panic!("missing HTTP context")
    };
    let APIError::ErrorResponse(error) = context.source else {
        panic!("expected structured error, got {:?}", context.source)
    };
    *error
}

fn reject_transaction(fields: serde_json::Value) -> serde_json::Value {
    let mut transaction = json!({"id":"3", "time":"2026-09-12T00:00:00Z", "userID":1,
        "accountID":"account", "batchID":"3"});
    transaction
        .as_object_mut()
        .unwrap()
        .extend(fields.as_object().unwrap().clone());
    transaction
}

#[tokio::test]
async fn endpoint_rejections_keep_reject_transactions() {
    let market_reject = reject_transaction(json!({"type":"MARKET_ORDER_REJECT",
        "instrument":"EUR_USD", "units":"-1", "timeInForce":"FOK", "positionFill":"REDUCE_ONLY",
        "reason":"TRADE_CLOSE", "rejectReason":"INSTRUMENT_NOT_TRADEABLE"}));
    let ids = json!({"relatedTransactionIDs":["3"], "lastTransactionID":"3"});
    let body = |fields: serde_json::Value| {
        let mut body = json!({"errorCode":"REJECTED", "errorMessage":"rejected"});
        body.as_object_mut()
            .unwrap()
            .extend(fields.as_object().unwrap().clone());
        body.to_string()
    };

    let mut fields = json!({"orderRejectTransaction": market_reject});
    fields
        .as_object_mut()
        .unwrap()
        .extend(ids.as_object().unwrap().clone());
    let (url, request) = fixture("404 Not Found", &body(fields)).await;
    let order = OrderRequest::Market(MarketOrderRequest::new("EUR_USD".into(), "1".into()));
    let error = structured_error(client(url).order().create(order).await.unwrap_err());
    assert!(matches!(
        error,
        ErrorResponse::OrderCreateError(ref e) if matches!(
            e.order_reject_transaction,
            Some(OrderCreateRejectTransaction::MarketOrderRejectTransaction(_))
        )
    ));
    request.await.unwrap();

    let mut fields = json!({"orderCancelRejectTransaction": reject_transaction(json!({
        "type":"ORDER_CANCEL_REJECT", "orderID":"1", "rejectReason":"ORDER_DOESNT_EXIST"}))});
    fields
        .as_object_mut()
        .unwrap()
        .extend(ids.as_object().unwrap().clone());
    let (url, request) = fixture("404 Not Found", &body(fields)).await;
    let error = structured_error(client(url).order().cancel("1".into()).await.unwrap_err());
    assert!(matches!(
        error,
        ErrorResponse::OrderCancelError(ref e)
            if e.order_cancel_reject_transaction.as_ref().is_some_and(|t| t.order_id == "1")
    ));
    request.await.unwrap();

    let mut fields = json!({"tradeClientExtensionsModifyRejectTransaction": reject_transaction(
        json!({"type":"TRADE_CLIENT_EXTENSIONS_MODIFY_REJECT", "tradeID":"7",
            "rejectReason":"TRADE_DOESNT_EXIST"}))});
    fields
        .as_object_mut()
        .unwrap()
        .extend(ids.as_object().unwrap().clone());
    let (url, request) = fixture("404 Not Found", &body(fields)).await;
    let error = structured_error(
        client(url)
            .trade()
            .update_client_extensions("7".into(), ClientExtensions::default())
            .await
            .unwrap_err(),
    );
    assert!(matches!(
        error,
        ErrorResponse::UpdateTradeClientExtensionsError(ref e)
            if e.trade_client_extensions_modify_reject_transaction
                .as_ref()
                .is_some_and(|t| t.trade_id == "7")
    ));
    request.await.unwrap();

    // The documented 400 body for closing a trade carries no transaction IDs.
    let (url, request) = fixture(
        "400 Bad Request",
        &body(json!({"orderRejectTransaction": market_reject})),
    )
    .await;
    let error = structured_error(
        client(url)
            .trade()
            .close("7".into(), CloseTradeRequest::new())
            .await
            .unwrap_err(),
    );
    assert!(matches!(
        error,
        ErrorResponse::CloseTradeError(ref e)
            if e.order_reject_transaction.is_some() && e.last_transaction_id.is_none()
    ));
    assert_eq!(
        error.to_string(),
        "Trade close was rejected REJECTED: rejected"
    );
    request.await.unwrap();

    let mut fields = json!({"longOrderRejectTransaction": market_reject});
    fields
        .as_object_mut()
        .unwrap()
        .extend(ids.as_object().unwrap().clone());
    let (url, request) = fixture("400 Bad Request", &body(fields)).await;
    let error = structured_error(
        client(url)
            .position()
            .close("EUR_USD".into(), ClosePositionRequest::new())
            .await
            .unwrap_err(),
    );
    assert!(matches!(
        error,
        ErrorResponse::ClosePositionError(ref e)
            if e.long_order_reject_transaction.is_some()
                && e.short_order_reject_transaction.is_none()
    ));
    request.await.unwrap();
}

#[tokio::test]
async fn documented_optional_error_fields_can_be_absent() {
    let market_reject = reject_transaction(json!({
        "type":"MARKET_ORDER_REJECT", "instrument":"EUR_USD", "units":"-1",
        "timeInForce":"FOK", "positionFill":"REDUCE_ONLY", "reason":"TRADE_CLOSE",
        "rejectReason":"INSTRUMENT_NOT_TRADEABLE"
    }));

    let order_body = json!({
        "orderRejectTransaction": market_reject,
        "relatedTransactionIDs": ["3"], "lastTransactionID": "3",
        "errorMessage": "order rejected"
    });
    let (url, request) = fixture("404 Not Found", &order_body.to_string()).await;
    let order = OrderRequest::Market(MarketOrderRequest::new("EUR_USD".into(), "1".into()));
    let error = structured_error(client(url).order().create(order).await.unwrap_err());
    assert!(matches!(
        error,
        ErrorResponse::OrderCreateError(ref e)
            if e.error_code.is_none() && e.order_reject_transaction.is_some()
    ));
    request.await.unwrap();

    let (url, request) = fixture("404 Not Found", r#"{"errorMessage":"account missing"}"#).await;
    let order = OrderRequest::Market(MarketOrderRequest::new("EUR_USD".into(), "1".into()));
    let error = structured_error(client(url).order().create(order).await.unwrap_err());
    assert!(matches!(
        error,
        ErrorResponse::OrderCreateError(ref e)
            if e.order_reject_transaction.is_none()
                && e.related_transaction_ids.is_none()
                && e.last_transaction_id.is_none()
    ));
    request.await.unwrap();

    let trade_body = json!({
        "orderRejectTransaction": market_reject, "errorMessage": "trade rejected"
    });
    let (url, request) = fixture("400 Bad Request", &trade_body.to_string()).await;
    let error = structured_error(
        client(url)
            .trade()
            .close("7".into(), CloseTradeRequest::new())
            .await
            .unwrap_err(),
    );
    assert!(matches!(
        error,
        ErrorResponse::CloseTradeError(ref e)
            if e.error_code.is_none() && e.order_reject_transaction.is_some()
    ));
    request.await.unwrap();

    let position_body = json!({
        "longOrderRejectTransaction": market_reject,
        "relatedTransactionIDs": ["3"], "lastTransactionID": "3",
        "errorMessage": "position rejected"
    });
    let (url, request) = fixture("400 Bad Request", &position_body.to_string()).await;
    let error = structured_error(
        client(url)
            .position()
            .close("EUR_USD".into(), ClosePositionRequest::new())
            .await
            .unwrap_err(),
    );
    assert!(matches!(
        error,
        ErrorResponse::ClosePositionError(ref e)
            if e.error_code.is_none() && e.long_order_reject_transaction.is_some()
    ));
    request.await.unwrap();

    let (url, request) = fixture("404 Not Found", r#"{"errorMessage":"account missing"}"#).await;
    let error = structured_error(
        client(url)
            .position()
            .close("EUR_USD".into(), ClosePositionRequest::new())
            .await
            .unwrap_err(),
    );
    assert!(matches!(
        error,
        ErrorResponse::ClosePositionError(ref e)
            if e.related_transaction_ids.is_none()
                && e.last_transaction_id.is_none()
                && e.error_code.is_none()
    ));
    request.await.unwrap();

    // The remaining reject types, with only the required errorMessage.
    let minimal = r#"{"errorMessage":"rejected"}"#;
    let (url, request) = fixture("404 Not Found", minimal).await;
    let error = structured_error(
        client(url)
            .order()
            .update_client_extensions("1".into(), UpdateOrderClientExtensionsRequest::new())
            .await
            .unwrap_err(),
    );
    assert!(matches!(
        error,
        ErrorResponse::UpdateOrderClientExtensionsError(ref e)
            if e.order_client_extensions_modify_reject_transaction.is_none()
    ));
    request.await.unwrap();

    let (url, request) = fixture("400 Bad Request", minimal).await;
    let error = structured_error(
        client(url)
            .trade()
            .update_client_extensions("7".into(), ClientExtensions::default())
            .await
            .unwrap_err(),
    );
    assert!(matches!(
        error,
        ErrorResponse::UpdateTradeClientExtensionsError(ref e)
            if e.trade_client_extensions_modify_reject_transaction.is_none()
    ));
    request.await.unwrap();

    let (url, request) = fixture("403 Forbidden", minimal).await;
    let error = structured_error(
        client(url)
            .account()
            .configure(&"account".into(), ConfigureAccountRequest::new())
            .await
            .unwrap_err(),
    );
    assert!(matches!(
        error,
        ErrorResponse::ConfigureAccountError(ref e)
            if e.client_configure_reject_transaction.is_none()
    ));
    // Without an errorCode the message names no code.
    assert_eq!(error.to_string(), "Configure account error: rejected");
    request.await.unwrap();
}

#[tokio::test]
async fn invalid_success_and_error_bodies_keep_status_and_request_id() {
    for (status, code, body) in [
        ("200 OK", 200, "{invalid"),
        ("502 Bad Gateway", 502, "<html>gateway unavailable</html>"),
    ] {
        let (url, request) = fixture(status, body).await;
        let error = client(url).account().list().await.unwrap_err();
        let APIError::Response(context) = error else {
            panic!("missing HTTP context")
        };
        assert_eq!(context.status.as_u16(), code);
        assert_eq!(context.request_id.as_deref(), Some("fixture-123"));
        assert!(matches!(context.source, APIError::JSONError(_)));
        request.await.unwrap();
    }
}

#[tokio::test]
async fn streams_accept_captured_handlers_and_custom_transport() {
    let body =
        "{\"type\":\"HEARTBEAT\",\"time\":\"2026-09-12T00:00:00Z\",\"lastTransactionID\":\"1\"}\n";
    for pricing in [false, true] {
        let (url, request) = fixture("200 OK", body).await;
        let client = StreamClient::new_practice("fixture-token")
            .unwrap()
            .with_http_client(custom_http_client())
            .with_base_url(url)
            .unwrap()
            .with_account_id("account".into());
        let mut count = 0;
        if pricing {
            client
                .pricing(&["EUR_USD", "USD_JPY"], |_| {
                    count += 1;
                    Ok(())
                })
                .await
                .unwrap();
        } else {
            client
                .transactions(|_| {
                    count += 1;
                    Ok(())
                })
                .await
                .unwrap();
        }
        assert_eq!(count, 1);
        let request = request.await.unwrap().to_ascii_lowercase();
        assert!(request.contains("authorization: bearer fixture-token\r\n"));
        assert!(request.contains("accept: application/octet-stream\r\n"));
        assert!(request.contains("x-fixture: injected\r\n"));
        if pricing {
            assert!(request.starts_with(
                "get /v3/accounts/account/pricing/stream?instruments=eur_usd%2cusd_jpy "
            ));
        } else {
            assert!(request.starts_with("get /v3/accounts/account/transactions/stream "));
        }
    }
}

#[tokio::test]
async fn pull_streams_decode_messages_and_send_pricing_options() {
    let body = concat!(
        "{\"type\":\"PRICE\",\"instrument\":\"USD_JPY\",\"bids\":[],\"asks\":[],\"closeoutBid\":\"150.00\",\"closeoutAsk\":\"150.02\",\"time\":\"2026-09-12T00:00:00Z\"}\n",
        "{\"homeConversions\":[{\"currency\":\"USD\",\"accountGain\":\"150.1\",\"accountLoss\":\"150.2\",\"positionValue\":\"150.15\"}]}\n",
        "{\"type\":\"HEARTBEAT\",\"time\":\"2026-09-12T00:00:05Z\"}\n",
    );
    let (url, request) = fixture("200 OK", body).await;
    let client = StreamClient::new_practice("fixture-token")
        .unwrap()
        .with_http_client(custom_http_client())
        .with_base_url(url)
        .unwrap()
        .with_account_id("account".into());
    let options = PricingStreamOptions {
        snapshot: false,
        include_home_conversions: true,
    };
    let stream = client
        .pricing_stream_with_options(&["USD_JPY"], options)
        .await
        .unwrap();
    futures_util::pin_mut!(stream);
    assert!(matches!(
        stream.next().await.unwrap().unwrap(),
        oanda_rust::pricing::PricingStreamItem::Price(_)
    ));
    assert!(matches!(
        stream.next().await.unwrap().unwrap(),
        oanda_rust::pricing::PricingStreamItem::HomeConversions(_)
    ));
    assert!(matches!(
        stream.next().await.unwrap().unwrap(),
        oanda_rust::pricing::PricingStreamItem::Heartbeat(_)
    ));
    assert!(stream.next().await.is_none());
    let request = request.await.unwrap().to_ascii_lowercase();
    assert!(request.contains("snapshot=false"));
    assert!(request.contains("includehomeconversions=true"));

    let body =
        "{\"type\":\"HEARTBEAT\",\"time\":\"2026-09-12T00:00:00Z\",\"lastTransactionID\":\"1\"}\n";
    let (url, request) = fixture("200 OK", body).await;
    let client = StreamClient::new_practice("fixture-token")
        .unwrap()
        .with_http_client(custom_http_client())
        .with_base_url(url)
        .unwrap()
        .with_account_id("account".into());
    let stream = client.transactions_stream().await.unwrap();
    futures_util::pin_mut!(stream);
    assert!(matches!(
        stream.next().await.unwrap().unwrap(),
        oanda_rust::transaction::TransactionStreamItem::Heartbeat(_)
    ));
    assert!(stream.next().await.is_none());
    request.await.unwrap();
}

#[tokio::test]
async fn base_url_path_prefix_is_kept() {
    for prefix in ["gateway/oanda", "gateway/oanda/"] {
        let (url, request) = fixture("200 OK", r#"{"accounts":[]}"#).await;
        client(url.join(prefix).unwrap())
            .account()
            .list()
            .await
            .unwrap();
        assert!(request
            .await
            .unwrap()
            .starts_with("GET /gateway/oanda/v3/accounts HTTP/1.1"));
    }

    let (url, request) = fixture("200 OK", r#"{"positions":[],"lastTransactionID":"1"}"#).await;
    client(url.join("gateway/").unwrap())
        .position()
        .list()
        .await
        .unwrap();
    assert!(request
        .await
        .unwrap()
        .starts_with("GET /gateway/v3/accounts/account/positions HTTP/1.1"));

    let (url, request) = fixture(
        "200 OK",
        r#"{"instrument":"EUR_USD","granularity":"D","candles":[]}"#,
    )
    .await;
    client(url.join("gateway/").unwrap())
        .instrument()
        .candlesticks(CandlesticksRequest::new("EUR_USD".into()))
        .await
        .unwrap();
    assert!(request
        .await
        .unwrap()
        .starts_with("GET /gateway/v3/instruments/EUR_USD/candles"));

    let body =
        "{\"type\":\"HEARTBEAT\",\"time\":\"2026-09-12T00:00:00Z\",\"lastTransactionID\":\"1\"}\n";
    let (url, request) = fixture("200 OK", body).await;
    StreamClient::new_practice("fixture-token")
        .unwrap()
        .with_http_client(custom_http_client())
        .with_base_url(url.join("gateway/").unwrap())
        .unwrap()
        .with_account_id("account".into())
        .transactions(|_| Ok(()))
        .await
        .unwrap();
    assert!(request
        .await
        .unwrap()
        .starts_with("GET /gateway/v3/accounts/account/transactions/stream HTTP/1.1"));
}

#[tokio::test]
async fn stream_http_error_retains_context() {
    let (url, request) = fixture("401 Unauthorized", r#"{"errorMessage":"invalid token"}"#).await;
    let client = StreamClient::new_practice("fixture-token")
        .unwrap()
        .with_http_client(custom_http_client())
        .with_base_url(url)
        .unwrap()
        .with_account_id("account".into());
    let error = client
        .transactions(|_| panic!("must not dispatch error response"))
        .await
        .unwrap_err();
    let APIError::Response(context) = error else {
        panic!("missing HTTP context")
    };
    assert_eq!(context.status, reqwest::StatusCode::UNAUTHORIZED);
    assert_eq!(context.request_id.as_deref(), Some("fixture-123"));
    request.await.unwrap();
}

#[tokio::test]
async fn missing_account_is_an_error_before_network_dispatch() {
    let client = Client::new_practice("fixture-token").unwrap();
    assert!(matches!(
        client.position().list().await,
        Err(APIError::InvalidRequest(_))
    ));
    assert!(matches!(
        client.account().get_details(&String::new()).await,
        Err(APIError::InvalidRequest(_))
    ));
    let stream = StreamClient::new_practice("fixture-token").unwrap();
    assert!(matches!(
        stream.transactions(|_| Ok(())).await,
        Err(APIError::InvalidRequest(_))
    ));
    assert!(matches!(
        stream.pricing(&["EUR_USD"], |_| Ok(())).await,
        Err(APIError::InvalidRequest(_))
    ));
}

#[test]
fn invalid_configuration_returns_errors_without_exposing_token() {
    for token in ["", " ", "secret\r\ninvalid", "secret token", "secret\tkey"] {
        let error = Client::new(token).err().expect("must reject token");
        assert!(matches!(error, APIError::InvalidRequest(_)));
        assert!(!error.to_string().contains("secret"));
        assert!(matches!(
            StreamClient::new(token),
            Err(APIError::InvalidRequest(_))
        ));
    }
    let client = Client::new_practice("fixture-token").unwrap();
    assert!(matches!(
        client.with_base_url(Url::parse("file:///tmp/api").unwrap()),
        Err(APIError::InvalidRequest(_))
    ));
}
