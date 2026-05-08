use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize, Serializer};

/// Stock auctions endpoint types and methods.
pub mod auctions;
/// Stock bars endpoint types and methods.
pub mod bars;
/// Stock metadata endpoint types and methods.
pub mod meta;
/// Stock quotes endpoint types and methods.
pub mod quotes;
/// Stock snapshots endpoint types and methods.
pub mod snapshots;
/// Stock trades endpoint types and methods.
pub mod trades;

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

/// Value passed to the historical stock endpoints' `asof` query parameter.
///
/// Alpaca uses `asof` to resolve symbol mapping across renames. Pass a
/// [`Date`][AsOf::Date] to anchor the mapping at a specific calendar day,
/// or [`SkipSymbolMapping`][AsOf::SkipSymbolMapping] to disable mapping
/// (sent as the literal `"-"`).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AsOf {
    /// A specific calendar date (sent as `YYYY-MM-DD`).
    Date(NaiveDate),
    /// Skip symbol mapping (sent as the literal `-`).
    SkipSymbolMapping,
}

impl Serialize for AsOf {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Date(date) => serializer.collect_str(&date.format("%Y-%m-%d")),
            Self::SkipSymbolMapping => serializer.serialize_str("-"),
        }
    }
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
/// A market data bar as returned by one of the bars endpoints.
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

#[cfg(test)]
mod tests {
    use super::AsOf;
    use chrono::NaiveDate;

    #[test]
    fn asof_date_serializes_as_iso_calendar_day() {
        let asof = AsOf::Date(NaiveDate::from_ymd_opt(2024, 1, 15).unwrap());
        assert_eq!(serde_json::to_string(&asof).unwrap(), "\"2024-01-15\"");
    }

    #[test]
    fn asof_skip_mapping_serializes_as_dash() {
        assert_eq!(
            serde_json::to_string(&AsOf::SkipSymbolMapping).unwrap(),
            "\"-\""
        );
    }
}
