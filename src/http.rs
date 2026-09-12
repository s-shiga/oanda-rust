//! Shared authenticated transport and response decoding.
use crate::account::AccountID;
use crate::errors::APIError;
use reqwest::header::{HeaderMap, HeaderValue, ACCEPT, AUTHORIZATION};
use reqwest::{Request, RequestBuilder, Response};
use url::Url;

pub(crate) struct HttpClient {
    pub(crate) client: reqwest::Client,
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

    pub(crate) async fn execute(&self, mut request: Request) -> Result<Response, reqwest::Error> {
        request.headers_mut().extend(self.headers.clone());
        self.client.execute(request).await
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

pub(crate) fn account_url(
    base: &Url,
    account: Option<&AccountID>,
    suffix: &str,
) -> Result<Url, APIError> {
    let id = account
        .filter(|id| !id.trim().is_empty())
        .ok_or_else(|| APIError::InvalidRequest("Missing account_id".into()))?;
    let path = if suffix.is_empty() {
        format!("/v3/accounts/{id}")
    } else {
        format!("/v3/accounts/{id}/{suffix}")
    };
    base.join(&path)
        .map_err(|error| APIError::InvalidRequest(error.to_string()))
}

pub(crate) fn validate_base_url(url: &Url) -> Result<(), APIError> {
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
