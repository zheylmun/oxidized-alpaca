pub mod bars;
pub mod orderbooks;
pub mod quotes;
pub mod snapshots;
pub mod trades;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Crypto exchange location.
#[derive(Clone, Copy, Debug, Serialize, PartialEq, Eq)]
#[non_exhaustive]
pub enum CryptoLocation {
    /// US exchanges
    #[serde(rename = "us")]
    Us,
}

impl CryptoLocation {
    pub(crate) fn as_str(&self) -> &str {
        match self {
            Self::Us => "us",
        }
    }
}

/// A crypto bar (OHLCV).
#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct CryptoBar {
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
    pub volume: f64,
    #[serde(rename = "n")]
    pub trade_count: i64,
    #[serde(rename = "vw")]
    pub vwap: f64,
}

/// A crypto trade.
#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct CryptoTrade {
    #[serde(rename = "t")]
    pub timestamp: DateTime<Utc>,
    #[serde(rename = "p")]
    pub price: f64,
    #[serde(rename = "s")]
    pub size: f64,
    #[serde(rename = "i", default)]
    pub trade_id: Option<i64>,
    #[serde(rename = "tks", default)]
    pub taker_side: Option<String>,
}

/// A crypto quote.
#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct CryptoQuote {
    #[serde(rename = "t")]
    pub timestamp: DateTime<Utc>,
    #[serde(rename = "bp")]
    pub bid_price: f64,
    #[serde(rename = "bs")]
    pub bid_size: f64,
    #[serde(rename = "ap")]
    pub ask_price: f64,
    #[serde(rename = "as")]
    pub ask_size: f64,
}

/// A crypto snapshot.
#[derive(Clone, Debug, Deserialize)]
pub struct CryptoSnapshot {
    #[serde(rename = "latestTrade")]
    pub latest_trade: Option<CryptoTrade>,
    #[serde(rename = "latestQuote")]
    pub latest_quote: Option<CryptoQuote>,
    #[serde(rename = "minuteBar")]
    pub minute_bar: Option<CryptoBar>,
    #[serde(rename = "dailyBar")]
    pub daily_bar: Option<CryptoBar>,
    #[serde(rename = "prevDailyBar")]
    pub prev_daily_bar: Option<CryptoBar>,
}

/// An orderbook entry.
#[derive(Clone, Debug, Deserialize)]
pub struct OrderbookEntry {
    #[serde(rename = "p")]
    pub price: f64,
    #[serde(rename = "s")]
    pub size: f64,
}

/// A crypto orderbook.
#[derive(Clone, Debug, Deserialize)]
pub struct CryptoOrderbook {
    #[serde(rename = "t")]
    pub timestamp: DateTime<Utc>,
    #[serde(rename = "b")]
    pub bids: Vec<OrderbookEntry>,
    #[serde(rename = "a")]
    pub asks: Vec<OrderbookEntry>,
}
