use oxidized_alpaca::{
    trading::{self, accounts::Currency},
    AccountType, RestClient,
};

#[tokio::test]
async fn test_account() {
    // Set up our RestClient
    let client = RestClient::new(AccountType::Paper).unwrap();
    // Get the account
    let account = trading::accounts::get(&client).await.unwrap();
    assert_eq!(account.currency, Currency::USD);
    println!("{account:#?}");
}
