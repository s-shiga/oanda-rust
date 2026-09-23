# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.3.0] - 2026-09-23

### Upgrading from 0.2.0

- `Position` and `PositionSide` fee fields are now `Option<AccountUnits>`, because
  OANDA omits them: `financing`, `commission`, `dividend_adjustment` and
  `guaranteed_execution_fees` on `Position`, and `financing`,
  `dividend_adjustment` and `guaranteed_execution_fees` on `PositionSide`.
- `ListTransactionsRequest::transaction_type` takes a `TransactionFilter` instead of
  a `TransactionType`, matching OANDA's `type` parameter. Group filters such as
  `TransactionFilter::Order` and `TransactionFilter::Funding` now work.
- Structured error types require only `error_message`; every other field is an
  `Option`. This covers `OrderCreateErrorResponse`, `OrderCancelErrorResponse`,
  `UpdateOrderClientExtensionsErrorResponse`,
  `UpdateTradeClientExtensionsErrorResponse` and `ConfigureAccountErrorResponse`.
- `OrderCancelErrorResponse::order_cancel_reject_transaction` holds an
  `OrderCancelRejectTransaction` instead of an `OrderCreateRejectTransaction`.
- Matches on `ErrorResponse` need arms for the new `CloseTradeError` and
  `ClosePositionError` variants.
- Matches on `OrderCancelReason` and `TransactionRejectReason` need arms for the
  added values and the `Unknown` fallback.
- Rename `PricingStreamItem::PRICE` and `PricingStreamItem::HEARTBEAT` to `Price`
  and `Heartbeat`, and `TransactionStreamItem::HEARTBEAT` to `Heartbeat`. The JSON
  tags are unchanged.
- `order_fill_transaction` on `CreateOrderResponse` and `ReplaceOrderResponse` is
  now `Option<Box<OrderFillTransaction>>`.
- `primitives::AcceptDatetimeFormat` is removed. Nothing sent the header, and the
  crate decodes RFC 3339 timestamps only.
- The `request_setter!` and `request_option_setter!` macros are no longer
  exported.

### Added

- The 18 `OrderCancelReason` and 55 `TransactionRejectReason` values documented by
  OANDA but previously missing, including the guaranteed stop-loss reasons.
- An `Unknown` value on both reason enums, used for values OANDA adds later.
- `TransactionFilter::GuaranteedStopLossOrder` and
  `TransactionFilter::GuaranteedStopLossOrderReject`.
- `CloseTradeErrorResponse` and `ClosePositionErrorResponse`, returned when closing
  a trade or position is rejected.
- `ReplaceOrderResponse::replacing_order_cancel_transaction`, present when the
  replacement order is cancelled immediately.
- `InstrumentFinancing::financing_days_of_week`, the instrument's financing
  schedule.

### Changed

- A 400, 403 or 404 response with a documented reject body always decodes to that
  endpoint's error type, even when OANDA sends only `errorMessage`.
  `ErrorResponse::CommonError` is used for other statuses, such as 401.
- Error messages include OANDA's error code only when one was sent.
- IDs, specifiers and instrument names are encoded as single URL path segments,
  so a `/` in a client ID is sent as `%2F`.
- Account IDs and instrument names are trimmed of surrounding whitespace.
- These now return `APIError::InvalidRequest` before sending a request: an empty,
  `.` or `..` path segment, and an empty or blank pricing instrument list.
- A stream message longer than 1 MiB returns a JSON error instead of being
  buffered without limit.

### Fixed

- A transaction with an unrecognised reason no longer fails the whole
  transactions page or ends the transaction stream.
- Position responses decode when OANDA omits fee fields, as in its documented
  examples.
- `Account` and `AccountSummary` decode when `resettablePLTime` is missing.
- A non-integer `liquidity` string returns a decode error instead of panicking.
- `with_base_url` keeps a path prefix such as `https://host/gateway/`; it was
  previously discarded.
- Reject bodies from order cancel and replace (404), order create (404) and trade
  client-extension updates now decode instead of falling back to
  `CommonError`.
- `TransactionType::ResetResettablePL` and `TransactionFilter::ResetResettablePL`
  serialize with serde as `RESET_RESETTABLE_PL` (was `RESET_RESETTABLE_P_L`).

## [0.2.0] - 2026-09-12

### Upgrading from 0.1.0

- Add `?` or handle the result from `Client::new`, `Client::new_practice`, and the
  corresponding `StreamClient` constructors.
- Match response failures through `APIError::Response(context)` and inspect
  `context.source` for structured API errors or decoding failures.
- The exported `handle_response!` macro has been replaced by internal typed
  response helpers.
- Order request structs no longer have an `order_type` field; the order type
  comes from the `OrderRequest` variant, such as `OrderRequest::Market`.

### Added

- `with_http_client` and `with_base_url` on `Client` and `StreamClient`, to supply
  a custom `reqwest::Client` or endpoint.
- `HttpResponseError`, which keeps the HTTP status and OANDA `RequestID` of a
  failed response.
- `GetTransactionsResponse::transactions` is public.
- Stream handlers accept `FnMut` closures, so they can capture state.

### Changed

- API tokens are validated when a client is created.
- The pricing and transaction streams share one newline-delimited JSON parser. A
  final message without a newline is delivered, and truncated JSON returns an
  error.

### Fixed

- Transaction filters are sent as the `type` query parameter, in OANDA's casing
  (for example `ORDER_FILL`).
- Transaction date ranges are sent as RFC 3339.
- Pricing responses keep their timestamp, whether OANDA sends `time` or
  `timestamp`.
- Order requests serialize the `type` discriminator once.

## [0.1.0] - 2026-09-10

Initial release.

[Unreleased]: https://github.com/s-shiga/oanda-rust/compare/v0.3.0...HEAD
[0.3.0]: https://github.com/s-shiga/oanda-rust/compare/v0.2.0...v0.3.0
[0.2.0]: https://github.com/s-shiga/oanda-rust/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/s-shiga/oanda-rust/tree/v0.1.0
