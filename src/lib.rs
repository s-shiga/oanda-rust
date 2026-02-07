mod account;
mod client;
mod errors;
mod instrument;
mod pricing;
mod primitives;
mod transaction;
mod order;

pub use crate::client::Client;

pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::instrument::{CandlestickGranularity, FetchCandlestickDataRequest};
    use crate::order::ListOrdersRequest;
    use crate::transaction::{GetTransactionsByIDRangeRequest, GetTransactionsBySinceIDRequest, ListTransactionsRequest};

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }

    fn setup() -> Client {
        let api_key = env!("OANDA_API_KEY_DEMO");
        let account_id = env!("OANDA_ACCOUNT_ID_DEMO").to_string();
        Client::new_practice(api_key).with_account_id(account_id)
    }

    #[tokio::test]
    async fn test_list() {
        let client = setup();
        let account = client.list_accounts().await.unwrap();
        println!("{:#?}", account);
    }

    #[tokio::test]
    async fn test_list_instruments() {
        let client = setup();
        let resp = client.list_instruments().await.unwrap();
        println!("{:#?}", resp);
    }

    #[tokio::test]
    async fn test_fetch_candlestick_data() {
        let client = setup();
        let req = FetchCandlestickDataRequest::new("USD_JPY".to_string())
            .granularity(CandlestickGranularity::M1)
            .count(50).unwrap();
        let resp = client.fetch_candlestick_data(req).await.unwrap();
        println!("{:#?}", resp);
    }

    #[tokio::test]
    async fn test_list_orders() {
        let client = setup();
        let req = ListOrdersRequest::new().instrument(String::from("USD_JPY"));
        let resp = client.list_orders(req).await.unwrap();
        println!("{:#?}", resp);
    }

    #[tokio::test]
    async fn test_list_transactions() {
        let client = setup();
        let req = ListTransactionsRequest::new();
        let resp = client.list_transactions(req).await.unwrap();
        println!("{:#?}", resp);
    }

    #[tokio::test]
    async fn test_get_transaction_details() {
        let client = setup();
        let resp = client.get_transaction_details("456".to_string()).await.unwrap();
        println!("{:#?}", resp);
    }

    #[tokio::test]
    async fn test_get_transactions_by_id_range() {
        let client = setup();
        let req = GetTransactionsByIDRangeRequest::new("500".to_string(), "510".to_string());
        let resp = client.get_transactions_by_id_range(req).await.unwrap();
        println!("{:#?}", resp);
    }

    #[tokio::test]
    async fn test_get_transactions_by_since_id() {
        let client = setup();
        let req = GetTransactionsBySinceIDRequest::new("500".to_string());
        let resp = client.get_transactions_by_since_id(req).await.unwrap();
        println!("{:#?}", resp);
    }
}
