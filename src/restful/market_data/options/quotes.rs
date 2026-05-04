use crate::restful::MarketDataClient;
use reqwest::Method;
use serde::Deserialize;

use super::OptionQuote;

#[derive(Debug, Deserialize)]
struct LatestQuotesResponse {
    quotes: std::collections::HashMap<String, OptionQuote>,
}

impl MarketDataClient {
    /// Get the latest option quotes.
    pub async fn option_latest_quotes(
        &self,
        symbols: &[&str],
    ) -> crate::Result<std::collections::HashMap<String, OptionQuote>> {
        let request = self
            .request(Method::GET, "v1beta1/options/quotes/latest")?
            .query(&[("symbols", symbols.join(","))]);
        let response: LatestQuotesResponse = self.send_and_deserialize(request).await?;
        Ok(response.quotes)
    }
}
