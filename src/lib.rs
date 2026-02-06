mod client;
mod account;
mod errors;
mod primitives;
mod instrument;
mod transaction;

use crate::client::Client;

pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
