use crate::{
    restful::{rest_client::RequestAPI, string_as_optional_f64},
    Error, RestClient,
};
use futures::TryFutureExt;
use serde::{Deserialize, Serialize};

/// `AssetClass` represents the category to which the asset belongs to.
/// It serves to identify the nature of the financial instrument
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AssetClass {
    UsEquity,
    UsOption,
    Crypto,
}

/// `Exchange` represents the exchange where the asset is traded
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Exchange {
    Amex,
    Arca,
    Bats,
    Nyse,
    Nasdaq,
    Nysearca,
    Otc,
    Crypto,
}

/// `Status` represents whether an asset is active or inactive.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Status {
    Active,
    Inactive,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct Asset {
    /// Asset ID
    id: String,
    /// Asset class
    class: AssetClass,
    /// Exchange the asset is traded on
    exchange: Exchange,
    symbol: String,
    name: String,
    status: Status,
    tradable: bool,
    marginable: bool,
    shortable: bool,
    easy_to_borrow: bool,
    fractionable: bool,
    #[serde(deserialize_with = "string_as_optional_f64", default)]
    margin_requirement_long: Option<f64>,
    #[serde(deserialize_with = "string_as_optional_f64", default)]
    margin_requirement_short: Option<f64>,
    attributes: Vec<String>,
}

#[derive(Clone, Debug, Serialize)]
pub struct AssetRequest<'a> {
    #[serde(skip)]
    client: &'a RestClient,
    #[serde(skip_serializing_if = "Option::is_none")]
    status: Option<Status>,
    #[serde(skip_serializing_if = "Option::is_none")]
    asset_class: Option<AssetClass>,
    #[serde(skip_serializing_if = "Option::is_none")]
    exchange: Option<Exchange>,
    #[serde(skip_serializing_if = "Option::is_none")]
    attributes: Option<String>,
}

impl<'a> AssetRequest<'a> {
    pub fn with_status(mut self, status: Status) -> Self {
        self.status = Some(status);
        self
    }
    pub fn with_asset_class(mut self, asset_class: AssetClass) -> Self {
        self.asset_class = Some(asset_class);
        self
    }
    pub fn with_exchange(mut self, exchange: Exchange) -> Self {
        self.exchange = Some(exchange);
        self
    }
    pub fn with_attribute_string(mut self, attributes: String) -> Self {
        self.attributes = Some(attributes);
        self
    }

    pub async fn execute(self) -> Result<Vec<Asset>, reqwest::Error> {
        let response = self
            .client
            .request(reqwest::Method::GET, RequestAPI::Trading, "assets")
            .send()
            .await?;
        response.json::<Vec<Asset>>().await
    }
}

pub fn get(client: &RestClient) -> AssetRequest {
    AssetRequest {
        client,
        status: None,
        asset_class: None,
        exchange: None,
        attributes: None,
    }
}

pub async fn get_by_id<'a>(client: &RestClient, id: &str) -> Result<Asset, Error> {
    let response = client
        .request(
            reqwest::Method::GET,
            RequestAPI::Trading,
            &format!("assets/{}", id),
        )
        .send()
        .await
        .map_err(Error::ReqwestSend)?;
    response
        .json::<Asset>()
        .map_err(Error::ReqwestDeserialize)
        .await
}

#[cfg(test)]
mod tests {
    use crate::AccountType;

    use super::*;

    #[tokio::test]
    async fn test_asset_deserilization() {
        let sample = r#"{
            "id": "931bb17d-f64d-4344-a7c8-af552886c3ff",
            "class": "us_equity",
            "exchange": "OTC",
            "symbol": "ISHHF",
            "name": "Ishares Plc Shs Exchange Traded Fund Eur (Ireland)",
            "status": "inactive",
            "tradable": false,
            "marginable": false,
            "maintenance_margin_requirement": 100,
            "margin_requirement_long": "100",
            "margin_requirement_short": "100",
            "shortable": false,
            "easy_to_borrow": false,
            "fractionable": false,
            "attributes": []
            }"#;
        let _asset: Asset = serde_json::from_str(sample).unwrap();
    }

    #[tokio::test]
    async fn test_asset_request() {
        let client = RestClient::new(AccountType::Paper).unwrap();
        let assets = get(&client).execute().await.unwrap();
        assert!(!assets.is_empty());
    }

    #[tokio::test]
    async fn test_asset_request_with_status() {
        let client = RestClient::new(AccountType::Paper).unwrap();
        let assets = get(&client)
            .with_status(Status::Active)
            .execute()
            .await
            .unwrap();
        assert!(!assets.is_empty());
    }

    #[tokio::test]
    async fn test_asset_request_with_asset_class() {
        let client = RestClient::new(AccountType::Paper).unwrap();
        let assets = get(&client)
            .with_asset_class(AssetClass::UsEquity)
            .execute()
            .await
            .unwrap();
        assert!(!assets.is_empty());
    }

    #[tokio::test]
    async fn test_asset_request_with_exchange() {
        let client = RestClient::new(AccountType::Paper).unwrap();
        let assets = get(&client)
            .with_exchange(Exchange::Otc)
            .execute()
            .await
            .unwrap();
        assert!(!assets.is_empty());
    }
    #[tokio::test]
    async fn test_asset_request_with_attributes() {
        let client = RestClient::new(AccountType::Paper).unwrap();
        let assets = get(&client)
            .with_attribute_string("ptp_no_exception".to_string())
            .execute()
            .await
            .unwrap();
        assert!(!assets.is_empty());
    }
}
