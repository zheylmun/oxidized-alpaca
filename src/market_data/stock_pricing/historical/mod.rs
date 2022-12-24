use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

pub mod bars;

pub(crate) const MARKET_DATA_REST_HOST: &str = "https://data.alpaca.markets/v2";

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
