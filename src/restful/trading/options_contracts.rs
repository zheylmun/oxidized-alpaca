use crate::OptionContractId;
use crate::restful::{
    TradingClient, string_as_decimal, string_as_optional_decimal, string_as_optional_u64,
};
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
#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
#[non_exhaustive]
pub enum OptionStyle {
    /// American-style (exercisable any time before expiry).
    American,
    /// European-style (exercisable only at expiry).
    European,
}

/// Option contract status.
#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum ContractStatus {
    /// Contract is active and tradable.
    Active,
    /// Contract is inactive.
    Inactive,
}

/// A deliverable underlying an option contract.
#[derive(Clone, Debug, Deserialize)]
#[non_exhaustive]
pub struct OptionDeliverable {
    /// Deliverable type (e.g. `equity`, `cash`).
    #[serde(rename = "type")]
    pub deliverable_type: String,
    /// Underlying symbol of the deliverable.
    pub symbol: String,
    /// Asset ID of the deliverable, when applicable.
    #[serde(default)]
    pub asset_id: Option<String>,
    /// Amount of the deliverable per contract.
    #[serde(deserialize_with = "string_as_decimal")]
    pub amount: Decimal,
    /// Percentage of the deliverable allocated.
    #[serde(deserialize_with = "string_as_decimal")]
    pub allocation_percentage: Decimal,
    /// Settlement type (e.g. `T+0`, `T+1`).
    pub settlement_type: String,
    /// Settlement method.
    pub settlement_method: String,
    /// Whether settlement is delayed.
    pub delayed_settlement: bool,
}

/// An options contract as returned by the Alpaca API.
#[derive(Clone, Debug, Deserialize)]
#[non_exhaustive]
pub struct OptionContract {
    /// Contract ID.
    pub id: OptionContractId,
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
    /// Contract multiplier (typically 100).
    #[serde(deserialize_with = "string_as_decimal")]
    pub multiplier: Decimal,
    /// Contract size (typically 100 — number of underlying shares per contract).
    #[serde(default, deserialize_with = "string_as_optional_decimal")]
    pub size: Option<Decimal>,
    /// Open interest.
    #[serde(default, deserialize_with = "string_as_optional_u64")]
    pub open_interest: Option<u64>,
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
    /// Date of the open-interest figure.
    #[serde(default)]
    pub open_interest_date: Option<NaiveDate>,
    /// Deliverables underlying the contract (included when requested with
    /// `show_deliverables`).
    #[serde(default)]
    pub deliverables: Option<Vec<OptionDeliverable>>,
}

/// Response wrapper for paginated option contract listings.
#[derive(Debug, Deserialize)]
struct OptionContractsResponse {
    option_contracts: Vec<OptionContract>,
    #[serde(default)]
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
    status: Option<ContractStatus>,
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
    style: Option<OptionStyle>,
    #[serde(skip_serializing_if = "Option::is_none")]
    strike_price_gte: Option<Decimal>,
    #[serde(skip_serializing_if = "Option::is_none")]
    strike_price_lte: Option<Decimal>,
    #[serde(skip_serializing_if = "Option::is_none")]
    limit: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    page_token: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    show_deliverables: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    ppind: Option<bool>,
}

impl ListOptionContractsRequest<'_> {
    /// Filter by underlying symbols.
    pub fn underlying_symbols(mut self, symbols: &[&str]) -> Self {
        self.underlying_symbols = Some(symbols.join(","));
        self
    }

    /// Filter by contract status (active or inactive).
    pub fn status(mut self, status: ContractStatus) -> Self {
        self.status = Some(status);
        self
    }

    /// Filter by exercise style (American or European).
    pub fn style(mut self, style: OptionStyle) -> Self {
        self.style = Some(style);
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

    /// Cap the total number of contracts returned across all auto-paginated pages.
    pub fn limit(mut self, limit: usize) -> Self {
        self.limit = Some(limit);
        self
    }

    /// Include the `deliverables` array on each returned contract.
    pub fn show_deliverables(mut self, show: bool) -> Self {
        self.show_deliverables = Some(show);
        self
    }

    /// Filter by penny-program eligibility (Penny Program Indicator).
    pub fn ppind(mut self, ppind: bool) -> Self {
        self.ppind = Some(ppind);
        self
    }

    /// Execute the request, auto-paginating until all matching contracts are
    /// retrieved or the configured `limit` is reached.
    pub async fn execute(mut self) -> crate::Result<Vec<OptionContract>> {
        let cap = self.limit;
        let mut all = Vec::new();
        loop {
            let request = self
                .client
                .request(Method::GET, "v2/options/contracts")?
                .query(&self);
            let response: OptionContractsResponse =
                self.client.send_and_deserialize(request).await?;
            all.extend(response.option_contracts);
            if let Some(cap) = cap
                && all.len() >= cap
            {
                all.truncate(cap);
                break;
            }
            match response.next_page_token {
                Some(token) => self.page_token = Some(token),
                None => break,
            }
        }
        Ok(all)
    }
}

impl TradingClient {
    /// List option contracts with filters.
    ///
    /// ```ignore
    /// use rust_decimal_macros::dec;
    ///
    /// let contracts = client.list_option_contracts()
    ///     .underlying_symbols(&["AAPL"])
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
            show_deliverables: None,
            ppind: None,
        }
    }

    /// Get a specific option contract by symbol or ID.
    pub async fn get_option_contract(&self, symbol_or_id: &str) -> crate::Result<OptionContract> {
        let request = self.request(Method::GET, &format!("v2/options/contracts/{symbol_or_id}"))?;
        self.send_and_deserialize(request).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::AccountType;
    use serial_test::serial;
    use std::env;

    fn paper_client() -> TradingClient {
        unsafe {
            if env::var("ALPACA_PAPER_API_KEY_ID").is_err() {
                env::set_var("ALPACA_PAPER_API_KEY_ID", "test_key_id");
            }
            if env::var("ALPACA_PAPER_API_SECRET_KEY").is_err() {
                env::set_var("ALPACA_PAPER_API_SECRET_KEY", "test_secret_key");
            }
        }
        TradingClient::new(AccountType::Paper).unwrap()
    }

    #[test]
    #[serial]
    fn show_deliverables_and_ppind_serialize_to_query() {
        let client = paper_client();
        let request = client
            .list_option_contracts()
            .show_deliverables(true)
            .ppind(false);
        let query = serde_urlencoded::to_string(&request).unwrap();
        assert!(
            query.contains("show_deliverables=true"),
            "expected show_deliverables in {query}"
        );
        assert!(query.contains("ppind=false"), "expected ppind in {query}");
    }

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
            "multiplier": "100",
            "size": "100",
            "open_interest": "5000",
            "open_interest_date": "2024-12-30",
            "close_price": "5.25",
            "close_price_date": "2024-12-30",
            "deliverables": [
                {
                    "type": "equity",
                    "symbol": "AAPL",
                    "asset_id": "904837e3-3b76-47ec-b432-046db621571b",
                    "amount": "100",
                    "allocation_percentage": "100",
                    "settlement_type": "T+0",
                    "settlement_method": "BTOB",
                    "delayed_settlement": false
                }
            ]
        }"#;
        let contract: OptionContract = serde_json::from_str(json).unwrap();
        assert_eq!(contract.option_type, OptionType::Call);
        assert_eq!(
            contract.strike_price,
            Decimal::from_str_exact("150.00").unwrap()
        );
        assert_eq!(contract.multiplier, Decimal::from_str_exact("100").unwrap());
        assert_eq!(
            contract.open_interest_date,
            Some(chrono::NaiveDate::from_ymd_opt(2024, 12, 30).unwrap())
        );
        let deliverables = contract.deliverables.as_ref().unwrap();
        assert_eq!(deliverables.len(), 1);
        assert_eq!(deliverables[0].symbol, "AAPL");
        assert_eq!(
            deliverables[0].amount,
            Decimal::from_str_exact("100").unwrap()
        );
        assert!(!deliverables[0].delayed_settlement);
    }
}
