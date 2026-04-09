pub mod market_data;
mod market_data_client;
pub use market_data_client::MarketDataClient;
pub mod trading;
mod trading_client;
pub use trading_client::TradingClient;

use rust_decimal::Decimal;
use serde::{Deserialize, Deserializer};

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
    Ok(Some(
        String::deserialize(deserializer)?
            .parse()
            .map_err(serde::de::Error::custom)?,
    ))
}
