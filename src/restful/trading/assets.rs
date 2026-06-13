use crate::AssetId;
use crate::restful::{TradingClient, null_def_vec, string_as_optional_decimal};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

pub use crate::asset::AssetClass;

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
    /// Over-the-counter markets.
    Otc,
    /// Cryptocurrency exchange.
    Crypto,
}

/// `Status` represents whether an asset is active or inactive.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum Status {
    /// Asset is active and tradable.
    Active,
    /// Asset is inactive.
    Inactive,
}

/// Optional attribute flag attached to an [`Asset`]. Unknown values from
/// Alpaca are preserved verbatim under [`AssetAttribute::Other`].
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum AssetAttribute {
    /// PTP no-exception trading enabled.
    PtpNoException,
    /// PTP with-exception trading enabled.
    PtpWithException,
    /// Asset is in IPO state.
    Ipo,
    /// Underlying has tradable options.
    HasOptions,
    /// Options late-close window is supported.
    OptionsLateClose,
    /// Fractional shares supported in extended hours.
    FractionalEhEnabled,
    /// Any attribute not modeled above; the raw string from the API.
    Other(String),
}

impl AssetAttribute {
    /// Wire string for this attribute. Used to encode the
    /// `?attributes=` query parameter.
    #[must_use]
    pub fn as_str(&self) -> &str {
        match self {
            Self::PtpNoException => "ptp_no_exception",
            Self::PtpWithException => "ptp_with_exception",
            Self::Ipo => "ipo",
            Self::HasOptions => "has_options",
            Self::OptionsLateClose => "options_late_close",
            Self::FractionalEhEnabled => "fractional_eh_enabled",
            Self::Other(raw) => raw,
        }
    }
}

impl serde::Serialize for AssetAttribute {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> serde::Deserialize<'de> for AssetAttribute {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let raw = String::deserialize(deserializer)?;
        Ok(match raw.as_str() {
            "ptp_no_exception" => Self::PtpNoException,
            "ptp_with_exception" => Self::PtpWithException,
            "ipo" => Self::Ipo,
            "has_options" => Self::HasOptions,
            "options_late_close" => Self::OptionsLateClose,
            "fractional_eh_enabled" => Self::FractionalEhEnabled,
            _ => Self::Other(raw),
        })
    }
}

impl std::fmt::Display for AssetAttribute {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Borrowing status for a US equity asset.
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum BorrowStatus {
    /// The asset is easy to borrow for shorting.
    EasyToBorrow,
    /// The asset is hard to borrow for shorting.
    HardToBorrow,
}

/// An asset as returned by the Alpaca API.
#[derive(Clone, Debug, Deserialize, PartialEq)]
#[non_exhaustive]
pub struct Asset {
    /// Asset ID.
    pub id: AssetId,
    /// Asset class.
    pub class: AssetClass,
    /// Exchange the asset is traded on.
    pub exchange: Exchange,
    /// Ticker symbol.
    pub symbol: String,
    /// Asset name.
    pub name: String,
    /// Active or inactive status.
    pub status: Status,
    /// Whether the asset is tradable.
    pub tradable: bool,
    /// Whether the asset is marginable.
    pub marginable: bool,
    /// Whether the asset is shortable.
    pub shortable: bool,
    /// Whether the asset is easy to borrow for shorting.
    pub easy_to_borrow: bool,
    /// Borrowing status (easy or hard to borrow); supersedes the
    /// `easy_to_borrow` flag for US equities.
    #[serde(default)]
    pub borrow_status: Option<BorrowStatus>,
    /// Whether the asset supports fractional shares.
    pub fractionable: bool,
    /// The CUSIP of the asset (US equities).
    #[serde(default)]
    pub cusip: Option<String>,
    /// Maintenance margin requirement percentage (deprecated in favor of
    /// `margin_requirement_long` / `margin_requirement_short`).
    #[serde(default)]
    pub maintenance_margin_requirement: Option<f64>,
    /// Long margin requirement percentage.
    #[serde(deserialize_with = "string_as_optional_decimal", default)]
    pub margin_requirement_long: Option<Decimal>,
    /// Short margin requirement percentage.
    #[serde(deserialize_with = "string_as_optional_decimal", default)]
    pub margin_requirement_short: Option<Decimal>,
    /// Additional asset attributes.
    #[serde(default, deserialize_with = "null_def_vec")]
    pub attributes: Vec<AssetAttribute>,
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
    /// Filter by asset status.
    pub fn status(mut self, status: Status) -> Self {
        self.status = Some(status);
        self
    }
    /// Filter by asset class.
    pub fn asset_class(mut self, asset_class: AssetClass) -> Self {
        self.asset_class = Some(asset_class);
        self
    }
    /// Filter by exchange.
    pub fn exchange(mut self, exchange: Exchange) -> Self {
        self.exchange = Some(exchange);
        self
    }
    /// Filter by asset attributes. Multiple values are sent as a
    /// comma-joined `attributes=` query parameter.
    pub fn attributes(mut self, attributes: &[AssetAttribute]) -> Self {
        let joined = attributes
            .iter()
            .map(AssetAttribute::as_str)
            .collect::<Vec<_>>()
            .join(",");
        self.attributes = if joined.is_empty() {
            None
        } else {
            Some(joined)
        };
        self
    }

    /// Execute the request and return matching assets.
    pub async fn execute(self) -> crate::Result<Vec<Asset>> {
        let request = self
            .client
            .request(reqwest::Method::GET, "v2/assets")?
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
        let request = self.request(reqwest::Method::GET, &format!("v2/assets/{symbol_or_id}"))?;
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
            "cusip": "464287655",
            "borrow_status": "hard_to_borrow",
            "attributes": []
            }"#;
        let asset: Asset = serde_json::from_str(sample).unwrap();
        assert_eq!(
            asset.margin_requirement_long,
            Some(Decimal::from_str_exact("100").unwrap())
        );
        assert_eq!(asset.maintenance_margin_requirement, Some(100.0));
        assert_eq!(asset.cusip.as_deref(), Some("464287655"));
        assert_eq!(asset.borrow_status, Some(BorrowStatus::HardToBorrow));
    }

    #[test]
    fn asset_attribute_known_codes_round_trip() {
        let cases = [
            (AssetAttribute::PtpNoException, "\"ptp_no_exception\""),
            (AssetAttribute::PtpWithException, "\"ptp_with_exception\""),
            (AssetAttribute::Ipo, "\"ipo\""),
            (AssetAttribute::HasOptions, "\"has_options\""),
            (AssetAttribute::OptionsLateClose, "\"options_late_close\""),
            (
                AssetAttribute::FractionalEhEnabled,
                "\"fractional_eh_enabled\"",
            ),
        ];
        for (variant, expected) in cases {
            assert_eq!(serde_json::to_string(&variant).unwrap(), expected);
            let parsed: AssetAttribute = serde_json::from_str(expected).unwrap();
            assert_eq!(parsed, variant);
        }
    }

    #[test]
    fn asset_attribute_unknown_falls_back_to_other() {
        let parsed: AssetAttribute = serde_json::from_str("\"future_flag\"").unwrap();
        assert_eq!(parsed, AssetAttribute::Other("future_flag".to_string()));
        assert_eq!(serde_json::to_string(&parsed).unwrap(), "\"future_flag\"");
    }

    #[test]
    fn asset_with_typed_attributes_deserializes() {
        let sample = r#"{
            "id": "id-1",
            "class": "us_equity",
            "exchange": "NASDAQ",
            "symbol": "AAPL",
            "name": "Apple Inc.",
            "status": "active",
            "tradable": true,
            "marginable": true,
            "shortable": true,
            "easy_to_borrow": true,
            "fractionable": true,
            "attributes": ["has_options", "fractional_eh_enabled", "future_flag"]
        }"#;
        let asset: Asset = serde_json::from_str(sample).unwrap();
        assert_eq!(
            asset.attributes,
            vec![
                AssetAttribute::HasOptions,
                AssetAttribute::FractionalEhEnabled,
                AssetAttribute::Other("future_flag".to_string()),
            ]
        );
    }
}
