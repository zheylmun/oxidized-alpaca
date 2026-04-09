use crate::restful::{TradingClient, string_as_optional_decimal};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// `AssetClass` represents the category to which the asset belongs to.
/// It serves to identify the nature of the financial instrument
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum AssetClass {
    UsEquity,
    UsOption,
    Crypto,
    CryptoPerp,
}

/// `Exchange` represents the exchange where the asset is traded
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[non_exhaustive]
pub enum Exchange {
    /// American Stock Exchange
    Amex,
    /// Archipelago Exchange
    Arca,
    /// Amsterdam Small Cap Index Exchange
    Ascx,
    /// BATS (Better Alternative Trading System) Exchange
    Bats,
    /// New York Stock Exchange
    Nyse,
    /// NASDAQ Exchange
    Nasdaq,
    /// NYSE Arca
    Nysearca,
    Otc,
    Crypto,
}

/// `Status` represents whether an asset is active or inactive.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum Status {
    Active,
    Inactive,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct Asset {
    /// Asset ID
    pub id: String,
    /// Asset class
    pub class: AssetClass,
    /// Exchange the asset is traded on
    pub exchange: Exchange,
    pub symbol: String,
    pub name: String,
    pub status: Status,
    pub tradable: bool,
    pub marginable: bool,
    pub shortable: bool,
    pub easy_to_borrow: bool,
    pub fractionable: bool,
    #[serde(deserialize_with = "string_as_optional_decimal", default)]
    pub margin_requirement_long: Option<Decimal>,
    #[serde(deserialize_with = "string_as_optional_decimal", default)]
    pub margin_requirement_short: Option<Decimal>,
    pub attributes: Vec<String>,
}

/// Builder for filtering asset list requests.
#[derive(Clone, Debug, Serialize)]
#[must_use]
pub struct AssetRequest<'a> {
    #[serde(skip)]
    client: &'a TradingClient,
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
    pub fn status(mut self, status: Status) -> Self {
        self.status = Some(status);
        self
    }
    pub fn asset_class(mut self, asset_class: AssetClass) -> Self {
        self.asset_class = Some(asset_class);
        self
    }
    pub fn exchange(mut self, exchange: Exchange) -> Self {
        self.exchange = Some(exchange);
        self
    }
    pub fn attributes(mut self, attributes: String) -> Self {
        self.attributes = Some(attributes);
        self
    }

    /// Execute the request and return matching assets.
    pub async fn execute(self) -> crate::Result<Vec<Asset>> {
        let request = self
            .client
            .request(reqwest::Method::GET, "assets")
            .query(&self);
        self.client.send_and_deserialize(request).await
    }
}

impl TradingClient {
    /// List assets with optional filters.
    ///
    /// ```ignore
    /// let assets = client.list_assets()
    ///     .status(Status::Active)
    ///     .asset_class(AssetClass::UsEquity)
    ///     .execute().await?;
    /// ```
    pub fn list_assets(&self) -> AssetRequest<'_> {
        AssetRequest {
            client: self,
            status: None,
            asset_class: None,
            exchange: None,
            attributes: None,
        }
    }

    /// Get a specific asset by symbol or asset ID.
    pub async fn get_asset(&self, symbol_or_id: &str) -> crate::Result<Asset> {
        let request = self.request(reqwest::Method::GET, &format!("assets/{symbol_or_id}"));
        self.send_and_deserialize(request).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_asset_deserialization() {
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
        let asset: Asset = serde_json::from_str(sample).unwrap();
        assert_eq!(
            asset.margin_requirement_long,
            Some(Decimal::from_str_exact("100").unwrap())
        );
    }
}
