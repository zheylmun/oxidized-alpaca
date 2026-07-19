use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::streaming::wire::StreamError;

/// Subscriptions for the crypto streaming feed.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct CryptoSubscriptionList {
    /// Symbols subscribed to minute bars.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bars: Option<Vec<String>>,
    /// Symbols subscribed to daily bars.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub daily_bars: Option<Vec<String>>,
    /// Symbols subscribed to updated bars.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_bars: Option<Vec<String>>,
    /// Symbols subscribed to quotes.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quotes: Option<Vec<String>>,
    /// Symbols subscribed to trades.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trades: Option<Vec<String>>,
    /// Symbols subscribed to orderbook updates.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub orderbooks: Option<Vec<String>>,
}

impl CryptoSubscriptionList {
    /// Create an empty subscription list.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Subscribe to minute bars for `symbol`.
    #[must_use]
    pub fn add_minute_bars(self, symbol: &str) -> Self {
        Self {
            bars: Some(append_unique(self.bars, symbol)),
            ..self
        }
    }

    /// Subscribe to daily bars for `symbol`.
    #[must_use]
    pub fn add_daily_bars(self, symbol: &str) -> Self {
        Self {
            daily_bars: Some(append_unique(self.daily_bars, symbol)),
            ..self
        }
    }

    /// Subscribe to updated bars for `symbol`.
    #[must_use]
    pub fn add_updated_bars(self, symbol: &str) -> Self {
        Self {
            updated_bars: Some(append_unique(self.updated_bars, symbol)),
            ..self
        }
    }

    /// Subscribe to quotes for `symbol`.
    #[must_use]
    pub fn add_quotes(self, symbol: &str) -> Self {
        Self {
            quotes: Some(append_unique(self.quotes, symbol)),
            ..self
        }
    }

    /// Subscribe to trades for `symbol`.
    #[must_use]
    pub fn add_trades(self, symbol: &str) -> Self {
        Self {
            trades: Some(append_unique(self.trades, symbol)),
            ..self
        }
    }

    /// Subscribe to orderbook updates for `symbol`.
    #[must_use]
    pub fn add_orderbooks(self, symbol: &str) -> Self {
        Self {
            orderbooks: Some(append_unique(self.orderbooks, symbol)),
            ..self
        }
    }
}

fn append_unique(list: Option<Vec<String>>, symbol: &str) -> Vec<String> {
    let mut list = list.unwrap_or_default();
    if !list.iter().any(|s| s == symbol) {
        list.push(symbol.to_string());
    }
    list
}

pub use crate::crypto::CryptoTakerSide;

/// OHLCV bar for a crypto pair.
///
/// `vwap` and `trade_count` are populated for minute and daily bars; updated
/// bars omit them.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct CryptoBarEvent {
    /// Trading pair symbol (e.g. `BTC/USD`).
    #[serde(rename = "S")]
    pub symbol: String,
    /// Opening price.
    #[serde(rename = "o")]
    pub open: f64,
    /// High price.
    #[serde(rename = "h")]
    pub high: f64,
    /// Low price.
    #[serde(rename = "l")]
    pub low: f64,
    /// Closing price.
    #[serde(rename = "c")]
    pub close: f64,
    /// Trade volume in base units.
    #[serde(rename = "v")]
    pub volume: f64,
    /// Volume-weighted average price (minute / daily bars only).
    #[serde(rename = "vw")]
    pub vwap: Option<f64>,
    /// Number of trades aggregated (minute / daily bars only).
    #[serde(rename = "n")]
    pub trade_count: Option<i64>,
    /// Bar timestamp.
    #[serde(rename = "t")]
    pub timestamp: DateTime<Utc>,
}

/// Real-time crypto quote with bid and ask.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct CryptoQuoteEvent {
    /// Trading pair symbol.
    #[serde(rename = "S")]
    pub symbol: String,
    /// Bid price.
    #[serde(rename = "bp")]
    pub bid_price: f64,
    /// Bid size.
    #[serde(rename = "bs")]
    pub bid_size: f64,
    /// Ask price.
    #[serde(rename = "ap")]
    pub ask_price: f64,
    /// Ask size.
    #[serde(rename = "as")]
    pub ask_size: f64,
    /// Quote timestamp.
    #[serde(rename = "t")]
    pub timestamp: DateTime<Utc>,
}

/// Real-time crypto trade event.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct CryptoTradeEvent {
    /// Trading pair symbol.
    #[serde(rename = "S")]
    pub symbol: String,
    /// Trade ID.
    #[serde(rename = "i")]
    pub trade_id: i64,
    /// Trade price.
    #[serde(rename = "p")]
    pub price: f64,
    /// Trade size in base units.
    #[serde(rename = "s")]
    pub size: f64,
    /// Side that initiated the trade.
    #[serde(rename = "tks")]
    pub taker_side: Option<CryptoTakerSide>,
    /// Trade timestamp.
    #[serde(rename = "t")]
    pub timestamp: DateTime<Utc>,
}

/// One side of an orderbook level.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct CryptoBookLevel {
    /// Price for this level.
    #[serde(rename = "p")]
    pub price: f64,
    /// Size at this level.
    #[serde(rename = "s")]
    pub size: f64,
}

/// Crypto orderbook update. When `reset` is `true` the bids/asks represent the
/// full book; otherwise they are an incremental update.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct CryptoOrderbookEvent {
    /// Trading pair symbol.
    #[serde(rename = "S")]
    pub symbol: String,
    /// Update timestamp.
    #[serde(rename = "t")]
    pub timestamp: DateTime<Utc>,
    /// Bid levels.
    #[serde(rename = "b")]
    pub bids: Vec<CryptoBookLevel>,
    /// Ask levels.
    #[serde(rename = "a")]
    pub asks: Vec<CryptoBookLevel>,
    /// `true` when this update is a full snapshot rather than an increment.
    #[serde(rename = "r", default)]
    pub reset: bool,
}

/// Messages received from the crypto streaming feed.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "T")]
#[non_exhaustive]
pub enum CryptoStreamMessage {
    /// Internally consumed stream acknowledging successful completion of requests.
    #[serde(rename = "success")]
    Control {
        /// The control message payload.
        msg: crate::streaming::wire::ControlMessage,
    },
    /// Error message from the server.
    #[serde(rename = "error")]
    Error(StreamError),
    /// Subscription confirmation with the current subscription list.
    #[serde(rename = "subscription")]
    Subscription(CryptoSubscriptionList),
    /// Minute bar update.
    #[serde(rename = "b")]
    Bar(CryptoBarEvent),
    /// Daily bar update.
    #[serde(rename = "d")]
    DailyBar(CryptoBarEvent),
    /// Updated bar.
    #[serde(rename = "u")]
    UpdatedBar(CryptoBarEvent),
    /// Trade event.
    #[serde(rename = "t")]
    Trade(CryptoTradeEvent),
    /// Quote update.
    #[serde(rename = "q")]
    Quote(CryptoQuoteEvent),
    /// Orderbook update.
    #[serde(rename = "o")]
    Orderbook(CryptoOrderbookEvent),
}

impl CryptoStreamMessage {
    pub(crate) const fn control(&self) -> Option<&crate::streaming::wire::ControlMessage> {
        match self {
            CryptoStreamMessage::Control { msg } => Some(msg),
            _ => None,
        }
    }

    pub(crate) const fn stream_error(&self) -> Option<&StreamError> {
        match self {
            CryptoStreamMessage::Error(err) => Some(err),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserializes_minute_bar() {
        let json = r#"{"T":"b","S":"BTC/USD","o":50000.0,"h":50100.0,"l":49900.0,"c":50050.0,"v":12.5,"vw":50025.0,"n":42,"t":"2024-01-02T15:30:00Z"}"#;
        match serde_json::from_str(json).unwrap() {
            CryptoStreamMessage::Bar(bar) => {
                assert_eq!(bar.symbol, "BTC/USD");
                assert_eq!(bar.vwap, Some(50025.0));
            }
            other => panic!("expected Bar, got {other:?}"),
        }
    }

    #[test]
    fn deserializes_updated_bar_without_vw_or_n() {
        let json = r#"{"T":"u","S":"BTC/USD","o":50000.0,"h":50100.0,"l":49900.0,"c":50050.0,"v":12.5,"t":"2024-01-02T15:30:00Z"}"#;
        match serde_json::from_str(json).unwrap() {
            CryptoStreamMessage::UpdatedBar(bar) => {
                assert!(bar.vwap.is_none());
                assert!(bar.trade_count.is_none());
            }
            other => panic!("expected UpdatedBar, got {other:?}"),
        }
    }

    #[test]
    fn deserializes_trade_with_taker_side() {
        let json = r#"{"T":"t","S":"BTC/USD","i":12345,"p":50050.0,"s":0.5,"tks":"B","t":"2024-01-02T15:30:00Z"}"#;
        match serde_json::from_str(json).unwrap() {
            CryptoStreamMessage::Trade(trade) => {
                assert_eq!(trade.taker_side, Some(CryptoTakerSide::Buyer));
            }
            other => panic!("expected Trade, got {other:?}"),
        }
    }

    #[test]
    fn deserializes_quote() {
        let json = r#"{"T":"q","S":"BTC/USD","bp":50000.0,"bs":1.0,"ap":50100.0,"as":2.0,"t":"2024-01-02T15:30:00Z"}"#;
        match serde_json::from_str(json).unwrap() {
            CryptoStreamMessage::Quote(quote) => {
                assert_eq!(quote.bid_price, 50000.0);
            }
            other => panic!("expected Quote, got {other:?}"),
        }
    }

    #[test]
    fn deserializes_orderbook_snapshot() {
        let json = r#"{"T":"o","S":"BTC/USD","t":"2024-01-02T15:30:00Z","b":[{"p":50000.0,"s":1.0}],"a":[{"p":50100.0,"s":2.0}],"r":true}"#;
        match serde_json::from_str(json).unwrap() {
            CryptoStreamMessage::Orderbook(book) => {
                assert!(book.reset);
                assert_eq!(book.bids.len(), 1);
                assert_eq!(book.asks.len(), 1);
            }
            other => panic!("expected Orderbook, got {other:?}"),
        }
    }

    #[test]
    fn deserializes_orderbook_increment_without_reset() {
        let json = r#"{"T":"o","S":"BTC/USD","t":"2024-01-02T15:30:00Z","b":[],"a":[]}"#;
        match serde_json::from_str(json).unwrap() {
            CryptoStreamMessage::Orderbook(book) => assert!(!book.reset),
            other => panic!("expected Orderbook, got {other:?}"),
        }
    }
}
