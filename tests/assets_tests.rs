use oxidized_alpaca::{
    trading::{
        self,
        assets::{AssetClass, Exchange, Status},
    },
    AccountType, RestClient,
};

#[tokio::test]
async fn get_all_assets() {
    // Set up our RestClient
    let client = RestClient::new(AccountType::Paper).unwrap();

    // Get all assets
    let assets = trading::assets::get(&client).execute().await.unwrap();
    assert!(!assets.is_empty());
}

#[tokio::test]
async fn get_all_active_assets() {
    // Set up our RestClient
    let client = RestClient::new(AccountType::Paper).unwrap();

    // Get all active assets
    let assets = trading::assets::get(&client)
        .with_status(Status::Active)
        .execute()
        .await
        .unwrap();
    assert!(!assets.is_empty());
}

#[tokio::test]
async fn get_all_us_equity_assets() {
    // Set up our RestClient
    let client = RestClient::new(AccountType::Paper).unwrap();

    // Get all US equity assets
    let assets = trading::assets::get(&client)
        .with_asset_class(AssetClass::UsEquity)
        .execute()
        .await
        .unwrap();
    assert!(!assets.is_empty());

    // Get all OTC assets
    let assets = trading::assets::get(&client)
        .with_exchange(Exchange::Otc)
        .execute()
        .await
        .unwrap();
    assert!(!assets.is_empty());

    let assets = trading::assets::get(&client)
        .with_attribute_string("ptp_no_exception".to_string())
        .execute()
        .await
        .unwrap();
    assert!(!assets.is_empty())
}
#[tokio::test]
async fn get_all_otc_assets() {
    // Set up our RestClient
    let client = RestClient::new(AccountType::Paper).unwrap();

    // Get all OTC assets
    let assets = trading::assets::get(&client)
        .with_exchange(Exchange::Otc)
        .execute()
        .await
        .unwrap();
    assert!(!assets.is_empty());

    let assets = trading::assets::get(&client)
        .with_attribute_string("ptp_no_exception".to_string())
        .execute()
        .await
        .unwrap();
    assert!(!assets.is_empty())
}

#[tokio::test]
async fn get_all_ptp_no_exception_assets() {
    // Set up our RestClient
    let client = RestClient::new(AccountType::Paper).unwrap();

    let assets = trading::assets::get(&client)
        .with_attribute_string("ptp_no_exception".to_string())
        .execute()
        .await
        .unwrap();
    assert!(!assets.is_empty())
}
