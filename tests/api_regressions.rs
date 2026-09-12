use oanda_rust::pricing::{ClientPrice, PricingStreamItem};
use serde_json::json;

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
