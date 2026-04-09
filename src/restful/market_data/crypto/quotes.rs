use crate::restful::MarketDataClient;
use reqwest::Method;
use serde::Deserialize;

use super::{CryptoLocation, CryptoQuote};

#[derive(Debug, Deserialize)]
struct LatestQuotesResponse {
    quotes: std::collections::HashMap<String, CryptoQuote>,
}

impl MarketDataClient {
    /// Get the latest crypto quotes.
    pub async fn crypto_latest_quotes(
        &self,
        symbols: &str,
        loc: CryptoLocation,
    ) -> crate::Result<std::collections::HashMap<String, CryptoQuote>> {
        let path = format!("v1beta3/crypto/{}/latest/quotes", loc.as_str());
        let request = self
            .request(Method::GET, &path)
            .query(&[("symbols", symbols)]);
        let response: LatestQuotesResponse = self.send_and_deserialize(request).await?;
        Ok(response.quotes)
    }
}
