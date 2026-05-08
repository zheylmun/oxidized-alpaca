use crate::{
    RestFeed,
    error::Error,
    restful::{MarketDataClient, null_def_vec},
};
use chrono::{DateTime, Utc};
use reqwest::Method;
use serde::{Deserialize, Serialize};

use super::{Adjustment, Bar, TimeFrame};

/// A request for /v2/stocks/{symbol}/bars
#[derive(Debug, Serialize)]
#[must_use]
#[serde(rename_all = "snake_case")]
pub struct StockBarsRequest<'a> {
    /// The `MarketDataClient` to use.
    #[serde(skip)]
    client: &'a MarketDataClient,
    /// The symbol for which to retrieve market data.
    #[serde(skip)]
    symbol: String,
    /// The time frame for the bars.
    timeframe: TimeFrame,
    /// The maximum total number of bars to return across all pages.
    ///
    /// When unset all matching bars are returned.
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
    /// Defaults to [`IEX`][RestFeed::IEX] for free users and
    /// [`SIP`][RestFeed::SIP] for users with an unlimited subscription.
    #[serde(skip_serializing_if = "Option::is_none")]
    feed: Option<RestFeed>,
    /// If provided we will pass a page token to continue where we left off.
    #[serde(skip_serializing_if = "Option::is_none")]
    page_token: Option<String>,
}

impl StockBarsRequest<'_> {
    /// Cap the total number of bars returned across all auto-paginated pages.
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
    pub fn feed(mut self, feed: RestFeed) -> Self {
        self.feed = Some(feed);
        self
    }

    /// Attempt to execute the configured request
    ///
    /// # Errors
    /// - Returns a [`Error::ReqwestSend`] if the rest request fails.
    /// - Returns a [`Error::ReqwestDeserialize`] if the response cannot be parsed
    #[cfg_attr(feature = "tracing", tracing::instrument)]
    pub async fn execute(mut self) -> Result<Vec<Bar>, Error> {
        let cap = self.limit;
        let mut results = Vec::new();
        loop {
            let response = self.internal_execute().await?;
            results.extend(response.bars);
            if let Some(cap) = cap
                && results.len() >= cap
            {
                results.truncate(cap);
                break;
            }
            match response.next_page_token {
                Some(token) => self.page_token = Some(token),
                None => break,
            }
        }
        Ok(results)
    }

    #[cfg_attr(feature = "tracing", tracing::instrument)]
    async fn internal_execute(&self) -> Result<Bars, Error> {
        let symbol = &self.symbol;
        let path = format!("v2/stocks/{symbol}/bars");
        let request = self.client.request(Method::GET, &path)?.query(&self);
        self.client.send_and_deserialize(request).await
    }
}

/// A collection of bars as returned by the API. This is one page of bars.
#[derive(Debug, Deserialize, PartialEq)]
struct Bars {
    /// The list of returned bars.
    #[serde(default, deserialize_with = "null_def_vec")]
    pub bars: Vec<Bar>,
    /// The symbol the bars correspond to.
    pub symbol: String,
    /// The token to provide to a request to get the next page of bars for this request.
    pub next_page_token: Option<String>,
}

impl MarketDataClient {
    /// Request historical bars for a single stock symbol.
    ///
    /// ```ignore
    /// let bars = client.stock_bars("AAPL", TimeFrame::OneDay)
    ///     .start(dt)
    ///     .limit(100)
    ///     .execute().await?;
    /// ```
    pub fn stock_bars<'a>(&'a self, symbol: &str, timeframe: TimeFrame) -> StockBarsRequest<'a> {
        StockBarsRequest {
            client: self,
            symbol: symbol.to_string(),
            timeframe,
            limit: None,
            start: None,
            end: None,
            adjustment: None,
            feed: None,
            page_token: None,
        }
    }
}

#[cfg(test)]
mod tests {
    /*
    use super::*;
    use serial_test::parallel;
    use std::str::FromStr as _;

    use crate::AccountType;

    /// Check that we can decode a response containing no bars correctly.
    #[tokio::test]
    #[parallel]
    async fn no_bars() {
        let client = MarketDataClient::new(AccountType::Paper).unwrap();
        let start = DateTime::from_str("2022-12-05T00:00:00Z").unwrap();
        let end = DateTime::from_str("2022-12-05T00:00:00Z").unwrap();
        let request = client.stock_bars("META", TimeFrame::OneDay)
            .start(start)
            .end(end);

        let res = request.execute().await;
        println!("{res:#?}");
        assert_eq!(res.unwrap(), vec![]);
    }

    /// Check that we can decode a response containing one bar correctly.
    #[tokio::test]
    #[parallel]
    async fn one_bar() {
        let client = MarketDataClient::new(AccountType::Paper).unwrap();
        let start = DateTime::from_str("2022-12-05T00:00:00Z").unwrap();
        let end = DateTime::from_str("2022-12-06T00:00:00Z").unwrap();
        let request = client.stock_bars("AAPL", TimeFrame::OneDay)
            .start(start)
            .end(end);

        let res = request.execute().await;
        let expected = Bar {
            time: DateTime::from_str("2022-12-05T05:00:00Z").unwrap(),
            open: 147.77,
            close: 146.63,
            high: 150.9199,
            low: 145.77,
            volume: 74981324,
        };
        assert!(res.is_ok());
        let res = res.unwrap();
        assert_eq!(res.len(), 1);
        assert_eq!(res[0], expected);
    }
    */
}
