pub mod bars;
pub mod quotes;
pub mod snapshots;
pub mod trades;

use chrono::{DateTime, Utc};
use serde::Deserialize;

/// An option bar (OHLCV).
#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct OptionBar {
    #[serde(rename = "t")]
    pub timestamp: DateTime<Utc>,
    #[serde(rename = "o")]
    pub open: f64,
    #[serde(rename = "h")]
    pub high: f64,
    #[serde(rename = "l")]
    pub low: f64,
    #[serde(rename = "c")]
    pub close: f64,
    #[serde(rename = "v")]
    pub volume: i64,
    #[serde(rename = "n")]
    pub trade_count: i64,
    #[serde(rename = "vw")]
    pub vwap: f64,
}

/// An option trade.
#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct OptionTrade {
    #[serde(rename = "t")]
    pub timestamp: DateTime<Utc>,
    #[serde(rename = "x")]
    pub exchange: String,
    #[serde(rename = "p")]
    pub price: f64,
    #[serde(rename = "s")]
    pub size: u32,
    #[serde(rename = "c", default)]
    pub condition: Option<String>,
}

/// An option quote.
#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct OptionQuote {
    #[serde(rename = "t")]
    pub timestamp: DateTime<Utc>,
    #[serde(rename = "bx")]
    pub bid_exchange: String,
    #[serde(rename = "bp")]
    pub bid_price: f64,
    #[serde(rename = "bs")]
    pub bid_size: u32,
    #[serde(rename = "ax")]
    pub ask_exchange: String,
    #[serde(rename = "ap")]
    pub ask_price: f64,
    #[serde(rename = "as")]
    pub ask_size: u32,
    #[serde(rename = "c", default)]
    pub condition: Option<String>,
}

/// Option greeks.
#[derive(Clone, Debug, Deserialize)]
pub struct OptionGreeks {
    pub delta: Option<f64>,
    pub gamma: Option<f64>,
    pub theta: Option<f64>,
    pub vega: Option<f64>,
    pub rho: Option<f64>,
}

/// An option snapshot.
#[derive(Clone, Debug, Deserialize)]
pub struct OptionSnapshot {
    #[serde(rename = "latestTrade")]
    pub latest_trade: Option<OptionTrade>,
    #[serde(rename = "latestQuote")]
    pub latest_quote: Option<OptionQuote>,
    #[serde(default)]
    pub greeks: Option<OptionGreeks>,
    #[serde(rename = "impliedVolatility", default)]
    pub implied_volatility: Option<f64>,
}
