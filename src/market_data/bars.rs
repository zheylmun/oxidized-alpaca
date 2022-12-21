use chrono::{DateTime, Utc};
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

/// An enumeration of the various supported time frames.
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
    /// A time frame of one week.
    #[serde(rename = "1Month")]
    OneMonth,
}

/// An enumeration of the adjustment
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
pub struct BarsReq {
    /// The symbol for which to retrieve market data.
    #[serde(skip)]
    pub symbol: String,
    /// The maximum number of bars to be returned for each symbol.
    ///
    /// It can be between 1 and 10000. Defaults to 1000 if the provided
    /// value is None.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<usize>,
    /// Filter bars equal to or after this time.
    pub start: DateTime<Utc>,
    /// Filter bars equal to or before this time.
    pub end: DateTime<Utc>,
    /// The time frame for the bars.
    pub timeframe: TimeFrame,
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
