use crate::{
    RestFeed,
    restful::{MarketDataClient, SortDirection, null_def_vec},
};
use chrono::{DateTime, Utc};
use reqwest::Method;
use serde::{Deserialize, Serialize};

use super::{AsOf, pagination};

/// A stock trade.
#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct StockTrade {
    /// The timestamp of the trade.
    #[serde(rename = "t")]
    pub timestamp: DateTime<Utc>,
    /// The exchange code.
    #[serde(rename = "x")]
    pub exchange: String,
    /// The trade price.
    #[serde(rename = "p")]
    pub price: f64,
    /// The trade size.
    #[serde(rename = "s")]
    pub size: u32,
    /// The trade ID.
    #[serde(rename = "i", default)]
    pub trade_id: Option<u64>,
    /// Trade condition flags.
    #[serde(rename = "c", default)]
    pub conditions: Vec<String>,
    /// The tape.
    #[serde(rename = "z", default)]
    pub tape: Option<String>,
}

#[derive(Debug, Deserialize)]
struct TradesResponse {
    #[serde(default, deserialize_with = "null_def_vec")]
    trades: Vec<StockTrade>,
    #[allow(dead_code)]
    symbol: String,
    next_page_token: Option<String>,
}

#[derive(Debug, Deserialize)]
struct LatestTradeResponse {
    trade: StockTrade,
    #[allow(dead_code)]
    symbol: String,
}

#[derive(Debug, Deserialize)]
struct MultiLatestTradesResponse {
    trades: std::collections::HashMap<String, StockTrade>,
}

/// Builder for requesting historical stock trades.
#[derive(Debug, Serialize)]
#[must_use]
pub struct StockTradesRequest<'a> {
    #[serde(skip)]
    client: &'a MarketDataClient,
    #[serde(skip)]
    symbol: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    start: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    end: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    feed: Option<RestFeed>,
    #[serde(skip_serializing_if = "Option::is_none")]
    limit: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    asof: Option<AsOf>,
    #[serde(skip_serializing_if = "Option::is_none")]
    currency: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    sort: Option<SortDirection>,
    #[serde(skip_serializing_if = "Option::is_none")]
    page_token: Option<String>,
}

impl StockTradesRequest<'_> {
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
    /// Set the data feed to use.
    pub fn feed(mut self, feed: RestFeed) -> Self {
        self.feed = Some(feed);
        self
    }
    /// Cap the total number of trades returned across all auto-paginated pages.
    pub fn limit(mut self, limit: usize) -> Self {
        self.limit = Some(limit);
        self
    }
    /// Set the `asof` value, used to anchor symbol mapping for renamed
    /// instruments. Pass [`AsOf::SkipSymbolMapping`] to disable mapping.
    pub fn asof(mut self, asof: AsOf) -> Self {
        self.asof = Some(asof);
        self
    }
    /// Set the response `currency` (ISO 4217). Defaults to USD when unset.
    pub fn currency(mut self, currency: impl Into<String>) -> Self {
        self.currency = Some(currency.into());
        self
    }
    /// Set the result `sort` order. Defaults to ascending when unset.
    pub fn sort(mut self, sort: SortDirection) -> Self {
        self.sort = Some(sort);
        self
    }

    /// Execute the request, auto-paginating until all matching trades are
    /// retrieved or the configured `limit` is reached.
    pub async fn execute(mut self) -> crate::Result<Vec<StockTrade>> {
        let cap = self.limit;
        let mut all_trades = Vec::new();
        loop {
            let symbol = &self.symbol;
            let path = format!("v2/stocks/{symbol}/trades");
            let request = self.client.request(Method::GET, &path)?.query(&self);
            let response: TradesResponse = self.client.send_and_deserialize(request).await?;
            all_trades.extend(response.trades);
            if let Some(cap) = cap
                && all_trades.len() >= cap
            {
                all_trades.truncate(cap);
                break;
            }
            match response.next_page_token {
                Some(token) => self.page_token = Some(token),
                None => break,
            }
        }
        Ok(all_trades)
    }
}

impl MarketDataClient {
    /// Request historical trades for a single stock symbol.
    pub fn stock_trades<'a>(&'a self, symbol: &str) -> StockTradesRequest<'a> {
        StockTradesRequest {
            client: self,
            symbol: symbol.to_string(),
            start: None,
            end: None,
            feed: None,
            limit: None,
            asof: None,
            currency: None,
            sort: None,
            page_token: None,
        }
    }

    /// Get the latest trade for a single stock symbol.
    pub async fn stock_latest_trade(&self, symbol: &str) -> crate::Result<StockTrade> {
        let path = format!("v2/stocks/{symbol}/trades/latest");
        let request = self.request(Method::GET, &path)?;
        let response: LatestTradeResponse = self.send_and_deserialize(request).await?;
        Ok(response.trade)
    }

    /// Get the latest trades for multiple stock symbols.
    pub async fn stock_latest_trades(
        &self,
        symbols: &[&str],
    ) -> crate::Result<std::collections::HashMap<String, StockTrade>> {
        let request = self
            .request(Method::GET, "v2/stocks/trades/latest")?
            .query(&[("symbols", symbols.join(","))]);
        let response: MultiLatestTradesResponse = self.send_and_deserialize(request).await?;
        Ok(response.trades)
    }

    /// Request historical trades for multiple stock symbols. Returns a
    /// map keyed by symbol; symbols with no trades in the queried range
    /// are omitted from the response.
    pub fn stock_trades_multi<'a>(&'a self, symbols: &[&str]) -> StockTradesMultiRequest<'a> {
        StockTradesMultiRequest {
            client: self,
            symbols: symbols.join(","),
            start: None,
            end: None,
            feed: None,
            limit: None,
            asof: None,
            currency: None,
            sort: None,
            page_token: None,
        }
    }
}

/// A request for `/v2/stocks/trades` (multi-symbol historical trades).
#[derive(Debug, Serialize)]
#[must_use]
pub struct StockTradesMultiRequest<'a> {
    #[serde(skip)]
    client: &'a MarketDataClient,
    symbols: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    start: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    end: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    feed: Option<RestFeed>,
    /// Per-symbol cap applied client-side during pagination, with each
    /// symbol's series truncated to the cap as pages arrive (see
    /// [`StockBarsMultiRequest`][super::bars::StockBarsMultiRequest]
    /// for rationale).
    #[serde(skip)]
    limit: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    asof: Option<AsOf>,
    #[serde(skip_serializing_if = "Option::is_none")]
    currency: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    sort: Option<SortDirection>,
    #[serde(skip_serializing_if = "Option::is_none")]
    page_token: Option<String>,
}

impl StockTradesMultiRequest<'_> {
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
    /// Set the data feed to use.
    pub fn feed(mut self, feed: RestFeed) -> Self {
        self.feed = Some(feed);
        self
    }
    /// Cap the total number of trades returned per symbol across all
    /// auto-paginated pages.
    pub fn limit(mut self, limit: usize) -> Self {
        self.limit = Some(limit);
        self
    }
    /// Set the `asof` value, used to anchor symbol mapping for renamed
    /// instruments. Pass [`AsOf::SkipSymbolMapping`] to disable mapping.
    pub fn asof(mut self, asof: AsOf) -> Self {
        self.asof = Some(asof);
        self
    }
    /// Set the response `currency` (ISO 4217). Defaults to USD when unset.
    pub fn currency(mut self, currency: impl Into<String>) -> Self {
        self.currency = Some(currency.into());
        self
    }
    /// Set the result `sort` order. Defaults to ascending when unset.
    pub fn sort(mut self, sort: SortDirection) -> Self {
        self.sort = Some(sort);
        self
    }

    /// Execute the request, auto-paginating until all matching trades are
    /// retrieved. When `limit` is set, each symbol's series is truncated
    /// to the cap as pages arrive, and pagination stops as soon as every
    /// requested symbol has reached the cap (or the API runs out of pages).
    pub async fn execute(
        mut self,
    ) -> crate::Result<std::collections::HashMap<String, Vec<StockTrade>>> {
        let cap = self.limit;
        if cap == Some(0) || self.symbols.is_empty() {
            return Ok(std::collections::HashMap::new());
        }
        let requested: Vec<String> = self.symbols.split(',').map(str::to_string).collect();
        let mut combined: std::collections::HashMap<String, Vec<StockTrade>> =
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
            let request = self
                .client
                .request(Method::GET, "v2/stocks/trades")?
                .query(&self);
            let response: MultiTradesResponse = self.client.send_and_deserialize(request).await?;
            pagination::extend_capped(&mut combined, response.trades, cap);
            match response.next_page_token {
                Some(token) => self.page_token = Some(token),
                None => break,
            }
        }
        Ok(combined)
    }
}

#[derive(Debug, Deserialize)]
struct MultiTradesResponse {
    #[serde(default)]
    trades: std::collections::HashMap<String, Vec<StockTrade>>,
    next_page_token: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::AccountType;
    use serial_test::serial;
    use std::env;

    fn ensure_paper_creds() {
        unsafe {
            if env::var("ALPACA_PAPER_API_KEY_ID").is_err() {
                env::set_var("ALPACA_PAPER_API_KEY_ID", "test_key_id");
            }
            if env::var("ALPACA_PAPER_API_SECRET_KEY").is_err() {
                env::set_var("ALPACA_PAPER_API_SECRET_KEY", "test_secret_key");
            }
        }
    }

    fn paper_client() -> MarketDataClient {
        ensure_paper_creds();
        MarketDataClient::new(AccountType::Paper).unwrap()
    }

    fn sample_start() -> DateTime<Utc> {
        DateTime::parse_from_rfc3339("2026-01-02T14:30:00Z")
            .unwrap()
            .with_timezone(&Utc)
    }

    fn sample_end() -> DateTime<Utc> {
        DateTime::parse_from_rfc3339("2026-01-03T20:00:00Z")
            .unwrap()
            .with_timezone(&Utc)
    }

    #[test]
    #[serial]
    fn multi_constructor_joins_symbols() {
        let client = paper_client();
        let request = client.stock_trades_multi(&["AAPL", "MSFT", "TSLA"]);
        let query = serde_urlencoded::to_string(&request).unwrap();
        assert!(
            query.contains("symbols=AAPL%2CMSFT%2CTSLA"),
            "expected joined symbols in {query}"
        );
    }

    #[test]
    #[serial]
    fn multi_builder_setters_serialize_to_query() {
        let client = paper_client();
        let request = client
            .stock_trades_multi(&["AAPL", "MSFT"])
            .start(sample_start())
            .end(sample_end())
            .feed(RestFeed::IEX)
            .asof(AsOf::SkipSymbolMapping)
            .currency("EUR")
            .sort(SortDirection::Desc);
        let query = serde_urlencoded::to_string(&request).unwrap();
        assert!(query.contains("start=2026-01-02T14%3A30%3A00Z"), "{query}");
        assert!(query.contains("end=2026-01-03T20%3A00%3A00Z"), "{query}");
        assert!(query.contains("feed=iex"), "{query}");
        assert!(query.contains("asof=-"), "{query}");
        assert!(query.contains("currency=EUR"), "{query}");
        assert!(query.contains("sort=desc"), "{query}");
    }

    #[test]
    #[serial]
    fn multi_limit_does_not_serialize() {
        let client = paper_client();
        let request = client.stock_trades_multi(&["AAPL"]).limit(50);
        let query = serde_urlencoded::to_string(&request).unwrap();
        assert!(
            !query.contains("limit"),
            "expected `limit` not to be serialized; got {query}"
        );
    }

    #[tokio::test]
    #[serial]
    async fn multi_limit_zero_short_circuits_without_request() {
        let client = paper_client();
        let result = client
            .stock_trades_multi(&["AAPL", "MSFT"])
            .limit(0)
            .execute()
            .await
            .unwrap();
        assert!(result.is_empty());
    }

    #[tokio::test]
    #[serial]
    async fn multi_empty_symbols_short_circuits_without_request() {
        let client = paper_client();
        let result = client.stock_trades_multi(&[]).execute().await.unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn deserializes_multi_symbol_trades_response_with_pagination() {
        let json = r#"{
            "next_page_token": "QUFQTHwxNzc4MTYwNjAwMDE3NTA0MDEwfFF8MzQ2MA==",
            "trades": {
                "AAPL": [
                    {"c":["@"],"i":5112,"p":289.27,"s":50,"t":"2026-05-07T13:30:00.01343918Z","x":"D","z":"C"},
                    {"c":["@","I"],"i":5113,"p":289.27,"s":13,"t":"2026-05-07T13:30:00.014659266Z","x":"D","z":"C"}
                ]
            }
        }"#;
        let parsed: MultiTradesResponse = serde_json::from_str(json).unwrap();
        assert_eq!(parsed.trades.len(), 1);
        assert_eq!(parsed.trades["AAPL"].len(), 2);
        assert_eq!(parsed.trades["AAPL"][0].price, 289.27);
        assert_eq!(parsed.trades["AAPL"][0].size, 50);
        assert!(parsed.next_page_token.is_some());
    }

    #[test]
    fn deserializes_empty_multi_symbol_trades_response() {
        let json = r#"{"trades": {}, "next_page_token": null}"#;
        let parsed: MultiTradesResponse = serde_json::from_str(json).unwrap();
        assert!(parsed.trades.is_empty());
    }
}
