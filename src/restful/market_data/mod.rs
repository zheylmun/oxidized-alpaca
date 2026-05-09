/// Corporate actions endpoint types and methods.
pub mod corporate_actions;
/// Crypto market data endpoint types and methods.
pub mod crypto;
/// Fixed income endpoint types and methods.
pub mod fixed_income;
/// Forex endpoint types and methods.
pub mod forex;
/// Logo endpoint types and methods.
pub mod logos;
/// News endpoint types and methods.
pub mod news;
/// Options market data endpoint types and methods.
pub mod options;
/// Screener endpoint types and methods.
pub mod screener;
/// Stock market data endpoint types and methods.
pub mod stock;

use serde::Serialize;

/// Supported time frames for the historical bars endpoints.
///
/// Accepted by [`stock_bars`][stock::bars], [`crypto_bars`][crypto::bars], and
/// [`option_bars`][options::bars] — every Alpaca historical-bars endpoint uses
/// the same wire format.
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
