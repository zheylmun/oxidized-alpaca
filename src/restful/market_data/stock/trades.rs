use crate::{
    RestFeed,
    restful::{MarketDataClient, SortDirection, null_def_vec},
};
use chrono::{DateTime, Utc};
use reqwest::Method;
use serde::{Deserialize, Serialize};

use super::AsOf;

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
pub struct TradesRequest<'a> {
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

impl TradesRequest<'_> {
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
    pub fn stock_trades<'a>(&'a self, symbol: &str) -> TradesRequest<'a> {
        TradesRequest {
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
    pub fn stock_trades_multi<'a>(&'a self, symbols: &[&str]) -> MultiSymbolTradesRequest<'a> {
        MultiSymbolTradesRequest {
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
pub struct MultiSymbolTradesRequest<'a> {
    #[serde(skip)]
    client: &'a MarketDataClient,
    symbols: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    start: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    end: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    feed: Option<RestFeed>,
    /// Per-symbol cap applied client-side after pagination (see
    /// [`MultiSymbolBarsRequest`][super::bars::MultiSymbolBarsRequest]
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

impl MultiSymbolTradesRequest<'_> {
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
    /// retrieved. When `limit` is set, pagination stops as soon as every
    /// requested symbol has reached the cap, and each symbol's series is
    /// truncated to at most that many trades afterward.
    pub async fn execute(
        mut self,
    ) -> crate::Result<std::collections::HashMap<String, Vec<StockTrade>>> {
        let cap = self.limit;
        let requested: Vec<String> = self.symbols.split(',').map(str::to_string).collect();
        let mut combined: std::collections::HashMap<String, Vec<StockTrade>> =
            std::collections::HashMap::new();
        loop {
            let request = self
                .client
                .request(Method::GET, "v2/stocks/trades")?
                .query(&self);
            let response: MultiTradesResponse = self.client.send_and_deserialize(request).await?;
            for (symbol, trades) in response.trades {
                combined.entry(symbol).or_default().extend(trades);
            }
            if let Some(cap) = cap
                && requested
                    .iter()
                    .all(|s| combined.get(s).is_some_and(|v| v.len() >= cap))
            {
                break;
            }
            match response.next_page_token {
                Some(token) => self.page_token = Some(token),
                None => break,
            }
        }
        if let Some(cap) = cap {
            for trades in combined.values_mut() {
                trades.truncate(cap);
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
    use super::MultiTradesResponse;

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
