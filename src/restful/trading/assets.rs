use crate::{
    Error,
    restful::{RestClient, rest_client::RequestAPI, string_as_optional_f64},
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
    /// American Stock Exchang
    Amex,
    ///
    Arca,
    /// BATS (Better Alternative Trading System) Exchange
    Bats,
    /// New York Stock Exchange
    Nyse,
    /// NASDAQ (National Association of Securities Dealers Automated Quotations) Exchange
    Nasdaq,
    /// NYSE Archa (Archipelago Exchange)
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

impl Asset {
    pub fn get(client: &RestClient) -> AssetRequest<'_> {
        AssetRequest {
            client,
            status: None,
            asset_class: None,
            exchange: None,
            attributes: None,
        }
    }

    pub async fn get_by_id(client: &RestClient, id: &str) -> Result<Asset, Error> {
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

impl AssetRequest<'_> {
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

#[cfg(test)]
mod tests {
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
}
