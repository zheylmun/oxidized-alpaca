use crate::{
    RestFeed,
    restful::{MarketDataClient, null_def_vec},
};
use chrono::{DateTime, Utc};
use reqwest::Method;
use serde::{Deserialize, Serialize};

/// An auction price entry.
#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct AuctionPrice {
    /// The date of the auction.
    #[serde(rename = "d")]
    pub date: Option<String>,
    /// The opening auction price.
    #[serde(rename = "o")]
    pub open: Option<f64>,
    /// The closing auction price.
    #[serde(rename = "c")]
    pub close: Option<f64>,
}

/// A daily auction record.
#[derive(Clone, Debug, Deserialize)]
pub struct DailyAuctions {
    /// The date of the auctions.
    #[serde(rename = "d")]
    pub date: String,
    /// Opening auction entries.
    #[serde(rename = "o", default)]
    pub opening: Vec<AuctionEntry>,
    /// Closing auction entries.
    #[serde(rename = "c", default)]
    pub closing: Vec<AuctionEntry>,
}

/// A single auction entry.
#[derive(Clone, Debug, Deserialize)]
pub struct AuctionEntry {
    /// The timestamp of the auction.
    #[serde(rename = "t")]
    pub timestamp: DateTime<Utc>,
    /// The auction price.
    #[serde(rename = "p")]
    pub price: f64,
    /// The auction size.
    #[serde(rename = "s")]
    pub size: u64,
    /// The exchange code.
    #[serde(rename = "x", default)]
    pub exchange: Option<String>,
    /// The condition flag.
    #[serde(rename = "c", default)]
    pub condition: Option<String>,
}

#[derive(Debug, Deserialize)]
struct AuctionsResponse {
    #[serde(default, deserialize_with = "null_def_vec")]
    auctions: Vec<DailyAuctions>,
    #[allow(dead_code)]
    symbol: String,
    next_page_token: Option<String>,
}

/// Builder for requesting stock auction data.
#[derive(Debug, Serialize)]
#[must_use]
pub struct AuctionsRequest<'a> {
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
    page_token: Option<String>,
}

impl AuctionsRequest<'_> {
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
    /// Cap the total number of daily auctions returned across all
    /// auto-paginated pages.
    pub fn limit(mut self, limit: usize) -> Self {
        self.limit = Some(limit);
        self
    }

    /// Execute the request, auto-paginating until all matching auctions are
    /// retrieved or the configured `limit` is reached.
    pub async fn execute(mut self) -> crate::Result<Vec<DailyAuctions>> {
        let cap = self.limit;
        let mut all = Vec::new();
        loop {
            let symbol = &self.symbol;
            let path = format!("v2/stocks/{symbol}/auctions");
            let request = self.client.request(Method::GET, &path)?.query(&self);
            let response: AuctionsResponse = self.client.send_and_deserialize(request).await?;
            all.extend(response.auctions);
            if let Some(cap) = cap
                && all.len() >= cap
            {
                all.truncate(cap);
                break;
            }
            match response.next_page_token {
                Some(token) => self.page_token = Some(token),
                None => break,
            }
        }
        Ok(all)
    }
}

impl MarketDataClient {
    /// Request historical auction data for a single stock symbol.
    pub fn stock_auctions<'a>(&'a self, symbol: &str) -> AuctionsRequest<'a> {
        AuctionsRequest {
            client: self,
            symbol: symbol.to_string(),
            start: None,
            end: None,
            feed: None,
            limit: None,
            page_token: None,
        }
    }
}
