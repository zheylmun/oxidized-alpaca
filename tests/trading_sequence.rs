use oxidized_alpaca::{
    AccountType, TradingClient,
    restful::trading::{
        accounts::Currency,
        assets::{AssetAttribute, AssetClass, Exchange, Status},
    },
};

#[tokio::test]
async fn test_account() {
    let client = TradingClient::new(AccountType::Paper).unwrap();
    let account = client.get_account().await.unwrap();
    assert_eq!(account.currency, Currency::USD);
}

#[tokio::test]
async fn trading_sequence() {
    let client = TradingClient::new(AccountType::Paper).unwrap();

    let assets = client.list_assets().execute().await.unwrap();
    assert!(!assets.is_empty());

    let assets = client
        .list_assets()
        .status(Status::Active)
        .execute()
        .await
        .unwrap();
    assert!(!assets.is_empty());

    let assets = client
        .list_assets()
        .asset_class(AssetClass::UsEquity)
        .execute()
        .await
        .unwrap();
    assert!(!assets.is_empty());

    let assets = client
        .list_assets()
        .exchange(Exchange::Otc)
        .execute()
        .await
        .unwrap();
    assert!(!assets.is_empty());

    let assets = client
        .list_assets()
        .attributes(&[AssetAttribute::PtpNoException])
        .execute()
        .await
        .unwrap();
    assert!(!assets.is_empty())
}
