use oanda_rust::order::*;
use oanda_rust::position::ListPositionsResponse;
use oanda_rust::pricing::{ClientPrice, PriceBucket, PricingStreamItem};
use oanda_rust::transaction::{GetTransactionsResponse, Transaction, TransactionStreamItem};
use oanda_rust::transaction::{OrderCancelReason, TransactionRejectReason};
use oanda_rust::transaction::{TransactionFilter, TransactionType};
use serde_json::json;
use url::Url;

#[test]
fn regression_prices_accept_rest_stream_and_full_price_timestamps() {
    let time = "2026-09-12T00:00:00.123456789Z";
    let mut fixture = json!({"instrument":"EUR_USD", "bids":[], "asks":[],
        "closeoutBid":"1.1", "closeoutAsk":"1.2", "time":time});
    let price: ClientPrice = serde_json::from_value(fixture.clone()).unwrap();
    assert_eq!(
        price.timestamp.unwrap(),
        time.parse::<chrono::DateTime<chrono::Utc>>().unwrap()
    );
    fixture["type"] = json!("PRICE");
    let stream: PricingStreamItem = serde_json::from_value(fixture.clone()).unwrap();
    let PricingStreamItem::PRICE(price) = stream else {
        panic!("expected price")
    };
    assert!(price.timestamp.is_some());
    fixture.as_object_mut().unwrap().remove("time");
    fixture["timestamp"] = json!(time);
    let full_price: ClientPrice = serde_json::from_value(fixture).unwrap();
    assert_eq!(full_price.timestamp, price.timestamp);
}

#[test]
fn regression_order_requests_have_one_discriminator_and_round_trip() {
    let orders = [
        (
            "MARKET",
            OrderRequest::Market(MarketOrderRequest::new("EUR_USD".into(), "1".into())),
        ),
        (
            "LIMIT",
            OrderRequest::Limit(LimitOrderRequest::new(
                "EUR_USD".into(),
                "1".into(),
                "1.1".into(),
            )),
        ),
        (
            "STOP",
            OrderRequest::Stop(StopOrderRequest::new(
                "EUR_USD".into(),
                "1".into(),
                "1.1".into(),
            )),
        ),
        (
            "MARKET_IF_TOUCHED",
            OrderRequest::MarketIfTouched(MarketIfTouchedOrderRequest::new(
                "EUR_USD".into(),
                "1".into(),
                "1.1".into(),
            )),
        ),
        (
            "TAKE_PROFIT",
            OrderRequest::TakeProfit(TakeProfitOrderRequest::new("42".into(), "1.1".into())),
        ),
        (
            "STOP_LOSS",
            OrderRequest::StopLoss(StopLossOrderRequest::new("42".into(), "1.1".into())),
        ),
        (
            "GUARANTEED_STOP_LOSS",
            OrderRequest::GuaranteedStopLoss(GuaranteedStopLossOrderRequest::new(
                "42".into(),
                "1.1".into(),
            )),
        ),
        (
            "TRAILING_STOP_LOSS",
            OrderRequest::TrailingStopLoss(TrailingStopLossOrderRequest::new(
                "42".into(),
                "0.01".into(),
            )),
        ),
    ];
    for (tag, order) in orders {
        let order_wire = serde_json::to_string(&order).unwrap();
        let wire = serde_json::to_string(&CreateOrderRequest { order }).unwrap();
        assert_eq!(wire.matches("\"type\":").count(), 1, "{wire}");
        let value: serde_json::Value = serde_json::from_str(&wire).unwrap();
        assert_eq!(value["order"]["type"], tag);
        let decoded: OrderRequest = serde_json::from_str(&order_wire).unwrap();
        assert_eq!(serde_json::to_value(decoded).unwrap(), value["order"]);
    }
}

#[test]
fn regression_order_state_filters_use_wire_values() {
    for (state, expected) in [
        (OrderStateFilter::Pending, "PENDING"),
        (OrderStateFilter::Filled, "FILLED"),
        (OrderStateFilter::Triggered, "TRIGGERED"),
        (OrderStateFilter::Cancelled, "CANCELLED"),
        (OrderStateFilter::All, "ALL"),
    ] {
        let mut url = Url::parse("https://example.com/orders").unwrap();
        ListOrdersRequest::new().state(state).set_params(&mut url);
        assert_eq!(url.query(), Some(format!("state={expected}").as_str()));
    }
}

#[test]
fn regression_transaction_filter_values_use_api_casing() {
    assert_eq!(TransactionType::OrderFill.to_string(), "ORDER_FILL");
    assert_eq!(TransactionType::MarketOrder.to_string(), "MARKET_ORDER");
    assert_eq!(TransactionFilter::OrderFill.to_string(), "ORDER_FILL");
    assert_eq!(TransactionFilter::Funding.to_string(), "FUNDING");
    assert_eq!(
        TransactionFilter::ResetResettablePL.to_string(),
        "RESET_RESETTABLE_PL"
    );
    assert_eq!(
        serde_json::to_value(TransactionFilter::ResetResettablePL).unwrap(),
        "RESET_RESETTABLE_PL"
    );
    assert_eq!(
        serde_json::to_value(TransactionType::ResetResettablePL).unwrap(),
        "RESET_RESETTABLE_PL"
    );
}

#[test]
fn regression_new_and_unknown_reasons_do_not_break_transaction_decoding() {
    let event = |fields: serde_json::Value| {
        let mut event = json!({"id":"3", "time":"2026-09-12T00:00:00Z", "userID":1,
            "accountID":"account", "batchID":"3", "orderID":"1"});
        event
            .as_object_mut()
            .unwrap()
            .extend(fields.as_object().unwrap().clone());
        event
    };
    let response: GetTransactionsResponse = serde_json::from_value(json!({
        "lastTransactionID":"3", "transactions":[
            event(json!({"type":"ORDER_CANCEL", "reason":"GUARANTEED_STOP_LOSS_ON_FILL_NOT_ALLOWED"})),
            event(json!({"type":"ORDER_CANCEL_REJECT", "rejectReason":"GUARANTEED_STOP_LOSS_ORDER_NOT_ALLOWED"})),
            event(json!({"type":"ORDER_CANCEL", "reason":"REASON_ADDED_AFTER_THIS_RELEASE"})),
        ]
    }))
    .unwrap();
    assert!(matches!(
        &response.transactions[0],
        Transaction::OrderCancelTransaction(event)
            if matches!(event.reason, OrderCancelReason::GuaranteedStopLossOnFillNotAllowed)
    ));
    assert!(matches!(
        &response.transactions[1],
        Transaction::OrderCancelRejectTransaction(event)
            if matches!(event.reject_reason, TransactionRejectReason::GuaranteedStopLossOrderNotAllowed)
    ));
    assert!(matches!(
        &response.transactions[2],
        Transaction::OrderCancelTransaction(event)
            if matches!(event.reason, OrderCancelReason::Unknown)
    ));

    let item: TransactionStreamItem = serde_json::from_value(event(
        json!({"type":"ORDER_CANCEL_REJECT", "rejectReason":"REASON_ADDED_AFTER_THIS_RELEASE"}),
    ))
    .unwrap();
    let TransactionStreamItem::Transaction(transaction) = item else {
        panic!("expected transaction")
    };
    assert!(matches!(
        *transaction,
        Transaction::OrderCancelRejectTransaction(ref event)
            if matches!(event.reject_reason, TransactionRejectReason::Unknown)
    ));
}

#[test]
fn regression_positions_decode_documented_response_without_fee_fields() {
    // Example response for GET /v3/accounts/{accountID}/openPositions from
    // https://developer.oanda.com/rest-live-v20/position-ep/
    let response: ListPositionsResponse = serde_json::from_value(json!({
        "lastTransactionID": "6387",
        "positions": [
            {
                "instrument": "EUR_USD",
                "long": {
                    "averagePrice": "1.13032", "pl": "-54344.85056",
                    "resettablePL": "-54344.85056", "tradeIDs": ["6383", "6385"],
                    "units": "350", "unrealizedPL": "-0.04700"
                },
                "pl": "-54300.44169", "resettablePL": "-54300.44169",
                "short": {
                    "pl": "44.40887", "resettablePL": "44.40887",
                    "units": "0", "unrealizedPL": "0.00000"
                },
                "unrealizedPL": "-0.04700"
            },
            {
                "instrument": "USD_CAD",
                "long": {
                    "pl": "-483.91941", "resettablePL": "-483.91941",
                    "units": "0", "unrealizedPL": "0.00000"
                },
                "pl": "-486.16662", "resettablePL": "-486.16662",
                "short": {
                    "averagePrice": "1.28241", "pl": "-2.24721",
                    "resettablePL": "-2.24721", "tradeIDs": ["6387"],
                    "units": "-600", "unrealizedPL": "-0.08525"
                },
                "unrealizedPL": "-0.08525"
            }
        ]
    }))
    .unwrap();
    assert_eq!(response.positions.len(), 2);
    assert!(response.positions[0].financing.is_none());
    assert!(response.positions[0].long.financing.is_none());
    assert_eq!(response.positions[1].short.units, "-600");
}

#[test]
fn regression_invalid_liquidity_is_a_decode_error_not_a_panic() {
    let bucket: PriceBucket =
        serde_json::from_value(json!({"price":"1.1", "liquidity":"10000000"})).unwrap();
    assert_eq!(bucket.liquidity, 10_000_000);
    assert!(
        serde_json::from_value::<PriceBucket>(json!({"price":"1.1", "liquidity":"1.5"})).is_err()
    );
}

#[test]
fn regression_transaction_events_are_accessible_to_consumers() {
    let response: GetTransactionsResponse = serde_json::from_value(json!({
        "lastTransactionID":"2", "transactions":[{
            "type":"ORDER_CANCEL", "id":"2", "time":"2026-09-12T00:00:00Z",
            "userID":1, "accountID":"account", "batchID":"2", "orderID":"1",
            "reason":"CLIENT_REQUEST"
        }]
    }))
    .unwrap();
    let events = response.transactions;
    assert!(
        matches!(&events[0], Transaction::OrderCancelTransaction(event) if event.order_id == "1")
    );
}
