use oanda_rust::order::*;
use oanda_rust::pricing::{ClientPrice, PricingStreamItem};
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
}
