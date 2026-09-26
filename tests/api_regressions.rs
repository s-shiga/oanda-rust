use oanda_rust::account::{Account, AccountSummary};
use oanda_rust::instrument::{DayOfWeek, InstrumentFinancing};
use oanda_rust::order::*;
use oanda_rust::position::ListPositionsResponse;
use oanda_rust::pricing::{ClientPrice, PriceBucket, PricingStreamItem};
use oanda_rust::transaction::{
    GetTransactionsResponse, OrderFillTransaction, Transaction, TransactionStreamItem,
};
use oanda_rust::transaction::{OrderCancelReason, TransactionRejectReason};
use oanda_rust::transaction::{TransactionFilter, TransactionType};
use serde_json::json;
use url::Url;

#[test]
fn regression_documented_order_fills_decode_in_history_and_streams() {
    for wire in [
        include_str!("fixtures/order_create_market_buy.json"),
        include_str!("fixtures/order_create_market_sell.json"),
    ] {
        let response: serde_json::Value = serde_json::from_str(wire).unwrap();
        let fill = &response["orderFillTransaction"];
        let history: GetTransactionsResponse = serde_json::from_value(json!({
            "lastTransactionID": response["lastTransactionID"],
            "transactions": [response["orderCreateTransaction"], fill]
        }))
        .unwrap();
        assert_eq!(history.transactions.len(), 2);
        let Transaction::OrderFillTransaction(event) = &history.transactions[1] else {
            panic!("expected order fill")
        };
        assert_eq!(event.id, fill["id"].as_str().unwrap());
        assert_eq!(event.price.as_deref(), fill["price"].as_str());
        assert!(event.home_conversion_factors.is_none());
        assert!(event.full_vwap.is_none());
        assert!(event.base_financing.is_none());
        assert!(event.quote_guaranteed_execution_fee.is_none());
        let opened = event.trade_opened.as_ref().unwrap();
        assert_eq!(opened.trade_id, event.id);
        if event.id == "6368" {
            assert!(event.full_price.is_none());
            assert!(event.commission.is_none());
            assert!(event.guaranteed_execution_fee.is_none());
            assert!(event.half_spread_cost.is_none());
            assert!(opened.price.is_none());
            assert!(opened.guaranteed_execution_fee.is_none());
            assert!(opened.half_spread_cost.is_none());
            assert!(opened.initial_margin_required.is_none());
        } else {
            assert_eq!(event.full_price.as_ref().unwrap().closeout_bid, "1.22794");
            assert_eq!(event.commission.as_deref(), Some("0.0000"));
            assert_eq!(event.guaranteed_execution_fee.as_deref(), Some("0.0000"));
            assert_eq!(event.half_spread_cost.as_deref(), Some("0.0078"));
            assert_eq!(
                event.gain_quote_home_conversion_factor.as_deref(),
                Some("1.30568")
            );
            assert_eq!(
                event.loss_quote_home_conversion_factor.as_deref(),
                Some("1.30588")
            );
            assert_eq!(opened.price.as_deref(), Some("1.22809"));
            assert_eq!(opened.guaranteed_execution_fee.as_deref(), Some("0.0000"));
            assert_eq!(opened.half_spread_cost.as_deref(), Some("0.0078"));
            assert_eq!(opened.initial_margin_required.as_deref(), Some("8.3391"));
        }

        let stream: TransactionStreamItem = serde_json::from_value(fill.clone()).unwrap();
        let TransactionStreamItem::Transaction(transaction) = stream else {
            panic!("expected transaction")
        };
        assert!(matches!(
            *transaction,
            Transaction::OrderFillTransaction(ref streamed) if streamed.id == event.id
        ));
    }
}

#[test]
fn regression_order_fills_preserve_additional_details_when_reported() {
    let mut response: serde_json::Value =
        serde_json::from_str(include_str!("fixtures/order_create_market_sell.json")).unwrap();
    let fill = &mut response["orderFillTransaction"];
    fill["homeConversionFactors"] = json!({
        "gainQuoteHome": {"factor": "1.30568"},
        "lossQuoteHome": {"factor": "1.30588"},
        "gainBaseHome": {"factor": "1.6035"},
        "lossBaseHome": {"factor": "1.6040"}
    });
    fill["fullVWAP"] = json!("1.22810");
    fill["baseFinancing"] = json!("-0.0123");
    fill["quoteFinancing"] = json!("-0.0151");
    fill["quoteGuaranteedExecutionFee"] = json!("0.0020");
    let decoded: CreateOrderResponse = serde_json::from_value(response.clone()).unwrap();
    let event = decoded.order_fill_transaction.unwrap();
    assert_eq!(event.full_vwap.as_deref(), Some("1.22810"));
    assert_eq!(event.price.as_deref(), Some("1.22809"));
    assert_eq!(event.base_financing.as_deref(), Some("-0.0123"));
    assert_eq!(event.quote_financing.as_deref(), Some("-0.0151"));
    assert_eq!(
        event.quote_guaranteed_execution_fee.as_deref(),
        Some("0.0020")
    );
    let price = event.full_price.as_ref().unwrap();
    assert_eq!(price.bids[0].price, "1.22809");
    assert_eq!(price.asks[0].price, "1.22821");
    assert_eq!(price.bids[0].liquidity, 10_000_000);
    let serialized = serde_json::to_value(event).unwrap();
    for field in [
        "homeConversionFactors",
        "gainQuoteHomeConversionFactor",
        "lossQuoteHomeConversionFactor",
        "price",
        "fullVWAP",
        "baseFinancing",
        "quoteFinancing",
        "quoteGuaranteedExecutionFee",
    ] {
        assert_eq!(
            serialized[field], response["orderFillTransaction"][field],
            "{field}"
        );
    }
}

#[test]
fn regression_order_fills_still_reject_missing_identity_and_invalid_details() {
    let response: serde_json::Value =
        serde_json::from_str(include_str!("fixtures/order_create_market_sell.json")).unwrap();
    let mut missing_id = response["orderFillTransaction"].clone();
    missing_id.as_object_mut().unwrap().remove("orderID");
    let error = serde_json::from_value::<OrderFillTransaction>(missing_id).unwrap_err();
    assert!(error.to_string().contains("missing field `orderID`"));

    let mut invalid_price = response["orderFillTransaction"].clone();
    invalid_price["fullVWAP"] = json!({"invalid": "price"});
    assert!(serde_json::from_value::<OrderFillTransaction>(invalid_price).is_err());

    let mut invalid_trade = response["orderFillTransaction"].clone();
    invalid_trade["tradeOpened"]["initialMarginRequired"] = json!(false);
    assert!(serde_json::from_value::<OrderFillTransaction>(invalid_trade).is_err());
}

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
    let PricingStreamItem::Price(price) = stream else {
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
fn regression_replace_response_keeps_replacing_order_cancel_transaction() {
    let response: ReplaceOrderResponse = serde_json::from_value(json!({
        "replacingOrderCancelTransaction": {
            "type":"ORDER_CANCEL", "id":"5", "time":"2026-09-12T00:00:00Z", "userID":1,
            "accountID":"account", "batchID":"4", "orderID":"4", "reason":"INSUFFICIENT_MARGIN"
        },
        "relatedTransactionIDs":["4", "5"], "lastTransactionID":"5"
    }))
    .unwrap();
    let cancel = response.replacing_order_cancel_transaction.unwrap();
    assert_eq!(cancel.order_id, "4");
    assert!(matches!(
        cancel.reason,
        OrderCancelReason::InsufficientMargin
    ));
}

#[test]
fn regression_instrument_financing_keeps_financing_days() {
    let financing: InstrumentFinancing = serde_json::from_value(json!({
        "longRate":"-0.0147", "shortRate":"-0.0053",
        "financingDaysOfWeek":[
            {"dayOfWeek":"MONDAY", "daysCharged":1},
            {"dayOfWeek":"WEDNESDAY", "daysCharged":3}
        ]
    }))
    .unwrap();
    let days = financing.financing_days_of_week.unwrap();
    assert!(matches!(days[1].day_of_week, DayOfWeek::Wednesday));
    assert_eq!(days[1].days_charged, 3);

    let financing: InstrumentFinancing =
        serde_json::from_value(json!({"longRate":"-0.0147", "shortRate":"-0.0053"})).unwrap();
    assert!(financing.financing_days_of_week.is_none());
}

#[test]
fn regression_accounts_decode_with_resettable_pl_time_absent_zero_or_set() {
    let base = json!({"id":"001", "currency":"USD", "createdByUserID":1,
        "createdTime":"2026-01-01T00:00:00Z"});
    let summary: AccountSummary = serde_json::from_value(base.clone()).unwrap();
    assert!(summary.resettable_pl_time.is_none());
    let account: Account = serde_json::from_value(base.clone()).unwrap();
    assert!(account.resettable_pl_time.is_none());

    let mut zero = base.clone();
    zero["resettablePLTime"] = json!("0");
    let account: Account = serde_json::from_value(zero).unwrap();
    assert!(account.resettable_pl_time.is_none());

    let mut set = base;
    set["resettablePLTime"] = json!("2026-02-01T00:00:00Z");
    let account: Account = serde_json::from_value(set).unwrap();
    assert!(account.resettable_pl_time.is_some());
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
