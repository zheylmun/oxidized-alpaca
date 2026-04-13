use crate::{Feed, restful::MarketDataClient};
use chrono::{DateTime, Utc};
use reqwest::Method;
use serde::{Deserialize, Serialize};

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
    feed: Option<Feed>,
    #[serde(skip_serializing_if = "Option::is_none")]
    limit: Option<usize>,
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
    pub fn feed(mut self, feed: Feed) -> Self {
        self.feed = Some(feed);
        self
    }
    /// Set the maximum number of trades to return.
    pub fn limit(mut self, limit: usize) -> Self {
        self.limit = Some(limit);
        self
    }

    /// Execute with auto-pagination.
    pub async fn execute(mut self) -> crate::Result<Vec<StockTrade>> {
        let mut all_trades = Vec::new();
        loop {
            let path = format!("v2/stocks/{}/trades", self.symbol);
            let request = self.client.request(Method::GET, &path).query(&self);
            let response: TradesResponse = self.client.send_and_deserialize(request).await?;
            all_trades.extend(response.trades);
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
            page_token: None,
        }
    }

    /// Get the latest trade for a single stock symbol.
    pub async fn stock_latest_trade(&self, symbol: &str) -> crate::Result<StockTrade> {
        let path = format!("v2/stocks/{symbol}/trades/latest");
        let request = self.request(Method::GET, &path);
        let response: LatestTradeResponse = self.send_and_deserialize(request).await?;
        Ok(response.trade)
    }

    /// Get the latest trades for multiple stock symbols.
    pub async fn stock_latest_trades(
        &self,
        symbols: &[&str],
    ) -> crate::Result<std::collections::HashMap<String, StockTrade>> {
        let request = self
            .request(Method::GET, "v2/stocks/trades/latest")
            .query(&[("symbols", symbols.join(","))]);
        let response: MultiLatestTradesResponse = self.send_and_deserialize(request).await?;
        Ok(response.trades)
    }
}
