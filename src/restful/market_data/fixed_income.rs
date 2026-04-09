use crate::restful::MarketDataClient;
use reqwest::Method;
use serde::Deserialize;

/// A fixed income price.
#[derive(Clone, Debug, Deserialize)]
pub struct FixedIncomePrice {
    pub symbol: String,
    pub price: f64,
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
        symbols: &str,
    ) -> crate::Result<std::collections::HashMap<String, FixedIncomePrice>> {
        let request = self
            .request(Method::GET, "v1beta1/fixed_income/latest/prices")
            .query(&[("symbols", symbols)]);
        let response: PricesResponse = self.send_and_deserialize(request).await?;
        Ok(response.prices)
    }
}
