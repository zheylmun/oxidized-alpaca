use oxidized_alpaca::{
    AccountType,
    restful::{
        RestClient,
        trading::accounts::{AccountDetails, Currency},
    },
};

#[tokio::test]
async fn test_account() {
    // Set up our RestClient
    let client = RestClient::new(AccountType::Paper).unwrap();
    // Get the account
    let account = AccountDetails::get(&client).await.unwrap();
    assert_eq!(account.currency, Currency::USD);
    println!("{account:#?}");
}
