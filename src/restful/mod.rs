/// Market data endpoint types and methods.
pub mod market_data;
mod market_data_client;
pub use market_data_client::MarketDataClient;
/// Trading endpoint types and methods.
pub mod trading;
mod trading_client;
pub use trading_client::TradingClient;

use serde::{Deserialize, Serialize};

pub(crate) use crate::serde_helpers::{
    decimal_as_string, null_def_vec, optional_decimal_as_string, string_as_decimal,
    string_as_optional_decimal, string_as_optional_u64,
};

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
