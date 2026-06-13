use crate::{RestFeed, restful::MarketDataClient};
use reqwest::Method;
use serde::{Deserialize, Serialize};

use super::{Bar, quotes::StockQuote, trades::StockTrade};

/// A stock snapshot containing latest trade, quote, and bar data.
#[derive(Clone, Debug, Deserialize)]
#[non_exhaustive]
pub struct StockSnapshot {
    /// The latest trade.
    #[serde(rename = "latestTrade")]
    pub latest_trade: Option<StockTrade>,
    /// The latest quote.
    #[serde(rename = "latestQuote")]
    pub latest_quote: Option<StockQuote>,
    /// The latest minute bar.
    #[serde(rename = "minuteBar")]
    pub minute_bar: Option<Bar>,
    /// The current daily bar.
    #[serde(rename = "dailyBar")]
    pub daily_bar: Option<Bar>,
    /// The previous day's daily bar.
    #[serde(rename = "prevDailyBar")]
    pub prev_daily_bar: Option<Bar>,
}

/// Builder for a single-symbol stock snapshot.
#[derive(Debug, Serialize)]
#[must_use]
pub struct StockSnapshotRequest<'a> {
    #[serde(skip)]
    client: &'a MarketDataClient,
    #[serde(skip)]
    symbol: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    feed: Option<RestFeed>,
    #[serde(skip_serializing_if = "Option::is_none")]
    currency: Option<String>,
}

impl StockSnapshotRequest<'_> {
    /// Set the data feed to use.
    pub fn feed(mut self, feed: RestFeed) -> Self {
        self.feed = Some(feed);
        self
    }
    /// Set the response `currency` (ISO 4217). Defaults to USD when unset.
    pub fn currency(mut self, currency: impl Into<String>) -> Self {
        self.currency = Some(currency.into());
        self
    }
    /// Send the request.
    pub async fn execute(self) -> crate::Result<StockSnapshot> {
        let symbol = &self.symbol;
        let path = format!("v2/stocks/{symbol}/snapshot");
        let request = self.client.request(Method::GET, &path)?.query(&self);
        self.client.send_and_deserialize(request).await
    }
}

/// Builder for multi-symbol stock snapshots.
#[derive(Debug, Serialize)]
#[must_use]
pub struct StockSnapshotsRequest<'a> {
    #[serde(skip)]
    client: &'a MarketDataClient,
    symbols: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    feed: Option<RestFeed>,
    #[serde(skip_serializing_if = "Option::is_none")]
    currency: Option<String>,
}

impl StockSnapshotsRequest<'_> {
    /// Set the data feed to use.
    pub fn feed(mut self, feed: RestFeed) -> Self {
        self.feed = Some(feed);
        self
    }
    /// Set the response `currency` (ISO 4217). Defaults to USD when unset.
    pub fn currency(mut self, currency: impl Into<String>) -> Self {
        self.currency = Some(currency.into());
        self
    }
    /// Send the request.
    pub async fn execute(self) -> crate::Result<std::collections::HashMap<String, StockSnapshot>> {
        let request = self
            .client
            .request(Method::GET, "v2/stocks/snapshots")?
            .query(&self);
        self.client.send_and_deserialize(request).await
    }
}

impl MarketDataClient {
    /// Request a snapshot for a single stock symbol.
    ///
    /// Returns a builder; call [`StockSnapshotRequest::execute`] to send.
    pub fn stock_snapshot<'a>(&'a self, symbol: &str) -> StockSnapshotRequest<'a> {
        StockSnapshotRequest {
            client: self,
            symbol: symbol.to_string(),
            feed: None,
            currency: None,
        }
    }

    /// Request snapshots for multiple stock symbols.
    ///
    /// Returns a builder; call [`StockSnapshotsRequest::execute`] to send.
    pub fn stock_snapshots<'a>(&'a self, symbols: &[&str]) -> StockSnapshotsRequest<'a> {
        StockSnapshotsRequest {
            client: self,
            symbols: symbols.join(","),
            feed: None,
            currency: None,
        }
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
    fn snapshot_feed_and_currency_serialize() {
        let client = paper_client();
        let request = client
            .stock_snapshot("AAPL")
            .feed(RestFeed::IEX)
            .currency("USD");
        let query = serde_urlencoded::to_string(&request).unwrap();
        assert!(query.contains("feed=iex"), "{query}");
        assert!(query.contains("currency=USD"), "{query}");
    }

    #[test]
    #[serial]
    fn snapshots_symbols_feed_currency_serialize() {
        let client = paper_client();
        let request = client
            .stock_snapshots(&["AAPL", "MSFT"])
            .feed(RestFeed::SIP)
            .currency("USD");
        let query = serde_urlencoded::to_string(&request).unwrap();
        assert!(query.contains("symbols=AAPL%2CMSFT"), "{query}");
        assert!(query.contains("feed=sip"), "{query}");
        assert!(query.contains("currency=USD"), "{query}");
    }
}
