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

use crate::crypto::CryptoTakerSide;

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
#[non_exhaustive]
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
#[non_exhaustive]
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
    /// The trade ID. Some feeds encode this as a JSON string rather than a
    /// number; both forms deserialize here.
    #[serde(
        rename = "i",
        default,
        deserialize_with = "crate::serde_helpers::string_or_int_as_optional_i64"
    )]
    pub trade_id: Option<i64>,
    /// The side that initiated the trade.
    #[serde(rename = "tks", default)]
    pub taker_side: Option<CryptoTakerSide>,
}

/// A crypto quote.
#[derive(Clone, Debug, Deserialize, PartialEq)]
#[non_exhaustive]
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
#[non_exhaustive]
pub struct CryptoSnapshot {
    /// The latest trade.
    #[serde(rename = "latestTrade", default)]
    pub latest_trade: Option<CryptoTrade>,
    /// The latest quote.
    #[serde(rename = "latestQuote", default)]
    pub latest_quote: Option<CryptoQuote>,
    /// The latest minute bar.
    #[serde(rename = "minuteBar", default)]
    pub minute_bar: Option<CryptoBar>,
    /// The current daily bar.
    #[serde(rename = "dailyBar", default)]
    pub daily_bar: Option<CryptoBar>,
    /// The previous day's daily bar.
    #[serde(rename = "prevDailyBar", default)]
    pub prev_daily_bar: Option<CryptoBar>,
}

/// An orderbook entry.
#[derive(Clone, Debug, Deserialize)]
#[non_exhaustive]
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
#[non_exhaustive]
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
    /// `true` when this book is a full snapshot rather than an increment.
    #[serde(rename = "r", default)]
    pub reset: bool,
}

#[cfg(test)]
mod tests {
    use super::{CryptoBar, CryptoLocation, CryptoQuote, CryptoTrade};
    use crate::CryptoTakerSide;

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

    #[test]
    fn deserializes_bar_payload() {
        let json = r#"{"t":"2026-05-07T13:29:00Z","o":103200.0,"h":103260.0,"l":103190.0,"c":103250.0,"v":12.5,"n":420,"vw":103225.0}"#;
        let bar: CryptoBar = serde_json::from_str(json).unwrap();
        assert_eq!(bar.open, 103_200.0);
        assert_eq!(bar.high, 103_260.0);
        assert_eq!(bar.low, 103_190.0);
        assert_eq!(bar.close, 103_250.0);
        assert_eq!(bar.volume, 12.5);
        assert_eq!(bar.trade_count, 420);
        assert_eq!(bar.vwap, 103_225.0);
    }

    #[test]
    fn deserializes_quote_payload() {
        let json = r#"{"t":"2026-05-07T13:30:00Z","bp":103250.0,"bs":0.5,"ap":103251.0,"as":0.4}"#;
        let quote: CryptoQuote = serde_json::from_str(json).unwrap();
        assert_eq!(quote.bid_price, 103_250.0);
        assert_eq!(quote.bid_size, 0.5);
        assert_eq!(quote.ask_price, 103_251.0);
        assert_eq!(quote.ask_size, 0.4);
    }

    #[test]
    fn deserializes_trade_payload_with_taker_side() {
        let json = r#"{"t":"2026-05-07T13:30:00Z","p":103250.5,"s":0.014,"i":12345,"tks":"B"}"#;
        let trade: CryptoTrade = serde_json::from_str(json).unwrap();
        assert_eq!(trade.price, 103_250.5);
        assert_eq!(trade.size, 0.014);
        assert_eq!(trade.trade_id, Some(12345));
        assert_eq!(trade.taker_side, Some(CryptoTakerSide::Buyer));
    }

    /// Some crypto venues emit the `i` trade id as a JSON string rather
    /// than a number; both encodings must land on the same `i64`.
    #[test]
    fn deserializes_trade_id_from_string_or_number() {
        let as_int = r#"{"t":"2026-05-07T13:30:00Z","p":1.0,"s":1.0,"i":98765}"#;
        let as_str = r#"{"t":"2026-05-07T13:30:00Z","p":1.0,"s":1.0,"i":"98765"}"#;
        let from_int: CryptoTrade = serde_json::from_str(as_int).unwrap();
        let from_str: CryptoTrade = serde_json::from_str(as_str).unwrap();
        assert_eq!(from_int.trade_id, Some(98765));
        assert_eq!(from_str.trade_id, Some(98765));
    }

    #[test]
    fn deserializes_trade_without_optional_fields() {
        let json = r#"{"t":"2026-05-07T13:30:00Z","p":1.0,"s":1.0}"#;
        let trade: CryptoTrade = serde_json::from_str(json).unwrap();
        assert!(trade.trade_id.is_none());
        assert!(trade.taker_side.is_none());
    }
}
