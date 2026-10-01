# oanda-rust

A typed Rust client for the OANDA v20 REST and streaming APIs.

```rust,no_run
use oanda_rust::{client::Client, errors::APIError};

async fn example() -> Result<(), APIError> {
    let client = Client::new_practice("YOUR_API_KEY")?
        .with_account_id("YOUR_ACCOUNT_ID".into());
    let prices = client.pricing().get(vec!["EUR_USD".into()]).await?;
    println!("{:?}", prices);
    Ok(())
}
```

Both `Client` and `StreamClient` constructors return `Result`. Missing or empty
account context produces `APIError::InvalidRequest` when an account endpoint is
called. Tokens are validated before building the HTTP client.

Use `with_http_client(reqwest_client)` to configure timeouts, proxies, or other
transport options. Authentication and Accept headers are retained when replacing
the HTTP client. `with_base_url(url)?` overrides the endpoint for local fixtures
or a custom gateway; requests to that endpoint include the configured token.

`StreamClient::pricing_stream` and `transactions_stream` return pull-based
streams. Use `futures_util::StreamExt::next` to await each item, and drop the
stream to close the connection. `pricing_stream_with_options` can disable the
connect snapshot or request home-currency conversion factors, delivered as
`PricingStreamItem::HomeConversions`. The caller owns
stall detection, reconnection, and transaction catch-up through the REST API.

The callback methods `pricing` and `transactions` remain available. They
accept `FnMut` and use the same newline-delimited JSON parser. A complete final
message without a newline is delivered; truncated JSON at EOF returns an error.
Returning an error from a handler stops dispatch immediately.

The REST services also expose the following candle, book, price-history, and
trade-order methods:

| Method | Purpose |
| --- | --- |
| `pricing().candles_latest(LatestCandlesRequest)` | Latest completed candles for multiple instrument/granularity/component series |
| `pricing().candlesticks(AccountCandlesticksRequest)` | Account-specific candles, including volume-weighted bid/ask prices through `units` |
| `pricing().get_with_options(instruments, PricingOptions)` | Prices filtered by `since`, with optional home conversions and legacy units availability |
| `instrument().order_book(instrument, time)` | Latest order-book snapshot, or a snapshot at the supplied UTC time |
| `instrument().position_book(instrument, time)` | Latest position-book snapshot, or a snapshot at the supplied UTC time |
| `instrument().price(instrument, time)` | Current instrument price, or the price at the supplied UTC time |
| `instrument().prices(InstrumentPricesRequest)` | One page of prices starting at a required UTC time, with an optional end time |
| `trade().update_orders(specifier, UpdateTradeOrdersRequest)` | Create, replace, or cancel a trade's dependent orders |

The existing `pricing().get(instruments)` continues to use OANDA's default
options. Instrument book and price endpoints do not require an account ID.
These methods follow the [OANDA pricing reference](https://developer.oanda.com/rest-live-v20/pricing-ep/),
[trade reference](https://developer.oanda.com/rest-live-v20/trade-ep/), and
[official instrument specification](https://github.com/oanda/v20-openapi/blob/master/yaml/separate/v20_instrument.yaml).

```rust,no_run
use oanda_rust::{client::Client, errors::APIError};
use oanda_rust::instrument::{CandlestickGranularity, CandlesticksRequest};
use oanda_rust::pricing::{AccountCandlesticksRequest, LatestCandlesRequest, PricingOptions};
use oanda_rust::trade::{StopLossOrderUpdate, UpdateTradeOrdersRequest};

async fn candles_and_orders(client: &Client) -> Result<(), APIError> {
    let latest = client.pricing().candles_latest(
        LatestCandlesRequest::new(vec!["EUR_USD:H1:M".into(), "USD_JPY:M5:BA".into()])
    ).await?;
    let candles = client.pricing().candlesticks(
        AccountCandlesticksRequest::new(
            CandlesticksRequest::new("EUR_USD".into())
                .bid().ask().granularity(CandlestickGranularity::H1).count(100)?
        ).units("1000".into())
    ).await?;
    let prices = client.pricing().get_with_options(
        vec!["EUR_USD".into()], PricingOptions::new().include_home_conversions(true)
    ).await?;
    let orders = client.trade().update_orders(
        "6397".into(),
        UpdateTradeOrdersRequest::new()
            .stop_loss(Some(StopLossOrderUpdate::new().price("1.0800".into())))
            .take_profit(None)
    ).await?;
    println!("{:?} {:?} {:?} {:?}", latest, candles, prices, orders);
    Ok(())
}
```

On `UpdateTradeOrdersRequest`, an unset order is unchanged, a setter passed
`None` cancels that order, and `Some(details)` creates or replaces it. Unset
fields inside the update details are omitted, allowing OANDA to inherit the
existing order's values on replacement. `StopLossOrderUpdate` and
`GuaranteedStopLossOrderUpdate` accept either a price or a distance; setting
one replaces the other.

HTTP response failures are wrapped in `APIError::Response`. The contained
`HttpResponseError` exposes `status`, `request_id`, and `source`; structured OANDA
errors remain available as `APIError::ErrorResponse` inside `source`. Transport
failures before receiving a response use `APIError::HTTPError`.

## Testing

Run the offline suite, including fixtures served on localhost:

```sh
cargo test
```

All tests use local data or localhost HTTP fixtures. No API credentials or
connections to OANDA are required.

## Changelog

See [CHANGELOG.md](CHANGELOG.md) for what changed in each release and how to
upgrade between versions.
