use crate::restful::{MarketDataClient, SortDirection};
use chrono::{DateTime, Utc};
use reqwest::Method;
use serde::{Deserialize, Serialize};

use super::{CryptoLocation, CryptoTrade};
use crate::restful::market_data::pagination;

#[derive(Debug, Deserialize)]
struct TradesResponse {
    trades: std::collections::HashMap<String, Vec<CryptoTrade>>,
    next_page_token: Option<String>,
}

#[derive(Debug, Deserialize)]
struct LatestTradesResponse {
    trades: std::collections::HashMap<String, CryptoTrade>,
}

/// Builder for requesting historical crypto trades.
#[derive(Debug, Serialize)]
#[must_use]
pub struct CryptoTradesRequest<'a> {
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

impl CryptoTradesRequest<'_> {
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
    /// Cap the total number of trades returned per symbol across all
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

    /// Execute the request, auto-paginating until all matching trades are
    /// retrieved. When `limit` is set, each symbol's series is truncated to
    /// the cap as pages arrive, and pagination stops as soon as every
    /// requested symbol has reached the cap (or the API runs out of pages).
    pub async fn execute(
        mut self,
    ) -> crate::Result<std::collections::HashMap<String, Vec<CryptoTrade>>> {
        let cap = self.limit;
        if cap == Some(0) || self.symbols.is_empty() {
            return Ok(std::collections::HashMap::new());
        }
        let requested: Vec<String> = self.symbols.split(',').map(str::to_string).collect();
        let mut combined: std::collections::HashMap<String, Vec<CryptoTrade>> =
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
            let path = format!("v1beta3/crypto/{loc}/trades");
            let request = self.client.request(Method::GET, &path)?.query(&self);
            let response: TradesResponse = self.client.send_and_deserialize(request).await?;
            pagination::extend_capped(&mut combined, response.trades, cap);
            match response.next_page_token {
                Some(token) => self.page_token = Some(token),
                None => break,
            }
        }
        Ok(combined)
    }
}

impl MarketDataClient {
    /// Request historical crypto trades.
    pub fn crypto_trades<'a>(
        &'a self,
        symbols: &[&str],
        loc: CryptoLocation,
    ) -> CryptoTradesRequest<'a> {
        CryptoTradesRequest {
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

    /// Get the latest crypto trades.
    pub async fn crypto_latest_trades(
        &self,
        symbols: &[&str],
        loc: CryptoLocation,
    ) -> crate::Result<std::collections::HashMap<String, CryptoTrade>> {
        let path = format!("v1beta3/crypto/{loc}/latest/trades");
        let request = self
            .request(Method::GET, &path)?
            .query(&[("symbols", symbols.join(","))]);
        let response: LatestTradesResponse = self.send_and_deserialize(request).await?;
        Ok(response.trades)
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
            .crypto_trades(&["BTC/USD", "ETH/USD"], CryptoLocation::Us)
            .start(sample_start())
            .sort(SortDirection::Desc);
        let query = serde_urlencoded::to_string(&request).unwrap();
        assert!(query.contains("sort=desc"), "{query}");
        assert!(query.contains("start=2026-01-02T14%3A30%3A00Z"), "{query}");
        assert!(query.contains("symbols=BTC%2FUSD%2CETH%2FUSD"), "{query}");
    }

    #[test]
    #[serial]
    fn limit_does_not_serialize() {
        let client = paper_client();
        let request = client
            .crypto_trades(&["BTC/USD"], CryptoLocation::Us)
            .limit(50);
        let query = serde_urlencoded::to_string(&request).unwrap();
        assert!(!query.contains("limit"), "{query}");
    }

    #[tokio::test]
    #[serial]
    async fn limit_zero_short_circuits_without_request() {
        let client = paper_client();
        let result = client
            .crypto_trades(&["BTC/USD"], CryptoLocation::Us)
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
            .crypto_trades(&[], CryptoLocation::Us)
            .execute()
            .await
            .unwrap();
        assert!(result.is_empty());
    }

    /// Regression test for per-symbol capping across pages.
    ///
    /// Two symbols, cap 3. The first page satisfies BTC but leaves ETH one
    /// short, so a second page is needed. Whatever narrowing the loop does
    /// to reduce payload, it must never re-request a range it has already
    /// merged: doing so appends records the map already holds, which shows
    /// up as duplicate (and out-of-order) trades once truncated to the cap.
    #[tokio::test]
    #[serial]
    async fn paginates_without_duplicating_already_merged_trades() {
        use wiremock::matchers::{method, path};
        use wiremock::{Mock, MockServer, Respond, ResponseTemplate};

        fn trade(price: f64) -> String {
            format!(r#"{{"t":"2026-05-07T13:30:00Z","p":{price},"s":1.0}}"#)
        }

        struct Pages;
        impl Respond for Pages {
            fn respond(&self, request: &wiremock::Request) -> ResponseTemplate {
                let query: std::collections::HashMap<_, _> =
                    request.url.query_pairs().into_owned().collect();
                let body = if !query.contains_key("page_token") {
                    // Page 1: BTC reaches the cap of 3, ETH gets 2.
                    format!(
                        r#"{{"trades":{{"BTC/USD":[{},{},{}],"ETH/USD":[{},{}]}},"next_page_token":"p2"}}"#,
                        trade(1.0),
                        trade(2.0),
                        trade(3.0),
                        trade(10.0),
                        trade(11.0)
                    )
                } else {
                    // Page 2 continues where page 1 stopped.
                    format!(
                        r#"{{"trades":{{"ETH/USD":[{}]}},"next_page_token":null}}"#,
                        trade(12.0)
                    )
                };
                ResponseTemplate::new(200).set_body_raw(body, "application/json")
            }
        }

        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/v1beta3/crypto/us/trades"))
            .respond_with(Pages)
            .mount(&server)
            .await;

        let client = paper_client().with_base_url(&server.uri());
        let result = client
            .crypto_trades(&["BTC/USD", "ETH/USD"], CryptoLocation::Us)
            .limit(3)
            .execute()
            .await
            .unwrap();

        let eth: Vec<f64> = result["ETH/USD"].iter().map(|t| t.price).collect();
        assert_eq!(
            eth,
            vec![10.0, 11.0, 12.0],
            "ETH/USD series should continue across pages, not restart"
        );
        let btc: Vec<f64> = result["BTC/USD"].iter().map(|t| t.price).collect();
        assert_eq!(btc, vec![1.0, 2.0, 3.0]);
    }

    #[test]
    fn deserializes_multi_symbol_trades_response_with_pagination() {
        let json = r#"{
            "next_page_token": "QlRDL1VTRHwxNzc4MTYwNjAwMDM0OTM4NDAz",
            "trades": {
                "BTC/USD": [
                    {"t":"2026-05-07T13:30:00.015531758Z","p":103250.5,"s":0.014,"i":12345,"tks":"B"},
                    {"t":"2026-05-07T13:30:01.015531758Z","p":103251.0,"s":0.002,"i":12346,"tks":"S"}
                ]
            }
        }"#;
        let parsed: TradesResponse = serde_json::from_str(json).unwrap();
        assert_eq!(parsed.trades["BTC/USD"].len(), 2);
        assert_eq!(parsed.trades["BTC/USD"][0].price, 103_250.5);
        assert_eq!(
            parsed.trades["BTC/USD"][0].taker_side,
            Some(crate::CryptoTakerSide::Buyer)
        );
        assert!(parsed.next_page_token.is_some());
    }
}
