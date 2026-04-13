use crate::restful::MarketDataClient;
use chrono::{DateTime, Utc};
use reqwest::Method;
use serde::Deserialize;

/// A forex rate.
#[derive(Clone, Debug, Deserialize)]
pub struct ForexRate {
    /// The bid price.
    #[serde(rename = "bp")]
    pub bid_price: f64,
    /// The ask price.
    #[serde(rename = "ap")]
    pub ask_price: f64,
    /// The mid price.
    #[serde(rename = "mp")]
    pub mid_price: f64,
    /// The rate timestamp.
    #[serde(rename = "t")]
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
struct RatesResponse {
    rates: std::collections::HashMap<String, ForexRate>,
}

impl MarketDataClient {
    /// Get the latest forex rates.
    pub async fn forex_latest_rates(
        &self,
        currency_pairs: &str,
    ) -> crate::Result<std::collections::HashMap<String, ForexRate>> {
        let request = self
            .request(Method::GET, "v1beta1/forex/latest/rates")
            .query(&[("currency_pairs", currency_pairs)]);
        let response: RatesResponse = self.send_and_deserialize(request).await?;
        Ok(response.rates)
    }
}
