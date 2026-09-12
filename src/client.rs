use crate::account::{AccountID, AccountService};
use crate::errors::APIError;
use crate::http::{self, HttpClient};
use crate::instrument::InstrumentService;
use crate::order::OrderService;
use crate::position::PositionService;
use crate::pricing::PricingService;
use crate::trade::TradeService;
use crate::transaction::TransactionService;
use url::Url;

/// Base URL for the OANDA live/production trading environment.
const FX_TRADE_URL: &str = "https://api-fxtrade.oanda.com";

/// Base URL for the OANDA practice (paper trading) environment.
const FX_TRADE_PRACTICE_URL: &str = "https://api-fxpractice.oanda.com";

/// The main entry point for interacting with the OANDA v20 REST API.
///
/// `Client` holds an authenticated HTTP client and optional account context,
/// and exposes service accessors for each API domain (accounts, orders,
/// trades, positions, etc.).
///
/// # Examples
///
/// ```no_run
/// use oanda_rust::client::Client;
///
/// let client = Client::new_practice("YOUR_API_KEY").unwrap()
///     .with_account_id("YOUR_ACCOUNT_ID".to_string());
///
/// // Access a specific service:
/// let account_service = client.account();
/// ```
pub struct Client {
    pub(crate) base_url: Url,
    pub(crate) http_client: HttpClient,
    pub(crate) account_id: Option<AccountID>,
}

impl<'a> Client {
    /// Creates a client targeting the **live** OANDA trading environment.
    ///
    /// Use [`Client::new_practice`] instead if you want to test against the
    /// paper-trading (fxpractice) environment.
    ///
    /// # Arguments
    ///
    /// * `api_key` – Your OANDA personal access token.
    ///
    /// Returns an error if the token is invalid or the HTTP client cannot be built.
    pub fn new(api_key: &str) -> Result<Client, APIError> {
        Ok(Client {
            base_url: Url::parse(FX_TRADE_URL).unwrap(),
            http_client: HttpClient::new(api_key, "application/json")?,
            account_id: None,
        })
    }

    /// Creates a client targeting the OANDA **practice** (paper trading) environment.
    ///
    /// # Arguments
    ///
    /// * `api_key` – Your OANDA practice personal access token.
    ///
    /// Returns an error if the token is invalid or the HTTP client cannot be built.
    pub fn new_practice(api_key: &str) -> Result<Client, APIError> {
        Ok(Client {
            base_url: Url::parse(FX_TRADE_PRACTICE_URL).unwrap(),
            http_client: HttpClient::new(api_key, "application/json")?,
            account_id: None,
        })
    }

    /// Uses a custom HTTP client while preserving OANDA authentication headers.
    pub fn with_http_client(mut self, client: reqwest::Client) -> Self {
        self.http_client.client = client;
        self
    }

    /// Overrides the API endpoint, for example for a local fixture server.
    /// Requests to this endpoint include the configured API token.
    pub fn with_base_url(mut self, url: Url) -> Result<Self, APIError> {
        http::validate_base_url(&url)?;
        self.base_url = url;
        Ok(self)
    }

    /// Sets the default account ID used by service calls that require one.
    ///
    /// Returns `self` so that this method can be chained directly after
    /// [`Client::new`] or [`Client::new_practice`].
    pub fn with_account_id(mut self, account_id: AccountID) -> Self {
        self.account_id = Some(account_id);
        self
    }

    /// Returns an [`AccountService`] for account-related API operations.
    pub fn account(&'a self) -> AccountService<'a> {
        AccountService::new(self)
    }

    /// Returns an [`InstrumentService`] for instrument/candlestick API operations.
    pub fn instrument(&'a self) -> InstrumentService<'a> {
        InstrumentService::new(self)
    }

    /// Returns an [`OrderService`] for order management API operations.
    pub fn order(&'a self) -> OrderService<'a> {
        OrderService::new(self)
    }

    /// Returns a [`TransactionService`] for transaction history API operations.
    pub fn transaction(&'a self) -> TransactionService<'a> {
        TransactionService::new(self)
    }

    /// Returns a [`PositionService`] for position management API operations.
    pub fn position(&'a self) -> PositionService<'a> {
        PositionService::new(self)
    }

    /// Returns a [`TradeService`] for trade management API operations.
    pub fn trade(&'a self) -> TradeService<'a> {
        TradeService::new(self)
    }

    /// Returns a [`PricingService`] for real-time pricing API operations.
    pub fn pricing(&'a self) -> PricingService<'a> {
        PricingService::new(self)
    }

    /// Builds a URL rooted at `/v3/accounts/{accountID}/{suffix}`.
    ///
    /// `suffix` is the path segment(s) after the account ID (e.g. `"positions"`,
    /// `"orders/123/cancel"`). Pass an empty string to target the account root.
    ///
    /// Returns [`APIError::InvalidRequest`] if no account ID is configured.
    pub(crate) fn account_url(&self, suffix: &str) -> Result<Url, APIError> {
        http::account_url(&self.base_url, self.account_id.as_ref(), suffix)
    }
}

/// Generates a builder-style setter method for a required (non-`Option`) field.
///
/// Expands to a method `pub fn $attr(mut self, $attr: $ty) -> Self` that
/// assigns the provided value and returns `self` for chaining.
#[macro_export]
macro_rules! request_setter {
    ($attr:ident, $ty:ty) => {
        pub fn $attr(mut self, $attr: $ty) -> Self {
            self.$attr = $attr;
            self
        }
    };
}

/// Generates a builder-style setter method for an optional (`Option<T>`) field.
///
/// Expands to a method `pub fn $attr(mut self, $attr: $ty) -> Self` that
/// wraps the value in `Some(...)`, assigns it, and returns `self` for chaining.
#[macro_export]
macro_rules! request_option_setter {
    ($attr:ident, $ty:ty) => {
        pub fn $attr(mut self, $attr: $ty) -> Self {
            self.$attr = Some($attr);
            self
        }
    };
}

#[cfg(test)]
pub(crate) fn setup_test_client() -> Client {
    let api_key = std::env::var("OANDA_API_KEY_DEMO").expect("OANDA_API_KEY_DEMO must be set");
    let account_id =
        std::env::var("OANDA_ACCOUNT_ID_DEMO").expect("OANDA_ACCOUNT_ID_DEMO must be set");
    Client::new_practice(&api_key)
        .unwrap()
        .with_account_id(account_id)
}
