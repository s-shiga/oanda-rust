//! Shared authenticated transport and response decoding.
use crate::account::AccountID;
use crate::errors::{APIError, CommonErrorResponse, ErrorResponse, HttpResponseError};
use reqwest::header::{HeaderMap, HeaderValue, ACCEPT, AUTHORIZATION};
use reqwest::{RequestBuilder, Response, StatusCode};
use serde::de::DeserializeOwned;
use url::Url;

pub(crate) struct HttpClient {
    client: reqwest::Client,
    headers: HeaderMap,
}

impl HttpClient {
    pub(crate) fn new(api_key: &str, accept: &'static str) -> Result<Self, APIError> {
        if api_key.is_empty()
            || api_key
                .bytes()
                .any(|byte| byte.is_ascii_whitespace() || byte.is_ascii_control())
        {
            return Err(APIError::InvalidRequest(
                "API token must be nonempty and contain no whitespace or control characters".into(),
            ));
        }
        let mut authorization = HeaderValue::from_str(&format!("Bearer {api_key}"))
            .map_err(|_| APIError::InvalidRequest("Invalid API token header".into()))?;
        authorization.set_sensitive(true);
        let mut headers = HeaderMap::new();
        headers.insert(AUTHORIZATION, authorization);
        headers.insert(ACCEPT, HeaderValue::from_static(accept));
        Ok(Self {
            client: reqwest::Client::builder().build()?,
            headers,
        })
    }

    pub(crate) fn get(&self, url: Url) -> RequestBuilder {
        self.client.get(url).headers(self.headers.clone())
    }

    /// Sends a GET request and decodes a `200 OK` JSON response.
    pub(crate) async fn get_json<T: DeserializeOwned>(&self, url: Url) -> Result<T, APIError> {
        decode_response(self.get(url).send().await?, StatusCode::OK, None).await
    }

    pub(crate) fn post(&self, url: Url) -> RequestBuilder {
        self.client.post(url).headers(self.headers.clone())
    }

    pub(crate) fn put(&self, url: Url) -> RequestBuilder {
        self.client.put(url).headers(self.headers.clone())
    }

    pub(crate) fn patch(&self, url: Url) -> RequestBuilder {
        self.client.patch(url).headers(self.headers.clone())
    }
}

/// The authenticated transport, endpoint, and default account shared by
/// [`Client`](crate::client::Client) and [`StreamClient`](crate::stream::StreamClient).
pub(crate) struct Connection {
    pub(crate) http_client: HttpClient,
    pub(crate) base_url: Url,
    pub(crate) account_id: Option<AccountID>,
}

impl Connection {
    /// `base_url` is one of the crate's OANDA host constants.
    pub(crate) fn new(
        api_key: &str,
        accept: &'static str,
        base_url: &str,
    ) -> Result<Self, APIError> {
        Ok(Self {
            http_client: HttpClient::new(api_key, accept)?,
            base_url: Url::parse(base_url).expect("OANDA host constants are valid URLs"),
            account_id: None,
        })
    }

    /// Replaces the transport while keeping the authentication headers.
    pub(crate) fn set_http_client(&mut self, client: reqwest::Client) {
        self.http_client.client = client;
    }

    pub(crate) fn set_base_url(&mut self, url: Url) -> Result<(), APIError> {
        validate_base_url(&url)?;
        self.base_url = url;
        Ok(())
    }

    /// Builds a URL rooted at `/v3/accounts/{accountID}` for the default
    /// account, appending each suffix element as a distinct path segment.
    ///
    /// Returns [`APIError::InvalidRequest`] if no account ID is configured.
    pub(crate) fn account_url(&self, suffix: &[&str]) -> Result<Url, APIError> {
        account_url(&self.base_url, self.account_id.as_ref(), suffix)
    }
}

pub(crate) fn account_url(
    base: &Url,
    account: Option<&AccountID>,
    suffix: &[&str],
) -> Result<Url, APIError> {
    let id = account
        .map(|id| id.trim())
        .filter(|id| !id.is_empty())
        .ok_or_else(|| APIError::InvalidRequest("Missing account_id".into()))?;
    let mut segments = vec!["v3", "accounts", id];
    segments.extend_from_slice(suffix);
    api_url(base, &segments)
}

/// Appends path segments to `base`, keeping any path prefix already on `base`
/// (for example a gateway mounted at `https://host/oanda/`).
pub(crate) fn api_url(base: &Url, segments: &[&str]) -> Result<Url, APIError> {
    // `PathSegmentsMut::extend` silently skips "." and "..", which would drop
    // an ID with either value from the path; an empty ID would leave `//`.
    if let Some(segment) = segments.iter().find(|s| matches!(**s, "" | "." | "..")) {
        return Err(APIError::InvalidRequest(format!(
            "Invalid URL path segment {segment:?}"
        )));
    }
    let mut url = base.clone();
    url.path_segments_mut()
        .map_err(|_| APIError::InvalidRequest("Base URL cannot have a path".into()))?
        .pop_if_empty()
        .extend(segments.iter().copied());
    Ok(url)
}

fn validate_base_url(url: &Url) -> Result<(), APIError> {
    if !matches!(url.scheme(), "http" | "https")
        || url.host_str().is_none()
        || !url.username().is_empty()
        || url.password().is_some()
        || url.query().is_some()
        || url.fragment().is_some()
    {
        return Err(APIError::InvalidRequest(
            "Base URL must be an HTTP(S) URL without credentials, query, or fragment".into(),
        ));
    }
    Ok(())
}

type ErrorDecoder = fn(StatusCode, &[u8]) -> Option<ErrorResponse>;

/// Decodes an endpoint's documented reject body and wraps it with `wrap`, or
/// returns `None` so the caller falls back to [`CommonErrorResponse`].
pub(crate) fn decode_reject<E: DeserializeOwned>(
    body: &[u8],
    wrap: impl FnOnce(E) -> ErrorResponse,
) -> Option<ErrorResponse> {
    serde_json::from_slice(body).ok().map(wrap)
}

pub(crate) async fn decode_response<T: DeserializeOwned>(
    response: Response,
    success: StatusCode,
    decode_error: Option<ErrorDecoder>,
) -> Result<T, APIError> {
    let status = response.status();
    let request_id = response
        .headers()
        .get("RequestID")
        .and_then(|value| value.to_str().ok())
        .map(str::to_owned);
    let result = async {
        let body = response.bytes().await?;
        if status == success {
            return Ok(serde_json::from_slice::<T>(&body)?);
        }
        if let Some(error) = decode_error.and_then(|decode| decode(status, &body)) {
            return Err(APIError::from(error));
        }
        let error = serde_json::from_slice::<CommonErrorResponse>(&body)?;
        Err(APIError::from(ErrorResponse::CommonError(error)))
    }
    .await;
    result.map_err(|source| {
        APIError::Response(Box::new(HttpResponseError {
            status,
            request_id,
            source,
        }))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn account_url_trims_account_id() {
        let base = Url::parse("https://example.com/").unwrap();
        let url = account_url(&base, Some(&" 001-001-1234567-001\n".into()), &["orders"]).unwrap();
        assert_eq!(url.path(), "/v3/accounts/001-001-1234567-001/orders");
        assert!(matches!(
            account_url(&base, Some(&"  ".into()), &[]),
            Err(APIError::InvalidRequest(_))
        ));
    }

    #[test]
    fn dynamic_path_segments_are_encoded_individually() {
        let base = Url::parse("https://example.com/gateway/").unwrap();
        let url = account_url(
            &base,
            Some(&"account/one".into()),
            &["orders", "@client/one", "cancel"],
        )
        .unwrap();
        assert_eq!(
            url.path(),
            "/gateway/v3/accounts/account%2Fone/orders/@client%2Fone/cancel"
        );
    }

    #[test]
    fn dot_and_empty_path_segments_are_rejected() {
        let base = Url::parse("https://example.com/").unwrap();
        for specifier in ["", ".", ".."] {
            assert!(matches!(
                account_url(
                    &base,
                    Some(&"account".into()),
                    &["orders", specifier, "cancel"]
                ),
                Err(APIError::InvalidRequest(_))
            ));
        }
        assert!(matches!(
            account_url(&base, Some(&"..".into()), &["orders"]),
            Err(APIError::InvalidRequest(_))
        ));
    }
}
