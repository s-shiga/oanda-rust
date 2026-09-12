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

Streaming handlers accept `FnMut`, so they can capture counters, channels, or
other application state. Both endpoints use the same newline-delimited JSON
parser. A complete final message without a newline is delivered; truncated JSON
at EOF returns an error. Handlers run synchronously and returning an error stops
dispatch immediately.

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

## Migration notes

- Add `?` or handle the result from `Client::new`, `Client::new_practice`, and the
  corresponding `StreamClient` constructors.
- Match response failures through `APIError::Response(context)` and inspect
  `context.source` for structured API errors or decoding failures.
- The exported `handle_response!` macro has been replaced by internal typed
  response helpers.
