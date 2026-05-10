use crate::restful::MarketDataClient;
use reqwest::Method;
use serde::Deserialize;

/// A fixed income price.
#[derive(Clone, Debug, Deserialize)]
#[non_exhaustive]
pub struct FixedIncomePrice {
    /// The instrument symbol.
    pub symbol: String,
    /// The price.
    pub price: f64,
    /// The price timestamp.
    #[serde(default)]
    pub timestamp: Option<String>,
}

#[derive(Debug, Deserialize)]
struct PricesResponse {
    prices: std::collections::HashMap<String, FixedIncomePrice>,
}

impl MarketDataClient {
    /// Get the latest fixed income prices.
    pub async fn fixed_income_latest_prices(
        &self,
        symbols: &[&str],
    ) -> crate::Result<std::collections::HashMap<String, FixedIncomePrice>> {
        let request = self
            .request(Method::GET, "v1beta1/fixed_income/latest/prices")?
            .query(&[("symbols", symbols.join(","))]);
        let response: PricesResponse = self.send_and_deserialize(request).await?;
        Ok(response.prices)
    }
}
