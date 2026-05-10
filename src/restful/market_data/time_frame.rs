//! Time-frame value used by every Alpaca historical-bars endpoint.

use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;

use crate::error::Error;

/// Bar duration accepted by [`super::stock::bars`], [`super::crypto::bars`],
/// and [`super::options::bars`] — every Alpaca historical-bars endpoint
/// uses the same wire format.
///
/// The wire format is the multiplier and unit concatenated, e.g. `5Min`,
/// `1Hour`, `3Month`. Construct values with the named constants for the
/// common cases ([`TimeFrame::ONE_DAY`], [`TimeFrame::FIFTEEN_MINUTES`], …)
/// or with the unit constructors ([`TimeFrame::minutes`],
/// [`TimeFrame::hours`], [`TimeFrame::days`], [`TimeFrame::weeks`],
/// [`TimeFrame::months`]) for arbitrary multipliers within Alpaca's
/// documented ranges.
///
/// Alpaca's documented multiplier limits are 1–59 minutes, 1–23 hours, and
/// 1–366 days. Week and month multipliers do not have explicit limits in
/// the public docs; the server may reject values it does not support.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct TimeFrame {
    amount: u16,
    unit: TimeFrameUnit,
}

/// The unit half of a [`TimeFrame`].
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum TimeFrameUnit {
    /// Minutes.
    Minute,
    /// Hours.
    Hour,
    /// Days.
    Day,
    /// Weeks.
    Week,
    /// Months.
    Month,
}

impl TimeFrameUnit {
    /// Wire-format suffix Alpaca expects for this unit.
    #[must_use]
    pub const fn as_wire_str(self) -> &'static str {
        match self {
            Self::Minute => "Min",
            Self::Hour => "Hour",
            Self::Day => "Day",
            Self::Week => "Week",
            Self::Month => "Month",
        }
    }
}

impl fmt::Display for TimeFrameUnit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_wire_str())
    }
}

impl TimeFrame {
    /// One-minute bars.
    pub const ONE_MINUTE: Self = Self::raw(1, TimeFrameUnit::Minute);
    /// Five-minute bars.
    pub const FIVE_MINUTES: Self = Self::raw(5, TimeFrameUnit::Minute);
    /// Fifteen-minute bars.
    pub const FIFTEEN_MINUTES: Self = Self::raw(15, TimeFrameUnit::Minute);
    /// Thirty-minute bars.
    pub const THIRTY_MINUTES: Self = Self::raw(30, TimeFrameUnit::Minute);
    /// One-hour bars.
    pub const ONE_HOUR: Self = Self::raw(1, TimeFrameUnit::Hour);
    /// Two-hour bars.
    pub const TWO_HOURS: Self = Self::raw(2, TimeFrameUnit::Hour);
    /// Four-hour bars.
    pub const FOUR_HOURS: Self = Self::raw(4, TimeFrameUnit::Hour);
    /// One-day bars.
    pub const ONE_DAY: Self = Self::raw(1, TimeFrameUnit::Day);
    /// One-week bars.
    pub const ONE_WEEK: Self = Self::raw(1, TimeFrameUnit::Week);
    /// One-month bars.
    pub const ONE_MONTH: Self = Self::raw(1, TimeFrameUnit::Month);
    /// Three-month bars (the longest multiplier explicitly documented by Alpaca).
    pub const THREE_MONTHS: Self = Self::raw(3, TimeFrameUnit::Month);

    const fn raw(amount: u16, unit: TimeFrameUnit) -> Self {
        Self { amount, unit }
    }

    /// Multiplier portion of the time frame (the `5` in `5Min`).
    #[must_use]
    pub const fn amount(self) -> u16 {
        self.amount
    }

    /// Unit portion of the time frame (the `Min` in `5Min`).
    #[must_use]
    pub const fn unit(self) -> TimeFrameUnit {
        self.unit
    }

    /// Build an `n`-minute time frame. Alpaca accepts 1–59.
    ///
    /// # Errors
    ///
    /// Returns [`Error::InvalidTimeFrame`] when `n` is outside the documented
    /// range.
    pub fn minutes(n: u16) -> Result<Self, Error> {
        Self::checked(n, TimeFrameUnit::Minute, 1, 59, "1..=59")
    }

    /// Build an `n`-hour time frame. Alpaca accepts 1–23.
    ///
    /// # Errors
    ///
    /// Returns [`Error::InvalidTimeFrame`] when `n` is outside the documented
    /// range.
    pub fn hours(n: u16) -> Result<Self, Error> {
        Self::checked(n, TimeFrameUnit::Hour, 1, 23, "1..=23")
    }

    /// Build an `n`-day time frame. Alpaca accepts 1–366.
    ///
    /// # Errors
    ///
    /// Returns [`Error::InvalidTimeFrame`] when `n` is outside the documented
    /// range.
    pub fn days(n: u16) -> Result<Self, Error> {
        Self::checked(n, TimeFrameUnit::Day, 1, 366, "1..=366")
    }

    /// Build an `n`-week time frame. Alpaca does not publish an explicit
    /// upper bound, so this method only rejects zero; the server may still
    /// reject larger values it does not support.
    ///
    /// # Errors
    ///
    /// Returns [`Error::InvalidTimeFrame`] when `n` is zero.
    pub fn weeks(n: u16) -> Result<Self, Error> {
        Self::checked(n, TimeFrameUnit::Week, 1, u16::MAX, "1..")
    }

    /// Build an `n`-month time frame. Alpaca documents `1Month` and
    /// `3Month` as accepted values; the server may reject other multipliers.
    ///
    /// # Errors
    ///
    /// Returns [`Error::InvalidTimeFrame`] when `n` is zero.
    pub fn months(n: u16) -> Result<Self, Error> {
        Self::checked(n, TimeFrameUnit::Month, 1, u16::MAX, "1..")
    }

    fn checked(
        amount: u16,
        unit: TimeFrameUnit,
        min: u16,
        max: u16,
        valid_range: &'static str,
    ) -> Result<Self, Error> {
        if amount < min || amount > max {
            return Err(Error::InvalidTimeFrame {
                unit,
                amount,
                valid_range,
            });
        }
        Ok(Self { amount, unit })
    }
}

impl fmt::Display for TimeFrame {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}{}", self.amount, self.unit)
    }
}

impl Serialize for TimeFrame {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.collect_str(self)
    }
}

impl<'de> Deserialize<'de> for TimeFrame {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let raw = <&str>::deserialize(deserializer)?;
        let split = raw
            .find(|c: char| !c.is_ascii_digit())
            .ok_or_else(|| serde::de::Error::custom(format!("invalid timeframe: {raw}")))?;
        if split == 0 {
            return Err(serde::de::Error::custom(format!(
                "invalid timeframe: {raw} (missing multiplier)",
            )));
        }
        let (amount_str, unit_str) = raw.split_at(split);
        let amount: u16 = amount_str
            .parse()
            .map_err(|_| serde::de::Error::custom(format!("invalid timeframe amount: {raw}")))?;
        let unit = match unit_str {
            "Min" => TimeFrameUnit::Minute,
            "Hour" => TimeFrameUnit::Hour,
            "Day" => TimeFrameUnit::Day,
            "Week" => TimeFrameUnit::Week,
            "Month" => TimeFrameUnit::Month,
            other => {
                return Err(serde::de::Error::custom(format!(
                    "invalid timeframe unit: {other}",
                )));
            }
        };
        Ok(Self { amount, unit })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn named_constants_match_legacy_wire_strings() {
        for (tf, wire) in [
            (TimeFrame::ONE_MINUTE, "1Min"),
            (TimeFrame::FIVE_MINUTES, "5Min"),
            (TimeFrame::FIFTEEN_MINUTES, "15Min"),
            (TimeFrame::THIRTY_MINUTES, "30Min"),
            (TimeFrame::ONE_HOUR, "1Hour"),
            (TimeFrame::TWO_HOURS, "2Hour"),
            (TimeFrame::FOUR_HOURS, "4Hour"),
            (TimeFrame::ONE_DAY, "1Day"),
            (TimeFrame::ONE_WEEK, "1Week"),
            (TimeFrame::ONE_MONTH, "1Month"),
            (TimeFrame::THREE_MONTHS, "3Month"),
        ] {
            assert_eq!(tf.to_string(), wire);
            assert_eq!(serde_json::to_string(&tf).unwrap(), format!("\"{wire}\""));
        }
    }

    #[test]
    fn arbitrary_multipliers_serialize_to_alpaca_wire_format() {
        assert_eq!(TimeFrame::minutes(7).unwrap().to_string(), "7Min");
        assert_eq!(TimeFrame::hours(3).unwrap().to_string(), "3Hour");
        assert_eq!(TimeFrame::days(45).unwrap().to_string(), "45Day");
    }

    #[test]
    fn out_of_range_multipliers_are_rejected() {
        assert!(matches!(
            TimeFrame::minutes(0),
            Err(Error::InvalidTimeFrame { .. })
        ));
        assert!(matches!(
            TimeFrame::minutes(60),
            Err(Error::InvalidTimeFrame { .. })
        ));
        assert!(matches!(
            TimeFrame::hours(0),
            Err(Error::InvalidTimeFrame { .. })
        ));
        assert!(matches!(
            TimeFrame::hours(24),
            Err(Error::InvalidTimeFrame { .. })
        ));
        assert!(matches!(
            TimeFrame::days(0),
            Err(Error::InvalidTimeFrame { .. })
        ));
        assert!(matches!(
            TimeFrame::days(367),
            Err(Error::InvalidTimeFrame { .. })
        ));
        assert!(matches!(
            TimeFrame::weeks(0),
            Err(Error::InvalidTimeFrame { .. })
        ));
        assert!(matches!(
            TimeFrame::months(0),
            Err(Error::InvalidTimeFrame { .. })
        ));
    }

    #[test]
    fn round_trips_through_serde() {
        for tf in [
            TimeFrame::ONE_MINUTE,
            TimeFrame::THREE_MONTHS,
            TimeFrame::days(45).unwrap(),
            TimeFrame::weeks(2).unwrap(),
        ] {
            let json = serde_json::to_string(&tf).unwrap();
            let parsed: TimeFrame = serde_json::from_str(&json).unwrap();
            assert_eq!(parsed, tf);
        }
    }

    #[test]
    fn deserialize_rejects_malformed_inputs() {
        for raw in ["", "Min", "5", "5Foo", "abcMin"] {
            let json = format!("\"{raw}\"");
            assert!(
                serde_json::from_str::<TimeFrame>(&json).is_err(),
                "expected `{raw}` to fail to deserialize"
            );
        }
    }
}
