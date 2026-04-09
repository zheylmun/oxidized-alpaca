use oxidized_alpaca::{AccountType, TradingClient, restful::trading::accounts::Currency};

#[tokio::test]
async fn test_account() {
    let client = TradingClient::new(AccountType::Paper).unwrap();
    let account = client.get_account().await.unwrap();
    assert_eq!(account.currency, Currency::USD);
    println!("{account:#?}");
}
