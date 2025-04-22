use oxidized_alpaca::{
    AccountType,
    restful::{
        RestClient,
        trading::{
            accounts::{AccountDetails, Currency},
            assets::{Asset, AssetClass, Exchange, Status},
        },
    },
};

#[tokio::test]
async fn test_account() {
    // Set up our RestClient
    let client = RestClient::new(AccountType::Paper).unwrap();
    // Get the account
    let account = AccountDetails::get(&client).await.unwrap();
    assert_eq!(account.currency, Currency::USD);
}

#[tokio::test]
async fn trading_sequence() {
    // Set up our RestClient
    let client = RestClient::new(AccountType::Paper).unwrap();

    // Get all assets
    let assets = Asset::get(&client).execute().await.unwrap();
    assert!(!assets.is_empty());

    // Get all active assets
    let assets = Asset::get(&client)
        .with_status(Status::Active)
        .execute()
        .await
        .unwrap();
    assert!(!assets.is_empty());

    // Get all US equity assets
    let assets = Asset::get(&client)
        .with_asset_class(AssetClass::UsEquity)
        .execute()
        .await
        .unwrap();
    assert!(!assets.is_empty());

    // Get all OTC assets
    let assets = Asset::get(&client)
        .with_exchange(Exchange::Otc)
        .execute()
        .await
        .unwrap();
    assert!(!assets.is_empty());

    let assets = Asset::get(&client)
        .with_attribute_string("ptp_no_exception".to_string())
        .execute()
        .await
        .unwrap();
    assert!(!assets.is_empty())
}
