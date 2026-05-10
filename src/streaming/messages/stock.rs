use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::streaming::wire::StreamError;

/// Subscriptions for the stock streaming feed.
///
/// Note: trade corrections (`c`) and trade cancellations / errata (`x`) are
/// delivered automatically when you subscribe to `trades` and have no
/// dedicated subscription channel of their own.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct StockSubscriptionList {
    /// Symbols subscribed to minute bars.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bars: Option<Vec<String>>,
    /// Symbols subscribed to daily bars.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub daily_bars: Option<Vec<String>>,
    /// Symbols subscribed to updated bars (late-trade corrections).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_bars: Option<Vec<String>>,
    /// Symbols subscribed to quotes.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quotes: Option<Vec<String>>,
    /// Symbols subscribed to trades. Trade corrections and cancellations/errata
    /// are delivered automatically alongside this channel.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trades: Option<Vec<String>>,
    /// Symbols subscribed to trading status updates.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub statuses: Option<Vec<String>>,
    /// Symbols subscribed to LULD (Limit Up–Limit Down) band events.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lulds: Option<Vec<String>>,
    /// Symbols subscribed to order imbalance events.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub imbalances: Option<Vec<String>>,
}

impl StockSubscriptionList {
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

    /// Subscribe to trades for `symbol`. Trade corrections and cancellations
    /// are delivered automatically once you subscribe to trades.
    #[must_use]
    pub fn add_trades(self, symbol: &str) -> Self {
        Self {
            trades: Some(append_unique(self.trades, symbol)),
            ..self
        }
    }

    /// Subscribe to trading status updates for `symbol`.
    #[must_use]
    pub fn add_statuses(self, symbol: &str) -> Self {
        Self {
            statuses: Some(append_unique(self.statuses, symbol)),
            ..self
        }
    }

    /// Subscribe to LULD band events for `symbol`.
    #[must_use]
    pub fn add_lulds(self, symbol: &str) -> Self {
        Self {
            lulds: Some(append_unique(self.lulds, symbol)),
            ..self
        }
    }

    /// Subscribe to order imbalance events for `symbol`.
    #[must_use]
    pub fn add_imbalances(self, symbol: &str) -> Self {
        Self {
            imbalances: Some(append_unique(self.imbalances, symbol)),
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

/// OHLCV bar for a stock symbol.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct StockBar {
    /// Ticker symbol.
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
    /// Trade volume (shares).
    #[serde(rename = "v")]
    pub volume: i64,
    /// Volume-weighted average price.
    #[serde(rename = "vw")]
    pub vwap: Option<f64>,
    /// Number of trades aggregated into this bar.
    #[serde(rename = "n")]
    pub trade_count: Option<i64>,
    /// Bar timestamp.
    #[serde(rename = "t")]
    pub timestamp: DateTime<Utc>,
}

/// Real-time quote with bid and ask data.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct StockQuote {
    /// Ticker symbol.
    #[serde(rename = "S")]
    pub symbol: String,
    /// Ask exchange code.
    #[serde(rename = "ax")]
    pub ask_exchange: Option<String>,
    /// Ask price.
    #[serde(rename = "ap")]
    pub ask_price: f64,
    /// Ask size in round lots.
    #[serde(rename = "as")]
    pub ask_size: f64,
    /// Bid exchange code.
    #[serde(rename = "bx")]
    pub bid_exchange: Option<String>,
    /// Bid price.
    #[serde(rename = "bp")]
    pub bid_price: f64,
    /// Bid size in round lots.
    #[serde(rename = "bs")]
    pub bid_size: f64,
    /// Quote condition flags.
    #[serde(rename = "c")]
    pub conditions: Option<Vec<String>>,
    /// Quote timestamp.
    #[serde(rename = "t")]
    pub timestamp: DateTime<Utc>,
    /// Tape identifier.
    #[serde(rename = "z")]
    pub tape: Option<String>,
}

/// Real-time trade event.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct StockTrade {
    /// Ticker symbol.
    #[serde(rename = "S")]
    pub symbol: String,
    /// Trade ID.
    #[serde(rename = "i")]
    pub trade_id: i64,
    /// Exchange code where the trade occurred.
    #[serde(rename = "x")]
    pub exchange: Option<String>,
    /// Trade price.
    #[serde(rename = "p")]
    pub price: f64,
    /// Trade size.
    #[serde(rename = "s")]
    pub size: f64,
    /// Trade timestamp.
    #[serde(rename = "t")]
    pub timestamp: DateTime<Utc>,
    /// Trade condition flags.
    #[serde(rename = "c")]
    pub conditions: Option<Vec<String>>,
    /// Tape identifier.
    #[serde(rename = "z")]
    pub tape: Option<String>,
}

/// Notification that a previously reported trade has been corrected.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct StockTradeCorrection {
    /// Ticker symbol.
    #[serde(rename = "S")]
    pub symbol: String,
    /// Exchange code where the correction was reported.
    #[serde(rename = "x")]
    pub exchange: Option<String>,
    /// Original trade ID.
    #[serde(rename = "oi")]
    pub original_trade_id: i64,
    /// Original trade price.
    #[serde(rename = "op")]
    pub original_price: f64,
    /// Original trade size.
    #[serde(rename = "os")]
    pub original_size: f64,
    /// Original trade condition flags.
    #[serde(rename = "oc")]
    pub original_conditions: Option<Vec<String>>,
    /// Corrected trade ID.
    #[serde(rename = "ci")]
    pub corrected_trade_id: i64,
    /// Corrected trade price.
    #[serde(rename = "cp")]
    pub corrected_price: f64,
    /// Corrected trade size.
    #[serde(rename = "cs")]
    pub corrected_size: f64,
    /// Corrected trade condition flags.
    #[serde(rename = "cc")]
    pub corrected_conditions: Option<Vec<String>>,
    /// Timestamp of the correction.
    #[serde(rename = "t")]
    pub timestamp: DateTime<Utc>,
    /// Tape identifier.
    #[serde(rename = "z")]
    pub tape: Option<String>,
}

/// Whether a previously reported trade was cancelled or erroneously reported.
#[derive(Clone, Debug, Deserialize, Serialize, Eq, PartialEq)]
#[non_exhaustive]
pub enum TradeCancelAction {
    /// The trade was cancelled.
    #[serde(rename = "C")]
    Cancel,
    /// The trade was reported in error.
    #[serde(rename = "E")]
    Error,
}

/// Notification that a previously reported trade was cancelled or erroneous.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct StockTradeCancelError {
    /// Ticker symbol.
    #[serde(rename = "S")]
    pub symbol: String,
    /// Trade ID being cancelled or corrected.
    #[serde(rename = "i")]
    pub trade_id: i64,
    /// Exchange code.
    #[serde(rename = "x")]
    pub exchange: Option<String>,
    /// Trade price.
    #[serde(rename = "p")]
    pub price: f64,
    /// Trade size.
    #[serde(rename = "s")]
    pub size: f64,
    /// Whether the trade was cancelled or reported in error.
    #[serde(rename = "a")]
    pub action: TradeCancelAction,
    /// Timestamp of the cancellation/erratum.
    #[serde(rename = "t")]
    pub timestamp: DateTime<Utc>,
    /// Tape identifier.
    #[serde(rename = "z")]
    pub tape: Option<String>,
}

/// Trading-status update for a symbol (halts, resumes, etc.).
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct StockTradingStatus {
    /// Ticker symbol.
    #[serde(rename = "S")]
    pub symbol: String,
    /// Status code (e.g. `"H"` for halted).
    #[serde(rename = "sc")]
    pub status_code: String,
    /// Human-readable status description.
    #[serde(rename = "sm")]
    pub status_message: String,
    /// Reason code.
    #[serde(rename = "rc")]
    pub reason_code: String,
    /// Human-readable reason description.
    #[serde(rename = "rm")]
    pub reason_message: String,
    /// Timestamp of the status change.
    #[serde(rename = "t")]
    pub timestamp: DateTime<Utc>,
    /// Tape identifier.
    #[serde(rename = "z")]
    pub tape: Option<String>,
}

/// Limit Up–Limit Down price-band update.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct StockLuld {
    /// Ticker symbol.
    #[serde(rename = "S")]
    pub symbol: String,
    /// Upper price-band limit.
    #[serde(rename = "u")]
    pub limit_up: f64,
    /// Lower price-band limit.
    #[serde(rename = "d")]
    pub limit_down: f64,
    /// LULD indicator code.
    #[serde(rename = "i")]
    pub indicator: String,
    /// Timestamp of the band update.
    #[serde(rename = "t")]
    pub timestamp: DateTime<Utc>,
    /// Tape identifier.
    #[serde(rename = "z")]
    pub tape: Option<String>,
}

/// Order imbalance event.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct StockImbalance {
    /// Ticker symbol.
    #[serde(rename = "S")]
    pub symbol: String,
    /// Reference price for the imbalance.
    #[serde(rename = "p")]
    pub price: f64,
    /// Tape identifier.
    #[serde(rename = "z")]
    pub tape: Option<String>,
    /// Timestamp of the imbalance event.
    #[serde(rename = "t")]
    pub timestamp: DateTime<Utc>,
}

/// Messages received from the stock streaming feed.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "T")]
#[non_exhaustive]
pub enum StockStreamMessage {
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
    Subscription(StockSubscriptionList),
    /// Minute bar update.
    #[serde(rename = "b")]
    Bar(StockBar),
    /// Daily bar update.
    #[serde(rename = "d")]
    DailyBar(StockBar),
    /// Updated (late-trade-corrected) bar.
    #[serde(rename = "u")]
    UpdatedBar(StockBar),
    /// Trade event.
    #[serde(rename = "t")]
    Trade(StockTrade),
    /// Quote update.
    #[serde(rename = "q")]
    Quote(StockQuote),
    /// Notification that a previously reported trade was corrected.
    #[serde(rename = "c")]
    Correction(StockTradeCorrection),
    /// Notification that a previously reported trade was cancelled or erroneous.
    #[serde(rename = "x")]
    CancelError(StockTradeCancelError),
    /// Trading-status update.
    #[serde(rename = "s")]
    TradingStatus(StockTradingStatus),
    /// LULD band update.
    #[serde(rename = "l")]
    Luld(StockLuld),
    /// Order imbalance.
    #[serde(rename = "i")]
    Imbalance(StockImbalance),
}

impl StockStreamMessage {
    pub(crate) const fn control(&self) -> Option<&crate::streaming::wire::ControlMessage> {
        match self {
            StockStreamMessage::Control { msg } => Some(msg),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserializes_minute_bar() {
        let json = r#"{"T":"b","S":"AAPL","o":150.0,"h":151.0,"l":149.5,"c":150.5,"v":12345,"vw":150.25,"n":42,"t":"2024-01-02T15:30:00Z"}"#;
        match serde_json::from_str(json).unwrap() {
            StockStreamMessage::Bar(bar) => {
                assert_eq!(bar.symbol, "AAPL");
                assert_eq!(bar.vwap, Some(150.25));
                assert_eq!(bar.trade_count, Some(42));
            }
            other => panic!("expected Bar, got {other:?}"),
        }
    }

    #[test]
    fn deserializes_trade_with_conditions() {
        let json = r#"{"T":"t","S":"AAPL","i":12345,"x":"V","p":150.10,"s":100,"c":["@"],"t":"2024-01-02T15:30:00.123456789Z","z":"C"}"#;
        match serde_json::from_str(json).unwrap() {
            StockStreamMessage::Trade(trade) => {
                assert_eq!(trade.symbol, "AAPL");
                assert_eq!(trade.conditions.as_deref(), Some(&["@".to_string()][..]));
            }
            other => panic!("expected Trade, got {other:?}"),
        }
    }

    #[test]
    fn deserializes_quote_with_conditions() {
        let json = r#"{"T":"q","S":"AAPL","ax":"V","ap":150.10,"as":1,"bx":"V","bp":150.05,"bs":2,"c":["R"],"t":"2024-01-02T15:30:00Z","z":"C"}"#;
        match serde_json::from_str(json).unwrap() {
            StockStreamMessage::Quote(quote) => {
                assert_eq!(quote.conditions.as_deref(), Some(&["R".to_string()][..]));
            }
            other => panic!("expected Quote, got {other:?}"),
        }
    }

    #[test]
    fn deserializes_trade_correction() {
        let json = r#"{"T":"c","S":"AAPL","x":"V","oi":1,"op":150.0,"os":100,"oc":["@"],"ci":2,"cp":150.5,"cs":100,"cc":["@"],"t":"2024-01-02T15:30:00Z","z":"C"}"#;
        match serde_json::from_str(json).unwrap() {
            StockStreamMessage::Correction(c) => {
                assert_eq!(c.original_trade_id, 1);
                assert_eq!(c.corrected_trade_id, 2);
            }
            other => panic!("expected Correction, got {other:?}"),
        }
    }

    #[test]
    fn deserializes_trade_cancel_error() {
        let json = r#"{"T":"x","S":"AAPL","i":1,"x":"V","p":150.0,"s":100,"a":"C","t":"2024-01-02T15:30:00Z","z":"C"}"#;
        match serde_json::from_str(json).unwrap() {
            StockStreamMessage::CancelError(c) => {
                assert_eq!(c.action, TradeCancelAction::Cancel);
            }
            other => panic!("expected CancelError, got {other:?}"),
        }
    }

    #[test]
    fn deserializes_trading_status() {
        let json = r#"{"T":"s","S":"AAPL","sc":"H","sm":"Halted","rc":"M1","rm":"Halt","t":"2024-01-02T15:30:00Z","z":"C"}"#;
        match serde_json::from_str(json).unwrap() {
            StockStreamMessage::TradingStatus(s) => {
                assert_eq!(s.status_code, "H");
                assert_eq!(s.reason_code, "M1");
            }
            other => panic!("expected TradingStatus, got {other:?}"),
        }
    }

    #[test]
    fn deserializes_luld() {
        let json = r#"{"T":"l","S":"AAPL","u":160.0,"d":140.0,"i":"A","t":"2024-01-02T15:30:00Z","z":"C"}"#;
        match serde_json::from_str(json).unwrap() {
            StockStreamMessage::Luld(l) => {
                assert_eq!(l.limit_up, 160.0);
                assert_eq!(l.limit_down, 140.0);
            }
            other => panic!("expected Luld, got {other:?}"),
        }
    }

    #[test]
    fn deserializes_imbalance() {
        let json = r#"{"T":"i","S":"AAPL","p":150.0,"z":"C","t":"2024-01-02T15:30:00Z"}"#;
        match serde_json::from_str(json).unwrap() {
            StockStreamMessage::Imbalance(i) => {
                assert_eq!(i.price, 150.0);
            }
            other => panic!("expected Imbalance, got {other:?}"),
        }
    }

    #[test]
    fn subscription_list_serializes_only_set_fields() {
        let list = StockSubscriptionList::new()
            .add_trades("AAPL")
            .add_lulds("AAPL");
        let json = serde_json::to_string(&list).unwrap();
        assert!(json.contains("\"trades\""));
        assert!(json.contains("\"lulds\""));
        assert!(!json.contains("\"bars\""));
        assert!(!json.contains("\"news\""));
    }
}
