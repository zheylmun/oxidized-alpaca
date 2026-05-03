/// Market data endpoint types and methods.
pub mod market_data;
mod market_data_client;
pub use market_data_client::MarketDataClient;
/// Trading endpoint types and methods.
pub mod trading;
mod trading_client;
pub use trading_client::TradingClient;

use rust_decimal::Decimal;
use serde::{Deserialize, Deserializer, Serialize};

/// Sort direction shared across endpoints that accept ordering hints.
#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
#[non_exhaustive]
pub enum SortDirection {
    /// Ascending order (oldest first).
    Asc,
    /// Descending order (newest first).
    Desc,
}

pub(crate) fn null_def_vec<'de, D, T>(deserializer: D) -> Result<Vec<T>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
{
    let opt = Option::<Vec<T>>::deserialize(deserializer)?;
    Ok(opt.unwrap_or_default())
}

pub(crate) fn string_as_decimal<'de, D>(deserializer: D) -> Result<Decimal, D::Error>
where
    D: Deserializer<'de>,
{
    String::deserialize(deserializer)?
        .parse()
        .map_err(serde::de::Error::custom)
}

pub(crate) fn string_as_optional_decimal<'de, D>(
    deserializer: D,
) -> Result<Option<Decimal>, D::Error>
where
    D: Deserializer<'de>,
{
    let opt = Option::<String>::deserialize(deserializer)?;
    match opt {
        Some(s) => s.parse().map(Some).map_err(serde::de::Error::custom),
        None => Ok(None),
    }
}

pub(crate) fn optional_decimal_as_string<S>(
    value: &Option<Decimal>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    match value {
        Some(d) => serializer.serialize_str(&d.to_string()),
        None => serializer.serialize_none(),
    }
}

pub(crate) fn decimal_as_string<S>(value: &Decimal, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    serializer.serialize_str(&value.to_string())
}
