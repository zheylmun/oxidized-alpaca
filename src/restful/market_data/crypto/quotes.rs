use crate::restful::{MarketDataClient, SortDirection};
use chrono::{DateTime, Utc};
use reqwest::Method;
use serde::{Deserialize, Serialize};

use super::{CryptoLocation, CryptoQuote};
use crate::restful::market_data::pagination;

#[derive(Debug, Deserialize)]
struct QuotesResponse {
    quotes: std::collections::HashMap<String, Vec<CryptoQuote>>,
    next_page_token: Option<String>,
}

#[derive(Debug, Deserialize)]
struct LatestQuotesResponse {
    quotes: std::collections::HashMap<String, CryptoQuote>,
}

/// Builder for requesting historical crypto quotes.
#[derive(Debug, Serialize)]
#[must_use]
pub struct CryptoQuotesRequest<'a> {
    #[serde(skip)]
    client: &'a MarketDataClient,
    #[serde(skip)]
    loc: CryptoLocation,
    symbols: String,
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

impl CryptoQuotesRequest<'_> {
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
    /// Cap the total number of quotes returned per symbol across all
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

    /// Execute the request, auto-paginating until all matching quotes are
    /// retrieved. When `limit` is set, each symbol's series is truncated to
    /// the cap as pages arrive, and pagination stops as soon as every
    /// requested symbol has reached the cap (or the API runs out of pages).
    pub async fn execute(
        mut self,
    ) -> crate::Result<std::collections::HashMap<String, Vec<CryptoQuote>>> {
        let cap = self.limit;
        if cap == Some(0) || self.symbols.is_empty() {
            return Ok(std::collections::HashMap::new());
        }
        let requested: Vec<String> = self.symbols.split(',').map(str::to_string).collect();
        let mut combined: std::collections::HashMap<String, Vec<CryptoQuote>> =
            std::collections::HashMap::new();
        loop {
            if let Some(cap) = cap {
                let pending = pagination::pending_symbols(&combined, &requested, cap);
                if pending.is_empty() {
                    break;
                }
                let next_symbols = pending.join(",");
                if next_symbols != self.symbols {
                    // Narrowing to the symbols still under the cap avoids
                    // paging through a saturated symbol's entire range to
                    // reach a lagging one. The cursor is tied to the symbol
                    // set, so it has to be cleared -- which restarts the
                    // range and would re-append what we already merged.
                    // Drop those partial series so the restart refills them.
                    pagination::drop_partials(&mut combined, &pending);
                    self.symbols = next_symbols;
                    self.page_token = None;
                }
            }
            let loc = self.loc;
            let path = format!("v1beta3/crypto/{loc}/quotes");
            let request = self.client.request(Method::GET, &path)?.query(&self);
            let response: QuotesResponse = self.client.send_and_deserialize(request).await?;
            pagination::extend_capped(&mut combined, response.quotes, cap);
            match response.next_page_token {
                Some(token) => self.page_token = Some(token),
                None => break,
            }
        }
        Ok(combined)
    }
}

impl MarketDataClient {
    /// Request historical crypto quotes.
    pub fn crypto_quotes<'a>(
        &'a self,
        symbols: &[&str],
        loc: CryptoLocation,
    ) -> CryptoQuotesRequest<'a> {
        CryptoQuotesRequest {
            client: self,
            loc,
            symbols: symbols.join(","),
            start: None,
            end: None,
            limit: None,
            page_token: None,
            sort: None,
        }
    }

    /// Get the latest crypto quotes.
    pub async fn crypto_latest_quotes(
        &self,
        symbols: &[&str],
        loc: CryptoLocation,
    ) -> crate::Result<std::collections::HashMap<String, CryptoQuote>> {
        let path = format!("v1beta3/crypto/{loc}/latest/quotes");
        let request = self
            .request(Method::GET, &path)?
            .query(&[("symbols", symbols.join(","))]);
        let response: LatestQuotesResponse = self.send_and_deserialize(request).await?;
        Ok(response.quotes)
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

    fn sample_start() -> DateTime<Utc> {
        DateTime::parse_from_rfc3339("2026-01-02T14:30:00Z")
            .unwrap()
            .with_timezone(&Utc)
    }

    #[test]
    #[serial]
    fn filters_serialize_to_query() {
        let client = paper_client();
        let request = client
            .crypto_quotes(&["BTC/USD"], CryptoLocation::Us)
            .start(sample_start())
            .sort(SortDirection::Desc);
        let query = serde_urlencoded::to_string(&request).unwrap();
        assert!(query.contains("sort=desc"), "{query}");
        assert!(query.contains("start=2026-01-02T14%3A30%3A00Z"), "{query}");
        assert!(query.contains("symbols=BTC%2FUSD"), "{query}");
    }

    #[test]
    #[serial]
    fn limit_does_not_serialize() {
        let client = paper_client();
        let request = client
            .crypto_quotes(&["BTC/USD"], CryptoLocation::Us)
            .limit(5);
        let query = serde_urlencoded::to_string(&request).unwrap();
        assert!(!query.contains("limit"), "{query}");
    }

    #[tokio::test]
    #[serial]
    async fn limit_zero_short_circuits_without_request() {
        let client = paper_client();
        let result = client
            .crypto_quotes(&["BTC/USD"], CryptoLocation::Us)
            .limit(0)
            .execute()
            .await
            .unwrap();
        assert!(result.is_empty());
    }

    #[tokio::test]
    #[serial]
    async fn empty_symbols_short_circuits_without_request() {
        let client = paper_client();
        let result = client
            .crypto_quotes(&[], CryptoLocation::Us)
            .execute()
            .await
            .unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn deserializes_multi_symbol_quotes_response_with_pagination() {
        let json = r#"{
            "next_page_token": "QlRDL1VTRHwxNzc4MTYwNjAwMDM0OTM4NDAz",
            "quotes": {
                "BTC/USD": [
                    {"t":"2026-05-07T13:30:00.015531758Z","bp":103250.0,"bs":0.5,"ap":103251.0,"as":0.4}
                ]
            }
        }"#;
        let parsed: QuotesResponse = serde_json::from_str(json).unwrap();
        assert_eq!(parsed.quotes["BTC/USD"].len(), 1);
        assert_eq!(parsed.quotes["BTC/USD"][0].bid_price, 103_250.0);
        assert!(parsed.next_page_token.is_some());
    }
}
