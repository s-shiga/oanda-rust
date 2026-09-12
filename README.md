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

## Testing

Run the offline suite, including fixtures served on localhost:

```sh
cargo test
```

Tests that contact the OANDA practice API are ignored by default. Set
`OANDA_API_KEY_DEMO` and `OANDA_ACCOUNT_ID_DEMO`, then opt in explicitly:

```sh
cargo test --lib -- --ignored
```

Tests that create orders, close positions, or change account configuration also
require the `write-tests` feature. To run only those tests:

```sh
cargo test --features write-tests --lib write_tests:: -- --ignored
```

## Migration notes

- Add `?` or handle the result from `Client::new`, `Client::new_practice`, and the
  corresponding `StreamClient` constructors.
