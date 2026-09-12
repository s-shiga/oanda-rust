# oanda-rust

A typed Rust client for the OANDA v20 REST and streaming APIs.

## Testing

Run the offline suite:

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
