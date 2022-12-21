use crate::{rest_client::RestClient, utils::null_def_vec};
use chrono::{DateTime, Utc};
use reqwest::Method;
use serde::{Deserialize, Serialize};

/// An enumeration of the different supported data feeds.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Feed {
    /// Use the Investors Exchange (IEX) as the data source.
    ///
    /// This feed is available unconditionally, i.e., with the free and
    /// unlimited plans.
    IEX,
    /// Use CTA (administered by NYSE) and UTP (administered by Nasdaq)
    /// SIPs as the data source.
    ///
    /// This feed is only usable with the unlimited market data plan.
    SIP,
}

/// Supported Time frames for bars
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
pub enum TimeFrame {
    /// A time frame of one minute.
    #[serde(rename = "1Min")]
    OneMinute,
    /// A time frame of five minutes.
    #[serde(rename = "5Min")]
    FiveMinute,
    /// A time frame of fifteen minutes.
    #[serde(rename = "15Min")]
    FifteenMinute,
    /// A time frame of thirty minutes.
    #[serde(rename = "30Min")]
    ThirtyMinute,
    /// A time frame of one hour.
    #[serde(rename = "1Hour")]
    OneHour,
    /// A time frame of two hours.
    #[serde(rename = "2Hour")]
    TwoHour,
    /// A time frame of four hours.
    #[serde(rename = "4Hour")]
    FourHour,
    /// A time frame of one day.
    #[serde(rename = "1Day")]
    OneDay,
    /// A time frame of one week.
    #[serde(rename = "1Week")]
    OneWeek,
    /// A time frame of one month.
    #[serde(rename = "1Month")]
    OneMonth,
}

///  Data adjustment Options
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Adjustment {
    /// No adjustment, i.e., raw data.
    Raw,
    /// Adjustment for stock splits.
    Split,
    /// Adjustment for dividends.
    Dividend,
    /// All available corporate adjustments.
    All,
}

/// A request for /v2/stocks/{symbol}/bars
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct BarsRequest {
    /// The symbol for which to retrieve market data.
    #[serde(skip)]
    pub symbol: String,
    /// The time frame for the bars.
    pub timeframe: TimeFrame,
    /// The maximum number of bars to be returned for each symbol.
    ///
    /// It can be between 1 and 10000. Defaults to 1000 if the provided
    /// value is None.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<usize>,
    /// Filter bars equal to or after this time.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start: Option<DateTime<Utc>>,
    /// Filter bars equal to or before this time.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end: Option<DateTime<Utc>>,
    /// The adjustment to use (defaults to raw)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub adjustment: Option<Adjustment>,
    /// The data feed to use.
    ///
    /// Defaults to [`IEX`][Feed::IEX] for free users and
    /// [`SIP`][Feed::SIP] for users with an unlimited subscription.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub feed: Option<Feed>,
    /// If provided we will pass a page token to continue where we left off.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page_token: Option<String>,
}

/// A market data bar as returned by the /v2/stocks/<symbol>/bars endpoint.
#[derive(Clone, Debug, Deserialize, PartialEq)]
#[non_exhaustive]
pub struct Bar {
    /// The beginning time of this bar.
    #[serde(rename = "t")]
    pub time: DateTime<Utc>,
    /// The open price.
    #[serde(rename = "o")]
    pub open: f64,
    /// The close price.
    #[serde(rename = "c")]
    pub close: f64,
    /// The highest price.
    #[serde(rename = "h")]
    pub high: f64,
    /// The lowest price.
    #[serde(rename = "l")]
    pub low: f64,
    /// The trading volume.
    #[serde(rename = "v")]
    pub volume: usize,
}

impl Eq for Bar {}

/// A collection of bars as returned by the API. This is one page of bars.
#[derive(Debug, Deserialize, Eq, PartialEq)]
pub struct Bars {
    /// The list of returned bars.
    #[serde(default, deserialize_with = "null_def_vec")]
    pub bars: Vec<Bar>,
    /// The symbol the bars correspond to.
    pub symbol: String,
    /// The token to provide to a request to get the next page of bars for this request.
    pub next_page_token: Option<String>,
}

impl Bars {
    /// Gets the bars data for the BarsRequest
    ///
    /// # Example
    ///
    /// To get the bars for AAPL from 2021-11-01 to 2021-11-31:
    ///
    /// ``` no run
    /// let alpaca_env = env::Env::new(Paper);
    /// let client = RestClient::new(alpaca_env);
    /// let start = DateTime::from_str("2021-11-05T00:00:00Z").unwrap();
    /// let end = DateTime::from_str("2021-11-31T00:00:00Z").unwrap();
    /// let request = BarsRequest {
    ///     symbol: "AAPL".to_string(),
    ///     timeframe: TimeFrame::OneDay,
    ///     limit: None,
    ///     start: Some(start),
    ///     end: Some(end),
    ///     adjustment: None,
    ///     feed: None,
    ///     page_token: None,
    /// };
    /// ```
    pub async fn get(client: &RestClient, request: &BarsRequest) -> Self {
        let path = format!("v2/stocks/{}/bars", request.symbol);
        let request = client
            .request(Method::GET, super::MARKET_DATA_REST_HOST, &path)
            .query(&request);
        let response = request.send().await.unwrap();

        assert!(response.status().is_success());
        //let text = response.text().await.unwrap();
        //print!("{}", text);
        response.json::<Bars>().await.unwrap()
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;
    use std::str::FromStr as _;

    use crate::env::{AccountType, Env};

    /// Check that we can decode a response containing no bars correctly.
    #[tokio::test]
    #[serial]
    async fn no_bars() {
        let env = Env::new(AccountType::Paper);
        let client = RestClient::new(env);
        let start = DateTime::from_str("2021-11-05T00:00:00Z").unwrap();
        let end = DateTime::from_str("2021-11-05T00:00:00Z").unwrap();
        let request = BarsRequest {
            symbol: "AAPL".to_string(),
            timeframe: TimeFrame::OneDay,
            limit: None,
            start: Some(start),
            end: Some(end),
            adjustment: None,
            feed: None,
            page_token: None,
        };
        let res = Bars::get(&client, &request).await;
        assert_eq!(res.bars, vec![]);
    }
}
