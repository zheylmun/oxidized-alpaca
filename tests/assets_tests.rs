use oxidized_alpaca::{
    AccountType, TradingClient,
    restful::trading::assets::{AssetClass, Exchange, Status},
};

#[tokio::test]
async fn get_all_assets() {
    let client = TradingClient::new(AccountType::Paper).unwrap();
    let assets = client.list_assets().execute().await.unwrap();
    assert!(!assets.is_empty());
}

#[tokio::test]
async fn get_all_active_assets() {
    let client = TradingClient::new(AccountType::Paper).unwrap();
    let assets = client
        .list_assets()
        .status(Status::Active)
        .execute()
        .await
        .unwrap();
    assert!(!assets.is_empty());
}

#[tokio::test]
async fn get_all_us_equity_assets() {
    let client = TradingClient::new(AccountType::Paper).unwrap();

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
        .attributes("ptp_no_exception".to_string())
        .execute()
        .await
        .unwrap();
    assert!(!assets.is_empty())
}

#[tokio::test]
async fn get_all_otc_assets() {
    let client = TradingClient::new(AccountType::Paper).unwrap();

    let assets = client
        .list_assets()
        .exchange(Exchange::Otc)
        .execute()
        .await
        .unwrap();
    assert!(!assets.is_empty());

    let assets = client
        .list_assets()
        .attributes("ptp_no_exception".to_string())
        .execute()
        .await
        .unwrap();
    assert!(!assets.is_empty())
}

#[tokio::test]
async fn get_all_ptp_no_exception_assets() {
    let client = TradingClient::new(AccountType::Paper).unwrap();

    let assets = client
        .list_assets()
        .attributes("ptp_no_exception".to_string())
        .execute()
        .await
        .unwrap();
    assert!(!assets.is_empty())
}
