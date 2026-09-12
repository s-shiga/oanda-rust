use oanda_rust::client::Client;
use oanda_rust::errors::{APIError, ErrorResponse};
use oanda_rust::order::{MarketOrderRequest, OrderRequest};
use oanda_rust::stream::StreamClient;
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
async fn post_keeps_json_body_and_accepts_created_status() {
    let (url, request) = fixture(
        "201 Created",
        r#"{"relatedTransactionIDs":[],"lastTransactionID":"1"}"#,
    )
    .await;
    let result = client(url)
        .order()
        .create(OrderRequest::Market(MarketOrderRequest::new(
            "EUR_USD".into(),
            "1".into(),
        )))
        .await
        .unwrap();
    assert_eq!(result.last_transaction_id, "1");
    let request = request.await.unwrap();
    assert!(request.starts_with("POST /v3/accounts/account/orders HTTP/1.1"));
    assert!(request
        .to_ascii_lowercase()
        .contains("authorization: bearer fixture-token\r\n"));
    let body: serde_json::Value =
        serde_json::from_str(request.split("\r\n\r\n").nth(1).unwrap()).unwrap();
    assert_eq!(body["order"]["type"], "MARKET");
}

#[tokio::test]
async fn structured_rejections_and_common_fallback_keep_http_context() {
    for (body, typed) in [
        (
            r#"{"errorCode":"ORDER_DOESNT_EXIST","errorMessage":"missing order","relatedTransactionIDs":[],"lastTransactionID":"2"}"#,
            true,
        ),
        (r#"{"errorMessage":"missing account"}"#, false),
    ] {
        let (url, request) = fixture("404 Not Found", body).await;
        let error = client(url).order().cancel("1".into()).await.unwrap_err();
        let APIError::Response(context) = error else {
            panic!("missing HTTP context")
        };
        assert_eq!(context.status, reqwest::StatusCode::NOT_FOUND);
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
