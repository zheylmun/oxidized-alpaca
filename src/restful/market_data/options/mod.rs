/// Option bars endpoint types and methods.
pub mod bars;
/// Option quotes endpoint types and methods.
pub mod quotes;
/// Option snapshots endpoint types and methods.
pub mod snapshots;
/// Option trades endpoint types and methods.
pub mod trades;

use chrono::{DateTime, Utc};
use serde::Deserialize;

/// An option bar (OHLCV).
#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct OptionBar {
    /// The bar timestamp.
    #[serde(rename = "t")]
    pub timestamp: DateTime<Utc>,
    /// The open price.
    #[serde(rename = "o")]
    pub open: f64,
    /// The highest price.
    #[serde(rename = "h")]
    pub high: f64,
    /// The lowest price.
    #[serde(rename = "l")]
    pub low: f64,
    /// The close price.
    #[serde(rename = "c")]
    pub close: f64,
    /// The trading volume.
    #[serde(rename = "v")]
    pub volume: i64,
    /// The number of trades.
    #[serde(rename = "n")]
    pub trade_count: i64,
    /// The volume-weighted average price.
    #[serde(rename = "vw")]
    pub vwap: f64,
}

/// An option trade.
#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct OptionTrade {
    /// The trade timestamp.
    #[serde(rename = "t")]
    pub timestamp: DateTime<Utc>,
    /// The exchange code.
    #[serde(rename = "x")]
    pub exchange: String,
    /// The trade price.
    #[serde(rename = "p")]
    pub price: f64,
    /// The trade size.
    #[serde(rename = "s")]
    pub size: u32,
    /// The trade condition.
    #[serde(rename = "c", default)]
    pub condition: Option<String>,
}

/// An option quote.
#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct OptionQuote {
    /// The quote timestamp.
    #[serde(rename = "t")]
    pub timestamp: DateTime<Utc>,
    /// The bid exchange code.
    #[serde(rename = "bx")]
    pub bid_exchange: String,
    /// The bid price.
    #[serde(rename = "bp")]
    pub bid_price: f64,
    /// The bid size.
    #[serde(rename = "bs")]
    pub bid_size: u32,
    /// The ask exchange code.
    #[serde(rename = "ax")]
    pub ask_exchange: String,
    /// The ask price.
    #[serde(rename = "ap")]
    pub ask_price: f64,
    /// The ask size.
    #[serde(rename = "as")]
    pub ask_size: u32,
    /// The quote condition.
    #[serde(rename = "c", default)]
    pub condition: Option<String>,
}

/// Option greeks.
#[derive(Clone, Debug, Deserialize)]
pub struct OptionGreeks {
    /// Delta value.
    pub delta: Option<f64>,
    /// Gamma value.
    pub gamma: Option<f64>,
    /// Theta value.
    pub theta: Option<f64>,
    /// Vega value.
    pub vega: Option<f64>,
    /// Rho value.
    pub rho: Option<f64>,
}

/// An option snapshot.
#[derive(Clone, Debug, Deserialize)]
pub struct OptionSnapshot {
    /// The latest trade.
    #[serde(rename = "latestTrade")]
    pub latest_trade: Option<OptionTrade>,
    /// The latest quote.
    #[serde(rename = "latestQuote")]
    pub latest_quote: Option<OptionQuote>,
    /// The option greeks.
    #[serde(default)]
    pub greeks: Option<OptionGreeks>,
    /// The implied volatility.
    #[serde(rename = "impliedVolatility", default)]
    pub implied_volatility: Option<f64>,
}
