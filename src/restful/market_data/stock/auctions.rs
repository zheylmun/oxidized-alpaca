use crate::{
    RestFeed,
    restful::{MarketDataClient, null_def_vec},
};
use chrono::{DateTime, Utc};
use reqwest::Method;
use serde::{Deserialize, Serialize};

/// An auction price entry.
#[derive(Clone, Debug, Deserialize, PartialEq)]
#[non_exhaustive]
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
#[non_exhaustive]
pub struct DailyAuctions {
    /// The date of the auctions.
    #[serde(rename = "d")]
    pub date: String,
    /// Opening auction entries.
    #[serde(rename = "o", default, deserialize_with = "null_def_vec")]
    pub opening: Vec<AuctionEntry>,
    /// Closing auction entries.
    #[serde(rename = "c", default, deserialize_with = "null_def_vec")]
    pub closing: Vec<AuctionEntry>,
}

/// A single auction entry.
#[derive(Clone, Debug, Deserialize)]
#[non_exhaustive]
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
pub struct StockAuctionsRequest<'a> {
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

impl StockAuctionsRequest<'_> {
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
    pub fn stock_auctions<'a>(&'a self, symbol: &str) -> StockAuctionsRequest<'a> {
        StockAuctionsRequest {
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

#[cfg(test)]
mod tests {
    use super::AuctionsResponse;

    /// Pre-open sessions return null for the closing auctions field; ensure
    /// that does not break deserialization.
    #[test]
    fn deserializes_daily_auction_with_null_closing() {
        let json = r#"{
            "auctions": [
                {
                    "c": null,
                    "d": "2026-05-08",
                    "o": [
                        {"c":"Q","p":290.11,"s":1,"t":"2026-05-08T13:30:00.138698399Z","x":"P"}
                    ]
                }
            ],
            "next_page_token": null,
            "symbol": "AAPL"
        }"#;
        let parsed: AuctionsResponse = serde_json::from_str(json).unwrap();
        assert_eq!(parsed.auctions.len(), 1);
        assert_eq!(parsed.auctions[0].opening.len(), 1);
        assert!(parsed.auctions[0].closing.is_empty());
    }

    /// Symmetric case: when only the closing auction has occurred, the
    /// opening field can come back null.
    #[test]
    fn deserializes_daily_auction_with_null_opening() {
        let json = r#"{
            "auctions": [
                {
                    "o": null,
                    "d": "2026-05-08",
                    "c": [
                        {"c":"M","p":291.5,"s":1234,"t":"2026-05-08T20:00:00Z","x":"Q"}
                    ]
                }
            ],
            "next_page_token": null,
            "symbol": "AAPL"
        }"#;
        let parsed: AuctionsResponse = serde_json::from_str(json).unwrap();
        assert_eq!(parsed.auctions.len(), 1);
        assert!(parsed.auctions[0].opening.is_empty());
        assert_eq!(parsed.auctions[0].closing.len(), 1);
        assert_eq!(parsed.auctions[0].closing[0].price, 291.5);
        assert_eq!(parsed.auctions[0].closing[0].size, 1234);
        assert_eq!(parsed.auctions[0].closing[0].exchange.as_deref(), Some("Q"));
        assert_eq!(
            parsed.auctions[0].closing[0].condition.as_deref(),
            Some("M")
        );
    }

    /// A normal post-close response with both sides populated across
    /// multiple days exercises the happy path and pagination signal.
    #[test]
    fn deserializes_multi_day_response_with_pagination_token() {
        let json = r#"{
            "auctions": [
                {
                    "d": "2026-05-07",
                    "o": [{"c":"Q","p":287.1,"s":100,"t":"2026-05-07T13:30:00Z","x":"P"}],
                    "c": [{"c":"M","p":290.0,"s":200,"t":"2026-05-07T20:00:00Z","x":"Q"}]
                },
                {
                    "d": "2026-05-08",
                    "o": [{"c":"Q","p":290.5,"s":150,"t":"2026-05-08T13:30:00Z","x":"P"}],
                    "c": [{"c":"M","p":291.5,"s":250,"t":"2026-05-08T20:00:00Z","x":"Q"}]
                }
            ],
            "next_page_token": "abc123",
            "symbol": "AAPL"
        }"#;
        let parsed: AuctionsResponse = serde_json::from_str(json).unwrap();
        assert_eq!(parsed.auctions.len(), 2);
        assert_eq!(parsed.auctions[0].date, "2026-05-07");
        assert_eq!(parsed.auctions[1].date, "2026-05-08");
        assert_eq!(parsed.next_page_token.as_deref(), Some("abc123"));
    }
}
