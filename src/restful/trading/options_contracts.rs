use crate::restful::{TradingClient, string_as_decimal, string_as_optional_decimal};
use chrono::NaiveDate;
use reqwest::Method;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// Option contract type.
#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
#[non_exhaustive]
pub enum OptionType {
    /// Call option.
    Call,
    /// Put option.
    Put,
}

/// Option exercise style.
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
#[non_exhaustive]
pub enum OptionStyle {
    /// American-style (exercisable any time before expiry).
    American,
    /// European-style (exercisable only at expiry).
    European,
}

/// Option contract status.
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum ContractStatus {
    /// Contract is active and tradable.
    Active,
    /// Contract is inactive.
    Inactive,
}

/// An options contract as returned by the Alpaca API.
#[derive(Clone, Debug, Deserialize)]
pub struct OptionContract {
    /// Contract ID.
    pub id: String,
    /// OCC symbol.
    pub symbol: String,
    /// Human-readable contract name.
    pub name: String,
    /// Active or inactive status.
    pub status: ContractStatus,
    /// Whether the contract is tradable.
    pub tradable: bool,
    /// Option chain ID.
    pub chain_id: Option<String>,
    /// Option chain symbol.
    pub chain_symbol: Option<String>,
    /// Contract expiration date.
    pub expiration_date: NaiveDate,
    /// Root symbol of the option.
    pub root_symbol: Option<String>,
    /// Underlying asset symbol.
    pub underlying_symbol: String,
    /// Underlying asset ID.
    pub underlying_asset_id: Option<String>,
    /// Call or put type.
    #[serde(rename = "type")]
    pub option_type: OptionType,
    /// Exercise style (American or European).
    pub style: OptionStyle,
    /// Strike price of the contract.
    #[serde(deserialize_with = "string_as_decimal")]
    pub strike_price: Decimal,
    /// Contract size (typically "100").
    pub size: Option<String>,
    /// Open interest.
    pub open_interest: Option<String>,
    /// Last close price.
    #[serde(
        default,
        deserialize_with = "string_as_optional_decimal",
        skip_serializing_if = "Option::is_none"
    )]
    pub close_price: Option<Decimal>,
    /// Date of the close price.
    #[serde(default)]
    pub close_price_date: Option<NaiveDate>,
}

/// Response wrapper for paginated option contract listings.
#[derive(Debug, Deserialize)]
struct OptionContractsResponse {
    option_contracts: Vec<OptionContract>,
    #[serde(default)]
    #[allow(dead_code)]
    next_page_token: Option<String>,
}

/// Builder for listing option contracts.
#[derive(Debug, Serialize)]
#[must_use]
pub struct ListOptionContractsRequest<'a> {
    #[serde(skip)]
    client: &'a TradingClient,
    #[serde(skip_serializing_if = "Option::is_none")]
    underlying_symbols: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    status: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    expiration_date: Option<NaiveDate>,
    #[serde(skip_serializing_if = "Option::is_none")]
    expiration_date_gte: Option<NaiveDate>,
    #[serde(skip_serializing_if = "Option::is_none")]
    expiration_date_lte: Option<NaiveDate>,
    #[serde(skip_serializing_if = "Option::is_none")]
    root_symbol: Option<String>,
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    option_type: Option<OptionType>,
    #[serde(skip_serializing_if = "Option::is_none")]
    style: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    strike_price_gte: Option<Decimal>,
    #[serde(skip_serializing_if = "Option::is_none")]
    strike_price_lte: Option<Decimal>,
    #[serde(skip_serializing_if = "Option::is_none")]
    limit: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    page_token: Option<String>,
}

impl ListOptionContractsRequest<'_> {
    /// Filter by underlying symbol(s), comma-separated.
    pub fn underlying_symbols(mut self, symbols: &str) -> Self {
        self.underlying_symbols = Some(symbols.to_string());
        self
    }

    /// Filter by status ("active" or "inactive").
    pub fn status(mut self, status: &str) -> Self {
        self.status = Some(status.to_string());
        self
    }

    /// Filter by exact expiration date.
    pub fn expiration_date(mut self, date: NaiveDate) -> Self {
        self.expiration_date = Some(date);
        self
    }

    /// Filter by minimum expiration date.
    pub fn expiration_date_gte(mut self, date: NaiveDate) -> Self {
        self.expiration_date_gte = Some(date);
        self
    }

    /// Filter by maximum expiration date.
    pub fn expiration_date_lte(mut self, date: NaiveDate) -> Self {
        self.expiration_date_lte = Some(date);
        self
    }

    /// Filter by root symbol.
    pub fn root_symbol(mut self, symbol: &str) -> Self {
        self.root_symbol = Some(symbol.to_string());
        self
    }

    /// Filter by option type (call or put).
    pub fn option_type(mut self, option_type: OptionType) -> Self {
        self.option_type = Some(option_type);
        self
    }

    /// Filter by minimum strike price.
    pub fn strike_price_gte(mut self, price: Decimal) -> Self {
        self.strike_price_gte = Some(price);
        self
    }

    /// Filter by maximum strike price.
    pub fn strike_price_lte(mut self, price: Decimal) -> Self {
        self.strike_price_lte = Some(price);
        self
    }

    /// Maximum number of results per page.
    pub fn limit(mut self, limit: u32) -> Self {
        self.limit = Some(limit);
        self
    }

    /// Execute the request (single page).
    pub async fn execute(self) -> crate::Result<Vec<OptionContract>> {
        let request = self
            .client
            .request(Method::GET, "options/contracts")
            .query(&self);
        let response: OptionContractsResponse = self.client.send_and_deserialize(request).await?;
        Ok(response.option_contracts)
    }
}

impl TradingClient {
    /// List option contracts with filters.
    ///
    /// ```ignore
    /// use rust_decimal_macros::dec;
    ///
    /// let contracts = client.list_option_contracts()
    ///     .underlying_symbols("AAPL")
    ///     .option_type(OptionType::Call)
    ///     .strike_price_gte(dec!(150))
    ///     .execute().await?;
    /// ```
    pub fn list_option_contracts(&self) -> ListOptionContractsRequest<'_> {
        ListOptionContractsRequest {
            client: self,
            underlying_symbols: None,
            status: None,
            expiration_date: None,
            expiration_date_gte: None,
            expiration_date_lte: None,
            root_symbol: None,
            option_type: None,
            style: None,
            strike_price_gte: None,
            strike_price_lte: None,
            limit: None,
            page_token: None,
        }
    }

    /// Get a specific option contract by symbol or ID.
    pub async fn get_option_contract(&self, symbol_or_id: &str) -> crate::Result<OptionContract> {
        let request = self.request(Method::GET, &format!("options/contracts/{symbol_or_id}"));
        self.send_and_deserialize(request).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_option_contract_deserialization() {
        let json = r#"{
            "id": "a1b2c3d4",
            "symbol": "AAPL250117C00150000",
            "name": "AAPL Jan 17 2025 150 Call",
            "status": "active",
            "tradable": true,
            "chain_id": null,
            "chain_symbol": null,
            "expiration_date": "2025-01-17",
            "root_symbol": "AAPL",
            "underlying_symbol": "AAPL",
            "underlying_asset_id": "904837e3-3b76-47ec-b432-046db621571b",
            "type": "call",
            "style": "american",
            "strike_price": "150.00",
            "size": "100",
            "open_interest": "5000",
            "close_price": "5.25",
            "close_price_date": "2024-12-30"
        }"#;
        let contract: OptionContract = serde_json::from_str(json).unwrap();
        assert_eq!(contract.option_type, OptionType::Call);
        assert_eq!(
            contract.strike_price,
            Decimal::from_str_exact("150.00").unwrap()
        );
    }
}
