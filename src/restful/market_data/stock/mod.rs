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

/// Value passed to the historical stock endpoints' `asof` query parameter.
///
/// Alpaca uses `asof` to resolve symbol mapping across renames. Pass a
/// [`Date`][AsOf::Date] to anchor the mapping at a specific calendar day,
/// or [`SkipSymbolMapping`][AsOf::SkipSymbolMapping] to disable mapping
/// (sent as the literal `"-"`).
///
/// `asof` is a request-only parameter; Alpaca does not echo it back on
/// any response shape, so this enum implements [`Serialize`] (for
/// query-string emission) but intentionally not `Deserialize`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
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
#[non_exhaustive]
pub enum Adjustment {
    /// No adjustment, i.e., raw data.
    Raw,
    /// Adjustment for stock splits.
    Split,
    /// Adjustment for dividends.
    Dividend,
    /// Adjustment for spin-offs.
    #[serde(rename = "spin-off")]
    SpinOff,
    /// All available corporate adjustments.
    All,
}

impl Adjustment {
    fn as_str(self) -> &'static str {
        match self {
            Self::Raw => "raw",
            Self::Split => "split",
            Self::Dividend => "dividend",
            Self::SpinOff => "spin-off",
            Self::All => "all",
        }
    }
}

/// Ordered, deduplicated list of [`Adjustment`] values for the
/// `adjustment` query parameter. Alpaca accepts multiple values
/// combined with commas (e.g. `split,dividend,spin-off`).
///
/// An empty list serializes to an empty string, which is not a valid
/// `adjustment` value. Prefer constructing through
/// [`StockBarsRequest::adjustments`][bars::StockBarsRequest::adjustments],
/// which omits the parameter when the iterator is empty so Alpaca's
/// default of `raw` is used.
///
/// ```
/// use oxidized_alpaca::restful::market_data::stock::{Adjustment, AdjustmentList};
///
/// let list = AdjustmentList::new([Adjustment::Split, Adjustment::Dividend]);
/// assert_eq!(list.iter().count(), 2);
/// assert_eq!(list.as_slice().len(), 2);
/// for adjustment in &list {
///     let _: &Adjustment = adjustment;
/// }
/// ```
#[derive(Clone, Debug)]
#[non_exhaustive]
pub struct AdjustmentList(Vec<Adjustment>);

impl AdjustmentList {
    /// Construct from any iterator of [`Adjustment`] values. Duplicate
    /// values are dropped while preserving the order of first occurrence.
    pub fn new<I: IntoIterator<Item = Adjustment>>(items: I) -> Self {
        let mut out = Vec::new();
        for item in items {
            if !out.contains(&item) {
                out.push(item);
            }
        }
        Self(out)
    }

    /// Returns `true` if no adjustments are set.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Borrow the contained adjustments as a slice.
    #[must_use]
    pub fn as_slice(&self) -> &[Adjustment] {
        &self.0
    }

    /// Iterate the contained adjustments by reference.
    pub fn iter(&self) -> std::slice::Iter<'_, Adjustment> {
        self.0.iter()
    }
}

impl<'a> IntoIterator for &'a AdjustmentList {
    type Item = &'a Adjustment;
    type IntoIter = std::slice::Iter<'a, Adjustment>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

impl IntoIterator for AdjustmentList {
    type Item = Adjustment;
    type IntoIter = std::vec::IntoIter<Adjustment>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl From<Adjustment> for AdjustmentList {
    fn from(a: Adjustment) -> Self {
        Self(vec![a])
    }
}

impl Serialize for AdjustmentList {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let joined = self
            .0
            .iter()
            .map(|a| a.as_str())
            .collect::<Vec<_>>()
            .join(",");
        serializer.serialize_str(&joined)
    }
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
    pub volume: u64,
    /// The number of trades in the bar.
    #[serde(rename = "n")]
    pub trade_count: u64,
    /// The volume-weighted average price.
    #[serde(rename = "vw")]
    pub vwap: f64,
}

#[cfg(test)]
mod tests {
    use super::{Adjustment, AdjustmentList, AsOf};
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

    #[test]
    fn adjustment_serializes_with_documented_wire_strings() {
        for (value, expected) in [
            (Adjustment::Raw, "\"raw\""),
            (Adjustment::Split, "\"split\""),
            (Adjustment::Dividend, "\"dividend\""),
            (Adjustment::SpinOff, "\"spin-off\""),
            (Adjustment::All, "\"all\""),
        ] {
            assert_eq!(serde_json::to_string(&value).unwrap(), expected);
        }
    }

    #[test]
    fn adjustment_list_joins_values_with_commas() {
        let list =
            AdjustmentList::new([Adjustment::Split, Adjustment::Dividend, Adjustment::SpinOff]);
        assert_eq!(
            serde_json::to_string(&list).unwrap(),
            "\"split,dividend,spin-off\""
        );
    }

    #[test]
    fn adjustment_list_serializes_every_variant() {
        let list = AdjustmentList::new([
            Adjustment::Raw,
            Adjustment::Split,
            Adjustment::Dividend,
            Adjustment::SpinOff,
            Adjustment::All,
        ]);
        assert_eq!(
            serde_json::to_string(&list).unwrap(),
            "\"raw,split,dividend,spin-off,all\""
        );
    }

    #[test]
    fn adjustment_list_from_single_value() {
        let list: AdjustmentList = Adjustment::Split.into();
        assert_eq!(serde_json::to_string(&list).unwrap(), "\"split\"");
    }

    #[test]
    fn adjustment_list_reports_empty() {
        assert!(AdjustmentList::new(std::iter::empty()).is_empty());
        assert!(!AdjustmentList::new([Adjustment::Split]).is_empty());
    }

    #[test]
    fn adjustment_list_iter_and_as_slice_expose_contents() {
        let list = AdjustmentList::new([Adjustment::Split, Adjustment::Dividend]);
        assert_eq!(list.as_slice(), &[Adjustment::Split, Adjustment::Dividend]);
        let collected: Vec<Adjustment> = list.iter().copied().collect();
        assert_eq!(collected, vec![Adjustment::Split, Adjustment::Dividend]);
    }

    #[test]
    fn adjustment_list_into_iter_yields_borrowed_and_owned_values() {
        let list = AdjustmentList::new([Adjustment::Split, Adjustment::Dividend]);
        let by_ref: Vec<&Adjustment> = (&list).into_iter().collect();
        assert_eq!(by_ref, vec![&Adjustment::Split, &Adjustment::Dividend]);
        let owned: Vec<Adjustment> = list.into_iter().collect();
        assert_eq!(owned, vec![Adjustment::Split, Adjustment::Dividend]);
    }

    #[test]
    fn adjustment_list_dedupes_preserving_first_occurrence() {
        let list = AdjustmentList::new([
            Adjustment::Split,
            Adjustment::Dividend,
            Adjustment::Split,
            Adjustment::SpinOff,
            Adjustment::Dividend,
        ]);
        assert_eq!(
            serde_json::to_string(&list).unwrap(),
            "\"split,dividend,spin-off\""
        );
    }
}
