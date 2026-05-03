use crate::restful::MarketDataClient;
use chrono::{DateTime, Utc};
use reqwest::Method;
use serde::{Deserialize, Serialize};

use super::OptionBar;

#[derive(Debug, Deserialize)]
struct BarsResponse {
    bars: std::collections::HashMap<String, Vec<OptionBar>>,
    next_page_token: Option<String>,
}

/// Builder for requesting option bars.
#[derive(Debug, Serialize)]
#[must_use]
pub struct OptionBarsRequest<'a> {
    #[serde(skip)]
    client: &'a MarketDataClient,
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

impl OptionBarsRequest<'_> {
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
    /// Cap the number of bars returned per symbol after auto-pagination.
    pub fn limit(mut self, limit: usize) -> Self {
        self.limit = Some(limit);
        self
    }

    /// Execute the request, auto-paginating until all matching bars are
    /// retrieved. When `limit` is set, each symbol's series is truncated to
    /// at most that many bars after pagination completes.
    pub async fn execute(
        mut self,
    ) -> crate::Result<std::collections::HashMap<String, Vec<OptionBar>>> {
        let cap = self.limit;
        let mut combined: std::collections::HashMap<String, Vec<OptionBar>> =
            std::collections::HashMap::new();
        loop {
            let request = self
                .client
                .request(Method::GET, "v1beta1/options/bars")
                .query(&self);
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
    /// Request option bars.
    pub fn option_bars<'a>(&'a self, symbols: &[&str], timeframe: &str) -> OptionBarsRequest<'a> {
        OptionBarsRequest {
            client: self,
            symbols: symbols.join(","),
            timeframe: timeframe.to_string(),
            start: None,
            end: None,
            limit: None,
            page_token: None,
        }
    }
}
