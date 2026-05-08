/// Crypto bars endpoint types and methods.
pub mod bars;
/// Crypto orderbooks endpoint types and methods.
pub mod orderbooks;
/// Crypto quotes endpoint types and methods.
pub mod quotes;
/// Crypto snapshots endpoint types and methods.
pub mod snapshots;
/// Crypto trades endpoint types and methods.
pub mod trades;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Crypto exchange location.
#[derive(Clone, Copy, Debug, Serialize, PartialEq, Eq)]
#[non_exhaustive]
pub enum CryptoLocation {
    /// US exchanges (default Alpaca-aggregated feed).
    #[serde(rename = "us")]
    Us,
    /// US-1 feed (Kraken-backed US data).
    #[serde(rename = "us-1")]
    Us1,
    /// US-2 feed.
    #[serde(rename = "us-2")]
    Us2,
    /// EU-1 feed.
    #[serde(rename = "eu-1")]
    Eu1,
    /// BS-1 feed (Bahamas).
    #[serde(rename = "bs-1")]
    Bs1,
}

impl CryptoLocation {
    fn as_str(&self) -> &'static str {
        match self {
            Self::Us => "us",
            Self::Us1 => "us-1",
            Self::Us2 => "us-2",
            Self::Eu1 => "eu-1",
            Self::Bs1 => "bs-1",
        }
    }
}

impl std::fmt::Display for CryptoLocation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A crypto bar (OHLCV).
#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct CryptoBar {
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
    pub volume: f64,
    /// The number of trades.
    #[serde(rename = "n")]
    pub trade_count: i64,
    /// The volume-weighted average price.
    #[serde(rename = "vw")]
    pub vwap: f64,
}

/// A crypto trade.
#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct CryptoTrade {
    /// The trade timestamp.
    #[serde(rename = "t")]
    pub timestamp: DateTime<Utc>,
    /// The trade price.
    #[serde(rename = "p")]
    pub price: f64,
    /// The trade size.
    #[serde(rename = "s")]
    pub size: f64,
    /// The trade ID.
    #[serde(rename = "i", default)]
    pub trade_id: Option<i64>,
    /// The taker side (buy or sell).
    #[serde(rename = "tks", default)]
    pub taker_side: Option<String>,
}

/// A crypto quote.
#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct CryptoQuote {
    /// The quote timestamp.
    #[serde(rename = "t")]
    pub timestamp: DateTime<Utc>,
    /// The bid price.
    #[serde(rename = "bp")]
    pub bid_price: f64,
    /// The bid size.
    #[serde(rename = "bs")]
    pub bid_size: f64,
    /// The ask price.
    #[serde(rename = "ap")]
    pub ask_price: f64,
    /// The ask size.
    #[serde(rename = "as")]
    pub ask_size: f64,
}

/// A crypto snapshot.
#[derive(Clone, Debug, Deserialize)]
pub struct CryptoSnapshot {
    /// The latest trade.
    #[serde(rename = "latestTrade")]
    pub latest_trade: Option<CryptoTrade>,
    /// The latest quote.
    #[serde(rename = "latestQuote")]
    pub latest_quote: Option<CryptoQuote>,
    /// The latest minute bar.
    #[serde(rename = "minuteBar")]
    pub minute_bar: Option<CryptoBar>,
    /// The current daily bar.
    #[serde(rename = "dailyBar")]
    pub daily_bar: Option<CryptoBar>,
    /// The previous day's daily bar.
    #[serde(rename = "prevDailyBar")]
    pub prev_daily_bar: Option<CryptoBar>,
}

/// An orderbook entry.
#[derive(Clone, Debug, Deserialize)]
pub struct OrderbookEntry {
    /// The price level.
    #[serde(rename = "p")]
    pub price: f64,
    /// The size at this price level.
    #[serde(rename = "s")]
    pub size: f64,
}

/// A crypto orderbook.
#[derive(Clone, Debug, Deserialize)]
pub struct CryptoOrderbook {
    /// The orderbook timestamp.
    #[serde(rename = "t")]
    pub timestamp: DateTime<Utc>,
    /// The bid entries.
    #[serde(rename = "b")]
    pub bids: Vec<OrderbookEntry>,
    /// The ask entries.
    #[serde(rename = "a")]
    pub asks: Vec<OrderbookEntry>,
}

#[cfg(test)]
mod tests {
    use super::CryptoLocation;

    #[test]
    fn crypto_location_renders_path_segment() {
        assert_eq!(CryptoLocation::Us.to_string(), "us");
        assert_eq!(CryptoLocation::Us1.to_string(), "us-1");
        assert_eq!(CryptoLocation::Us2.to_string(), "us-2");
        assert_eq!(CryptoLocation::Eu1.to_string(), "eu-1");
        assert_eq!(CryptoLocation::Bs1.to_string(), "bs-1");
    }

    #[test]
    fn crypto_location_serializes_as_path_segment() {
        for (loc, expected) in [
            (CryptoLocation::Us, "\"us\""),
            (CryptoLocation::Us1, "\"us-1\""),
            (CryptoLocation::Us2, "\"us-2\""),
            (CryptoLocation::Eu1, "\"eu-1\""),
            (CryptoLocation::Bs1, "\"bs-1\""),
        ] {
            assert_eq!(serde_json::to_string(&loc).unwrap(), expected);
        }
    }
}
