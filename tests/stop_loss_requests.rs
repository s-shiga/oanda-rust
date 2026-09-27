use oanda_rust::order::{
    CreateOrderResponse, GuaranteedStopLossOrderRequest, MarketOrderRequest, OrderRequest,
    StopLossOrderRequest,
};
use oanda_rust::transaction::{GuaranteedStopLossDetails, StopLossDetails};
use serde_json::json;

#[test]
fn distance_setters_replace_the_absolute_price() {
    for order in [
        OrderRequest::StopLoss(
            StopLossOrderRequest::new("42".into(), "1.10".into()).distance("0.01".into()),
        ),
        OrderRequest::GuaranteedStopLoss(
            GuaranteedStopLossOrderRequest::new("42".into(), "1.10".into()).distance("0.01".into()),
        ),
    ] {
        let value = serde_json::to_value(order).unwrap();
        assert_eq!(value["distance"], "0.01");
        assert!(value.get("price").is_none());
        let decoded: OrderRequest = serde_json::from_value(value.clone()).unwrap();
        assert_eq!(serde_json::to_value(decoded).unwrap(), value);
    }
}

#[test]
fn on_fill_details_accept_distance_without_price() {
    let value = json!({"distance":"0.01", "timeInForce":"GTC"});
    let stop: StopLossDetails = serde_json::from_value(value.clone()).unwrap();
    let guaranteed: GuaranteedStopLossDetails = serde_json::from_value(value).unwrap();
    for value in [
        serde_json::to_value(stop).unwrap(),
        serde_json::to_value(guaranteed).unwrap(),
    ] {
        assert_eq!(value["distance"], "0.01");
        assert!(value.get("price").is_none());
    }
}

#[test]
fn distance_constructors_do_not_require_a_placeholder_price() {
    for order in [
        OrderRequest::StopLoss(StopLossOrderRequest::from_distance(
            "42".into(),
            "0.01".into(),
        )),
        OrderRequest::GuaranteedStopLoss(GuaranteedStopLossOrderRequest::from_distance(
            "42".into(),
            "0.01".into(),
        )),
    ] {
        let value = serde_json::to_value(order).unwrap();
        assert_eq!(value["distance"], "0.01");
        assert!(value.get("price").is_none());
    }
    for value in [
        serde_json::to_value(StopLossDetails::new("1.10".into())).unwrap(),
        serde_json::to_value(GuaranteedStopLossDetails::new("1.10".into())).unwrap(),
    ] {
        assert_eq!(value, json!({"price":"1.10", "timeInForce":"GTC"}));
    }
}

#[test]
fn market_orders_and_creation_responses_keep_distance_on_fill() {
    for (field, request) in [
        (
            "stopLossOnFill",
            MarketOrderRequest::new("EUR_USD".into(), "100".into())
                .stop_loss_on_fill(StopLossDetails::from_distance("0.01".into())),
        ),
        (
            "guaranteedStopLossOnFill",
            MarketOrderRequest::new("EUR_USD".into(), "100".into()).guaranteed_stop_loss_on_fill(
                GuaranteedStopLossDetails::from_distance("0.01".into()),
            ),
        ),
    ] {
        let wire = serde_json::to_value(OrderRequest::Market(request)).unwrap();
        assert_eq!(wire[field], json!({"distance":"0.01", "timeInForce":"GTC"}));
        let decoded: OrderRequest = serde_json::from_value(wire.clone()).unwrap();
        assert_eq!(serde_json::to_value(decoded).unwrap(), wire);
        let mut response: serde_json::Value =
            serde_json::from_str(include_str!("fixtures/order_create_market_buy.json")).unwrap();
        response["orderCreateTransaction"][field] = wire[field].clone();
        let decoded: CreateOrderResponse = serde_json::from_value(response).unwrap();
        let serialized = serde_json::to_value(decoded).unwrap();
        assert_eq!(serialized["orderCreateTransaction"][field], wire[field]);
    }
}

#[test]
fn stop_loss_thresholds_reject_both_missing_and_malformed_values() {
    for threshold in [
        json!({}),
        json!({"price":"1.10", "distance":"0.01"}),
        json!({"price":true}),
        json!({"distance":{}}),
    ] {
        let mut details = threshold.clone();
        details["timeInForce"] = json!("GTC");
        assert!(serde_json::from_value::<StopLossDetails>(details.clone()).is_err());
        assert!(serde_json::from_value::<GuaranteedStopLossDetails>(details).is_err());
        for kind in ["STOP_LOSS", "GUARANTEED_STOP_LOSS"] {
            let mut order = threshold.clone();
            order["type"] = json!(kind);
            order["tradeID"] = json!("42");
            order["timeInForce"] = json!("GTC");
            order["triggerCondition"] = json!("DEFAULT");
            assert!(serde_json::from_value::<OrderRequest>(order).is_err());
        }
    }
}
