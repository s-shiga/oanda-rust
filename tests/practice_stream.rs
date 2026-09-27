//! Read-only smoke test against an OANDA practice account.

use futures_util::StreamExt;
use oanda_rust::client::Client;
use oanda_rust::pricing::PricingStreamItem;
use oanda_rust::stream::StreamClient;
use oanda_rust::transaction::TransactionStreamItem;
use std::time::Duration;

#[tokio::test]
#[ignore = "requires practice credentials and network access"]
async fn practice_streams_receive_price_and_heartbeats() {
    let token = std::env::var("OANDA_API_TOKEN").expect("OANDA_API_TOKEN is required");
    let alias = std::env::var("OANDA_ACCOUNT_ALIAS").expect("OANDA_ACCOUNT_ALIAS is required");
    let rest = Client::new_practice(&token).unwrap().with_http_client(
        reqwest::Client::builder()
            .timeout(Duration::from_secs(10))
            .build()
            .unwrap(),
    );
    let mut selected = None;
    for account in rest.account().list().await.unwrap().accounts {
        let details = rest.account().get_details(&account.id).await.unwrap();
        if details.account.alias.as_deref() == Some(alias.as_str()) {
            assert!(
                selected.replace(account.id).is_none(),
                "account alias is ambiguous"
            );
        }
    }
    let account_id = selected.expect("no practice account matches OANDA_ACCOUNT_ALIAS");
    let stream_client = StreamClient::new_practice(&token)
        .unwrap()
        .with_http_client(
            reqwest::Client::builder()
                .connect_timeout(Duration::from_secs(10))
                .build()
                .unwrap(),
        )
        .with_account_id(account_id);

    let prices = stream_client.pricing_stream(&["USD_JPY"]).await.unwrap();
    futures_util::pin_mut!(prices);
    let mut saw_price = false;
    let mut saw_heartbeat = false;
    tokio::time::timeout(Duration::from_secs(20), async {
        while !(saw_price && saw_heartbeat) {
            match prices.next().await.expect("pricing stream ended").unwrap() {
                PricingStreamItem::Price(_) => saw_price = true,
                PricingStreamItem::Heartbeat(_) => saw_heartbeat = true,
            }
        }
    })
    .await
    .expect("pricing stream did not provide price and heartbeat");

    let transactions = stream_client.transactions_stream().await.unwrap();
    futures_util::pin_mut!(transactions);
    tokio::time::timeout(Duration::from_secs(20), async {
        loop {
            if matches!(
                transactions
                    .next()
                    .await
                    .expect("transaction stream ended")
                    .unwrap(),
                TransactionStreamItem::Heartbeat(_)
            ) {
                break;
            }
        }
    })
    .await
    .expect("transaction stream did not provide a heartbeat");
}
