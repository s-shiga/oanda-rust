mod client;
mod account;
mod errors;

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

    #[tokio::test]
    async fn list_accounts() {
        let api_key = env!("OANDA_API_KEY_DEMO");
        let client = Client::new_practice(api_key);
        let account = client.list_accounts().await;
        println!("{:?}", account);
    }
}
