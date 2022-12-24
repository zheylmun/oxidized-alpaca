use crate::{rest_client::RestClient, utils::null_def_vec};
use chrono::{DateTime, Utc};
use reqwest::Method;
use serde::{Deserialize, Serialize};

use super::{Adjustment, Bar, Feed, TimeFrame};

/// A request for /v2/stocks/{symbol}/bars
#[derive(Serialize)]
#[must_use]
#[serde(rename_all = "snake_case")]
pub struct Request {
    /// The `RestClient` to use.
    #[serde(skip)]
    rest_client: RestClient,
    /// The symbol for which to retrieve market data.
    #[serde(skip)]
    symbol: String,
    /// The time frame for the bars.
    #[serde(rename = "timeframe")]
    time_frame: TimeFrame,
    /// The maximum number of bars to be returned for each symbol.
    ///
    /// It can be between 1 and 10000. Defaults to 1000 if the provided
    /// value is None.
    #[serde(skip_serializing_if = "Option::is_none")]
    limit: Option<usize>,
    /// Filter bars equal to or after this time.
    #[serde(skip_serializing_if = "Option::is_none")]
    start: Option<DateTime<Utc>>,
    /// Filter bars equal to or before this time.
    #[serde(skip_serializing_if = "Option::is_none")]
    end: Option<DateTime<Utc>>,
    /// The adjustment to use (defaults to raw)
    #[serde(skip_serializing_if = "Option::is_none")]
    adjustment: Option<Adjustment>,
    /// The data feed to use.
    ///
    /// Defaults to [`IEX`][Feed::IEX] for free users and
    /// [`SIP`][Feed::SIP] for users with an unlimited subscription.
    #[serde(skip_serializing_if = "Option::is_none")]
    feed: Option<Feed>,
    /// If provided we will pass a page token to continue where we left off.
    #[serde(skip_serializing_if = "Option::is_none")]
    page_token: Option<String>,
}

impl Request {
    /// Create a new request for market data bars`RestClient`
    ///
    pub fn new(client: RestClient, symbol: &str, time_frame: TimeFrame) -> Self {
        Self {
            rest_client: client,
            symbol: symbol.to_string(),
            time_frame,
            limit: None,
            start: None,
            end: None,
            adjustment: None,
            feed: None,
            page_token: None,
        }
    }

    /// Set the `limit` for the number of bars to be returned for each symbol.
    pub fn limit(mut self, limit: usize) -> Self {
        self.limit = Some(limit);
        self
    }

    /// Set the `start` date for the bars request.
    pub fn start(mut self, start: DateTime<Utc>) -> Self {
        self.start = Some(start);
        self
    }

    /// Set the `end` date for the bars request.
    pub fn end(mut self, end: DateTime<Utc>) -> Self {
        self.end = Some(end);
        self
    }

    /// Set the `adjustment` for the bars request.
    pub fn adjustment(mut self, adjustment: Adjustment) -> Self {
        self.adjustment = Some(adjustment);
        self
    }

    /// Set the `feed` for the bars request.
    pub fn feed(mut self, feed: Feed) -> Self {
        self.feed = Some(feed);
        self
    }

    /// Attempt to execute the configured request
    /// # Panics
    /// TEMP: This function will panic if the request fails.
    pub async fn execute(self) -> Vec<Bar> {
        let path = format!("v2/stocks/{}/bars", self.symbol);
        let request = self
            .rest_client
            .request(Method::GET, super::MARKET_DATA_REST_HOST, &path)
            .query(&self);
        let response = request.send().await.unwrap();

        let response = response.json::<Bars>().await.unwrap();
        response.bars
    }
}

/// A collection of bars as returned by the API. This is one page of bars.
#[derive(Debug, Deserialize, Eq, PartialEq)]
struct Bars {
    /// The list of returned bars.
    #[serde(default, deserialize_with = "null_def_vec")]
    pub bars: Vec<Bar>,
    /// The symbol the bars correspond to.
    pub symbol: String,
    /// The token to provide to a request to get the next page of bars for this request.
    pub next_page_token: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;
    use std::str::FromStr as _;

    use crate::AccountType;

    /// Check that we can decode a response containing no bars correctly.
    #[tokio::test]
    #[serial]
    async fn no_bars() {
        let client = RestClient::new(AccountType::Paper).unwrap();
        let start = DateTime::from_str("2021-11-05T00:00:00Z").unwrap();
        let end = DateTime::from_str("2021-11-05T00:00:00Z").unwrap();
        let request = Request::new(client, "AAPL", TimeFrame::OneDay)
            .start(start)
            .end(end);

        let res = request.execute().await;
        assert_eq!(res, vec![]);
    }
}
