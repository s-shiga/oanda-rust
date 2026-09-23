use crate::account::{AccountID, AccountService};
use crate::errors::APIError;
use crate::http::Connection;
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
    connection: Connection,
}

impl Client {
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
            connection: Connection::new(api_key, "application/json", FX_TRADE_URL)?,
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
            connection: Connection::new(api_key, "application/json", FX_TRADE_PRACTICE_URL)?,
        })
    }

    /// Uses a custom HTTP client while preserving OANDA authentication headers.
    pub fn with_http_client(mut self, client: reqwest::Client) -> Self {
        self.connection.set_http_client(client);
        self
    }

    /// Overrides the API endpoint, for example for a local fixture server.
    /// Requests to this endpoint include the configured API token.
    pub fn with_base_url(mut self, url: Url) -> Result<Self, APIError> {
        self.connection.set_base_url(url)?;
        Ok(self)
    }

    /// Sets the default account ID used by service calls that require one.
    ///
    /// Returns `self` so that this method can be chained directly after
    /// [`Client::new`] or [`Client::new_practice`].
    pub fn with_account_id(mut self, account_id: AccountID) -> Self {
        self.connection.account_id = Some(account_id);
        self
    }

    /// Returns an [`AccountService`] for account-related API operations.
    pub fn account(&self) -> AccountService<'_> {
        AccountService::new(&self.connection)
    }

    /// Returns an [`InstrumentService`] for instrument/candlestick API operations.
    pub fn instrument(&self) -> InstrumentService<'_> {
        InstrumentService::new(&self.connection)
    }

    /// Returns an [`OrderService`] for order management API operations.
    pub fn order(&self) -> OrderService<'_> {
        OrderService::new(&self.connection)
    }

    /// Returns a [`TransactionService`] for transaction history API operations.
    pub fn transaction(&self) -> TransactionService<'_> {
        TransactionService::new(&self.connection)
    }

    /// Returns a [`PositionService`] for position management API operations.
    pub fn position(&self) -> PositionService<'_> {
        PositionService::new(&self.connection)
    }

    /// Returns a [`TradeService`] for trade management API operations.
    pub fn trade(&self) -> TradeService<'_> {
        TradeService::new(&self.connection)
    }

    /// Returns a [`PricingService`] for real-time pricing API operations.
    pub fn pricing(&self) -> PricingService<'_> {
        PricingService::new(&self.connection)
    }
}

/// Generates a builder-style setter method for a required (non-`Option`) field.
///
/// Expands to a method `pub fn $attr(mut self, $attr: $ty) -> Self` that
/// assigns the provided value and returns `self` for chaining.
macro_rules! request_setter {
    ($attr:ident, $ty:ty) => {
        pub fn $attr(mut self, $attr: $ty) -> Self {
            self.$attr = $attr;
            self
        }
    };
}
pub(crate) use request_setter;

/// Generates a builder-style setter method for an optional (`Option<T>`) field.
///
/// Expands to a method `pub fn $attr(mut self, $attr: $ty) -> Self` that
/// wraps the value in `Some(...)`, assigns it, and returns `self` for chaining.
macro_rules! request_option_setter {
    ($attr:ident, $ty:ty) => {
        pub fn $attr(mut self, $attr: $ty) -> Self {
            self.$attr = Some($attr);
            self
        }
    };
}
pub(crate) use request_option_setter;
