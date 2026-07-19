use crate::restful::{MarketDataClient, SortDirection, market_data::TimeFrame};
use chrono::{DateTime, Utc};
use reqwest::Method;
use serde::{Deserialize, Serialize};

use super::{CryptoBar, CryptoLocation};
use crate::restful::market_data::pagination;

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
    timeframe: TimeFrame,
    #[serde(skip_serializing_if = "Option::is_none")]
    start: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    end: Option<DateTime<Utc>>,
    /// Per-symbol cap applied client-side during pagination.
    #[serde(skip)]
    limit: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    page_token: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    sort: Option<SortDirection>,
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
    /// Set the result `sort` order. Defaults to ascending when unset.
    pub fn sort(mut self, sort: SortDirection) -> Self {
        self.sort = Some(sort);
        self
    }

    /// Execute the request, auto-paginating until all matching bars are
    /// retrieved. When `limit` is set, each symbol's series is truncated to
    /// the cap as pages arrive, and pagination stops as soon as every
    /// requested symbol has reached the cap (or the API runs out of pages).
    pub async fn execute(
        mut self,
    ) -> crate::Result<std::collections::HashMap<String, Vec<CryptoBar>>> {
        let cap = self.limit;
        if cap == Some(0) || self.symbols.is_empty() {
            return Ok(std::collections::HashMap::new());
        }
        let requested: Vec<String> = self.symbols.split(',').map(str::to_string).collect();
        let mut combined: std::collections::HashMap<String, Vec<CryptoBar>> =
            std::collections::HashMap::new();
        loop {
            if let Some(cap) = cap {
                let pending = pagination::pending_symbols(&combined, &requested, cap);
                if pending.is_empty() {
                    break;
                }
                let next_symbols = pending.join(",");
                if next_symbols != self.symbols {
                    self.symbols = next_symbols;
                    self.page_token = None;
                }
            }
            let loc = self.loc;
            let path = format!("v1beta3/crypto/{loc}/bars");
            let request = self.client.request(Method::GET, &path)?.query(&self);
            let response: BarsResponse = self.client.send_and_deserialize(request).await?;
            pagination::extend_capped(&mut combined, response.bars, cap);
            match response.next_page_token {
                Some(token) => self.page_token = Some(token),
                None => break,
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
        timeframe: TimeFrame,
        loc: CryptoLocation,
    ) -> CryptoBarsRequest<'a> {
        CryptoBarsRequest {
            client: self,
            loc,
            symbols: symbols.join(","),
            timeframe,
            start: None,
            end: None,
            limit: None,
            page_token: None,
            sort: None,
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
            .request(Method::GET, &path)?
            .query(&[("symbols", symbols.join(","))]);
        let response: LatestBarsResponse = self.send_and_deserialize(request).await?;
        Ok(response.bars)
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

    #[test]
    #[serial]
    fn sort_serializes_to_query() {
        let client = paper_client();
        let request = client
            .crypto_bars(&["BTC/USD"], TimeFrame::ONE_DAY, CryptoLocation::Us)
            .sort(crate::restful::SortDirection::Desc);
        let query = serde_urlencoded::to_string(&request).unwrap();
        assert!(query.contains("sort=desc"), "{query}");
    }
}
