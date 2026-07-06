use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::streaming::wire::StreamError;

/// Subscriptions for the options streaming feed.
///
/// Symbols are full OCC option codes (e.g. `AAPL240119C00150000`).
/// Note: Alpaca rejects the wildcard `"*"` for option quotes due to
/// volume; subscribe to specific contracts instead.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct OptionSubscriptionList {
    /// Contracts subscribed to trades.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trades: Option<Vec<String>>,
    /// Contracts subscribed to quotes. `"*"` is **not** accepted by the
    /// server for this channel.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quotes: Option<Vec<String>>,
}

impl OptionSubscriptionList {
    /// Create an empty subscription list.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Subscribe to trades for `symbol`.
    #[must_use]
    pub fn add_trades(self, symbol: &str) -> Self {
        Self {
            trades: Some(append_unique(self.trades, symbol)),
            ..self
        }
    }

    /// Subscribe to quotes for `symbol`. The server rejects `"*"` here.
    #[must_use]
    pub fn add_quotes(self, symbol: &str) -> Self {
        Self {
            quotes: Some(append_unique(self.quotes, symbol)),
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

/// Real-time options trade event.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct OptionTradeEvent {
    /// OCC option symbol.
    #[serde(rename = "S")]
    pub symbol: String,
    /// Trade timestamp.
    #[serde(rename = "t")]
    pub timestamp: DateTime<Utc>,
    /// Trade price.
    #[serde(rename = "p")]
    pub price: f64,
    /// Trade size (contracts).
    #[serde(rename = "s")]
    pub size: i64,
    /// Exchange code.
    #[serde(rename = "x")]
    pub exchange: Option<String>,
    /// Trade condition.
    #[serde(rename = "c")]
    pub condition: Option<String>,
}

/// Real-time options quote with bid and ask.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct OptionQuoteEvent {
    /// OCC option symbol.
    #[serde(rename = "S")]
    pub symbol: String,
    /// Quote timestamp.
    #[serde(rename = "t")]
    pub timestamp: DateTime<Utc>,
    /// Bid exchange code.
    #[serde(rename = "bx")]
    pub bid_exchange: Option<String>,
    /// Bid price.
    #[serde(rename = "bp")]
    pub bid_price: f64,
    /// Bid size (contracts).
    #[serde(rename = "bs")]
    pub bid_size: i64,
    /// Ask exchange code.
    #[serde(rename = "ax")]
    pub ask_exchange: Option<String>,
    /// Ask price.
    #[serde(rename = "ap")]
    pub ask_price: f64,
    /// Ask size (contracts).
    #[serde(rename = "as")]
    pub ask_size: i64,
    /// Quote condition.
    #[serde(rename = "c")]
    pub condition: Option<String>,
}

/// Messages received from the options streaming feed.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "T")]
#[non_exhaustive]
pub enum OptionStreamMessage {
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
    Subscription(OptionSubscriptionList),
    /// Trade event.
    #[serde(rename = "t")]
    Trade(OptionTradeEvent),
    /// Quote update.
    #[serde(rename = "q")]
    Quote(OptionQuoteEvent),
}

impl OptionStreamMessage {
    pub(crate) const fn control(&self) -> Option<&crate::streaming::wire::ControlMessage> {
        match self {
            OptionStreamMessage::Control { msg } => Some(msg),
            _ => None,
        }
    }

    pub(crate) const fn stream_error(&self) -> Option<&StreamError> {
        match self {
            OptionStreamMessage::Error(err) => Some(err),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trips_trade_via_msgpack() {
        let original = OptionTradeEvent {
            symbol: "AAPL240119C00150000".to_string(),
            timestamp: "2024-01-02T15:30:00Z".parse().unwrap(),
            price: 12.50,
            size: 5,
            exchange: Some("X".to_string()),
            condition: Some("@".to_string()),
        };
        let bytes = rmp_serde::to_vec_named(&original).unwrap();
        let decoded: OptionTradeEvent = rmp_serde::from_slice(&bytes).unwrap();
        assert_eq!(decoded.symbol, original.symbol);
        assert_eq!(decoded.size, 5);
        assert_eq!(decoded.condition.as_deref(), Some("@"));
    }

    #[test]
    fn round_trips_quote_via_msgpack() {
        let original = OptionQuoteEvent {
            symbol: "AAPL240119C00150000".to_string(),
            timestamp: "2024-01-02T15:30:00Z".parse().unwrap(),
            bid_exchange: Some("X".to_string()),
            bid_price: 12.45,
            bid_size: 10,
            ask_exchange: Some("X".to_string()),
            ask_price: 12.55,
            ask_size: 8,
            condition: Some("R".to_string()),
        };
        let bytes = rmp_serde::to_vec_named(&original).unwrap();
        let decoded: OptionQuoteEvent = rmp_serde::from_slice(&bytes).unwrap();
        assert_eq!(decoded.bid_size, 10);
        assert_eq!(decoded.ask_size, 8);
        assert_eq!(decoded.condition.as_deref(), Some("R"));
    }

    #[test]
    fn round_trips_stream_message_via_msgpack() {
        let trade = OptionStreamMessage::Trade(OptionTradeEvent {
            symbol: "AAPL240119C00150000".to_string(),
            timestamp: "2024-01-02T15:30:00Z".parse().unwrap(),
            price: 12.50,
            size: 1,
            exchange: None,
            condition: None,
        });
        let bytes = rmp_serde::to_vec_named(&trade).unwrap();
        match rmp_serde::from_slice::<OptionStreamMessage>(&bytes).unwrap() {
            OptionStreamMessage::Trade(t) => assert_eq!(t.symbol, "AAPL240119C00150000"),
            other => panic!("expected Trade, got {other:?}"),
        }
    }

    #[test]
    fn subscription_list_serializes_only_set_fields() {
        let list = OptionSubscriptionList::new()
            .add_trades("AAPL240119C00150000")
            .add_trades("AAPL240119C00150000"); // dedup
        let json = serde_json::to_string(&list).unwrap();
        assert!(json.contains("\"trades\""));
        assert!(!json.contains("\"quotes\""));
        assert_eq!(list.trades.as_deref().unwrap().len(), 1);
    }
}
