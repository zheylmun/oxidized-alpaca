use crate::restful::MarketDataClient;
use chrono::{DateTime, Utc};
use reqwest::Method;
use serde::Deserialize;

/// A fixed income price.
///
/// Instances are keyed by ISIN in the response map; the symbol/ISIN is the
/// map key rather than a field on this struct.
#[derive(Clone, Debug, Deserialize)]
#[non_exhaustive]
pub struct FixedIncomePrice {
    /// The price, as a percentage of par value.
    #[serde(rename = "p")]
    pub price: f64,
    /// The price timestamp.
    #[serde(rename = "t")]
    pub timestamp: DateTime<Utc>,
    /// Yield to maturity.
    #[serde(rename = "ytm", default)]
    pub yield_to_maturity: Option<f64>,
    /// Yield to worst.
    #[serde(rename = "ytw", default)]
    pub yield_to_worst: Option<f64>,
}

#[derive(Debug, Deserialize)]
struct PricesResponse {
    prices: std::collections::HashMap<String, FixedIncomePrice>,
}

impl MarketDataClient {
    /// Get the latest fixed income prices.
    pub async fn fixed_income_latest_prices(
        &self,
        isins: &[&str],
    ) -> crate::Result<std::collections::HashMap<String, FixedIncomePrice>> {
        let request = self.fixed_income_latest_prices_request(isins)?;
        let response: PricesResponse = self.send_and_deserialize(request).await?;
        Ok(response.prices)
    }

    fn fixed_income_latest_prices_request(
        &self,
        isins: &[&str],
    ) -> crate::Result<reqwest::RequestBuilder> {
        Ok(self
            .request(Method::GET, "v1beta1/fixed_income/latest/prices")?
            .query(&[("isins", isins.join(","))]))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::AccountType;
    use serial_test::serial;
    use std::env;

    fn paper_client() -> MarketDataClient {
        unsafe {
            if env::var("ALPACA_PAPER_API_KEY_ID").is_err() {
                env::set_var("ALPACA_PAPER_API_KEY_ID", "test_key_id");
            }
            if env::var("ALPACA_PAPER_API_SECRET_KEY").is_err() {
                env::set_var("ALPACA_PAPER_API_SECRET_KEY", "test_secret_key");
            }
        }
        MarketDataClient::new(AccountType::Paper).unwrap()
    }

    /// The endpoint's query parameter is `isins` (a comma-separated list of
    /// ISINs), not `symbols`.
    #[test]
    #[serial]
    fn request_uses_isins_query_param() {
        let client = paper_client();
        let request = client
            .fixed_income_latest_prices_request(&["US912797KJ59", "US912797KS58"])
            .unwrap()
            .build()
            .unwrap();
        let query = request.url().query().unwrap();
        assert_eq!(query, "isins=US912797KJ59%2CUS912797KS58");
    }

    /// The latest-prices response keys each `fixed_income_price` by ISIN and
    /// encodes the price as `p` and the timestamp as `t` (per the OpenAPI
    /// `fixed_income_price` schema). Pins the wire contract.
    #[test]
    fn prices_response_deserializes_spec_example() {
        let json = r#"{
            "prices": {
                "US912797KJ59": {
                    "p": 99.6459,
                    "t": "2025-02-14T20:58:00.648Z",
                    "ytm": 4.249,
                    "ytw": 4.249
                }
            }
        }"#;
        let parsed: PricesResponse = serde_json::from_str(json).unwrap();
        let price = &parsed.prices["US912797KJ59"];
        assert_eq!(price.price, 99.6459);
        assert_eq!(
            price.timestamp,
            DateTime::parse_from_rfc3339("2025-02-14T20:58:00.648Z")
                .unwrap()
                .with_timezone(&Utc)
        );
        assert_eq!(price.yield_to_maturity, Some(4.249));
        assert_eq!(price.yield_to_worst, Some(4.249));
    }
}
