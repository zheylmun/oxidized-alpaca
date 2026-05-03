use crate::restful::MarketDataClient;
use chrono::{DateTime, Utc};
use reqwest::Method;
use serde::{Deserialize, Serialize};

use super::{CryptoBar, CryptoLocation};

#[derive(Debug, Deserialize)]
struct BarsResponse {
    bars: std::collections::HashMap<String, Vec<CryptoBar>>,
    next_page_token: Option<String>,
}

#[derive(Debug, Deserialize)]
struct LatestBarsResponse {
    bars: std::collections::HashMap<String, CryptoBar>,
}

/// Builder for requesting crypto bars.
#[derive(Debug, Serialize)]
#[must_use]
pub struct CryptoBarsRequest<'a> {
    #[serde(skip)]
    client: &'a MarketDataClient,
    #[serde(skip)]
    loc: CryptoLocation,
    symbols: String,
    timeframe: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    start: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    end: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    limit: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    page_token: Option<String>,
}

impl CryptoBarsRequest<'_> {
    /// Set the start time filter.
    pub fn start(mut self, start: DateTime<Utc>) -> Self {
        self.start = Some(start);
        self
    }
    /// Set the end time filter.
    pub fn end(mut self, end: DateTime<Utc>) -> Self {
        self.end = Some(end);
        self
    }
    /// Cap the total number of bars returned per symbol across all
    /// auto-paginated pages.
    pub fn limit(mut self, limit: usize) -> Self {
        self.limit = Some(limit);
        self
    }

    /// Execute the request, auto-paginating until all matching bars are
    /// retrieved. When `limit` is set, each symbol's series is truncated to
    /// at most that many bars after pagination completes.
    pub async fn execute(
        mut self,
    ) -> crate::Result<std::collections::HashMap<String, Vec<CryptoBar>>> {
        let cap = self.limit;
        let mut combined: std::collections::HashMap<String, Vec<CryptoBar>> =
            std::collections::HashMap::new();
        loop {
            let loc = self.loc;
            let path = format!("v1beta3/crypto/{loc}/bars");
            let request = self.client.request(Method::GET, &path).query(&self);
            let response: BarsResponse = self.client.send_and_deserialize(request).await?;
            for (symbol, bars) in response.bars {
                combined.entry(symbol).or_default().extend(bars);
            }
            match response.next_page_token {
                Some(token) => self.page_token = Some(token),
                None => break,
            }
        }
        if let Some(cap) = cap {
            for bars in combined.values_mut() {
                bars.truncate(cap);
            }
        }
        Ok(combined)
    }
}

impl MarketDataClient {
    /// Request crypto bars.
    pub fn crypto_bars<'a>(
        &'a self,
        symbols: &[&str],
        timeframe: &str,
        loc: CryptoLocation,
    ) -> CryptoBarsRequest<'a> {
        CryptoBarsRequest {
            client: self,
            loc,
            symbols: symbols.join(","),
            timeframe: timeframe.to_string(),
            start: None,
            end: None,
            limit: None,
            page_token: None,
        }
    }

    /// Get the latest crypto bars.
    pub async fn crypto_latest_bars(
        &self,
        symbols: &[&str],
        loc: CryptoLocation,
    ) -> crate::Result<std::collections::HashMap<String, CryptoBar>> {
        let path = format!("v1beta3/crypto/{loc}/latest/bars");
        let request = self
            .request(Method::GET, &path)
            .query(&[("symbols", symbols.join(","))]);
        let response: LatestBarsResponse = self.send_and_deserialize(request).await?;
        Ok(response.bars)
    }
}
