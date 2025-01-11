use crate::{
    error::{self, Error},
    restful::{null_def_vec, rest_client::RequestAPI},
    Feed, RestClient,
};
use chrono::{DateTime, Utc};
use reqwest::Method;
use serde::{Deserialize, Serialize};

use super::{Adjustment, Bar, TimeFrame};

/// A request for /v2/stocks/{symbol}/bars
#[derive(Debug, Serialize)]
#[must_use]
#[serde(rename_all = "snake_case")]
pub struct Request<'a> {
    /// The `RestClient` to use.
    #[serde(skip)]
    rest_client: &'a RestClient,
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

impl Request<'_> {
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
    ///
    /// # Errors
    /// - Returns a [`Error::ReqwestSend`] if the rest request fails.
    /// - Returns a [`Error::ReqwestDeserialize`] if the response cannot be parsed
    #[tracing::instrument]
    pub async fn execute(mut self) -> Result<Vec<Bar>, Error> {
        let mut response = self.internal_execute().await?;
        let mut results = response.bars;
        while response.next_page_token.is_some() {
            self.page_token = response.next_page_token;
            response = self.internal_execute().await?;
            results.extend(response.bars);
        }
        Ok(results)
    }

    #[tracing::instrument]
    async fn internal_execute(&self) -> Result<Bars, Error> {
        let path = format!("v2/stocks/{}/bars", self.symbol);
        let request = self
            .rest_client
            .request(Method::GET, RequestAPI::StockMarketData, &path)
            .query(&self);
        let response = request.send().await.map_err(Error::ReqwestSend)?;
        response
            .json::<Bars>()
            .await
            .map_err(error::Error::ReqwestDeserialize)
    }
}

/// Create a new request for market data bars`RestClient`
///
/// # Arguments
/// * `client` - The `RestClient` to use.
/// * `symbol` - The symbol for which to retrieve market data.
/// * `time_frame` - The time frame for the bars.
pub fn get<'a>(client: &'a RestClient, symbol: &str, time_frame: TimeFrame) -> Request<'a> {
    Request {
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
        let client = RestClient::new(AccountType::Paper).unwrap();
        let start = DateTime::from_str("2022-12-05T00:00:00Z").unwrap();
        let end = DateTime::from_str("2022-12-05T00:00:00Z").unwrap();
        let request = Request::new(&client, "META", TimeFrame::OneDay)
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
        let client = RestClient::new(AccountType::Paper).unwrap();
        let start = DateTime::from_str("2022-12-05T00:00:00Z").unwrap();
        let end = DateTime::from_str("2022-12-06T00:00:00Z").unwrap();
        let request = Request::new(&client, "AAPL", TimeFrame::OneDay)
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

    /// Check that we can get a response containing several bars correctly.
    #[tokio::test]
    #[parallel]
    async fn some_bars() {
        let client = RestClient::new(AccountType::Paper).unwrap();
        let start = DateTime::from_str("2022-12-05T00:00:00Z").unwrap();
        let end = DateTime::from_str("2022-12-24T00:00:00Z").unwrap();
        let request = Request::new(&client, "NFLX", TimeFrame::OneDay)
            .start(start)
            .end(end)
            .adjustment(Adjustment::All);

        let res = request.execute().await;

        assert!(res.is_ok());
        let res = res.unwrap();
        assert_eq!(res.len(), 15);
    }

    /// Check that we can get a request requiring multiple pages of data successfully.
    #[tokio::test]
    #[parallel]
    async fn lots_of_bars() {
        let client = RestClient::new(AccountType::Paper).unwrap();
        let start = DateTime::from_str("2021-12-05T00:00:00Z").unwrap();
        let end = DateTime::from_str("2022-12-24T00:00:00Z").unwrap();
        let request = Request::new(&client, "GOOGL", TimeFrame::OneDay)
            .start(start)
            .end(end)
            .feed(Feed::IEX)
            .limit(50);

        let res = request.execute().await;

        assert!(res.is_ok());
        let res = res.unwrap();
        assert_eq!(res.len(), 266);
    }
    */
}
