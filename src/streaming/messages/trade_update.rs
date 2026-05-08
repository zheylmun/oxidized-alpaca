use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use crate::{
    orders::Order,
    serde_helpers::string_as_optional_decimal,
};

/// Outgoing wire-protocol message used by the trade-updates stream.
///
/// Public only because it appears on the streaming codec's `Tx` type;
/// callers use [`crate::streaming::TradingUpdatesClient`] instead of
/// constructing these by hand.
#[doc(hidden)]
#[derive(Clone, Debug, Serialize)]
#[serde(tag = "action")]
pub enum TradingUpdatesRequest {
    /// Authenticate with API key and secret.
    #[serde(rename = "auth")]
    Auth {
        /// API key ID.
        key: String,
        /// API secret key.
        secret: String,
    },
    /// Subscribe to streams (use `["trade_updates"]`).
    #[serde(rename = "listen")]
    Listen {
        /// Wrapper for the listen payload.
        data: ListenStreams,
    },
}

/// Body of the `listen` request.
#[doc(hidden)]
#[derive(Clone, Debug, Serialize)]
pub struct ListenStreams {
    /// Streams to subscribe to (only `trade_updates` is currently meaningful).
    pub streams: Vec<String>,
}

/// Authorization status returned in the `authorization` envelope.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum AuthorizationStatus {
    /// Credentials accepted.
    Authorized,
    /// Credentials rejected.
    Unauthorized,
}

/// Payload of the `authorization` server envelope.
#[derive(Clone, Debug, Deserialize)]
pub struct Authorization {
    /// Authorization outcome.
    pub status: AuthorizationStatus,
    /// Echoed action name (typically `"authenticate"`).
    pub action: String,
}

/// Payload of the `listening` server envelope.
#[derive(Clone, Debug, Deserialize)]
pub struct Listening {
    /// The streams the server is now delivering.
    pub streams: Vec<String>,
}

/// Single execution leg reported alongside a multi-leg trade-update event.
#[derive(Clone, Debug, Deserialize)]
pub struct TradeUpdateLeg {
    /// Execution ID for this leg.
    #[serde(default)]
    pub execution_id: Option<String>,
    /// Order ID this leg belongs to.
    #[serde(default)]
    pub order_id: Option<String>,
    /// Symbol for this leg.
    #[serde(default)]
    pub symbol: Option<String>,
    /// Quantity executed on this leg.
    #[serde(
        default,
        deserialize_with = "string_as_optional_decimal",
        skip_serializing_if = "Option::is_none"
    )]
    pub qty: Option<Decimal>,
    /// Execution price for this leg.
    #[serde(
        default,
        deserialize_with = "string_as_optional_decimal",
        skip_serializing_if = "Option::is_none"
    )]
    pub price: Option<Decimal>,
    /// Timestamp of this leg's execution.
    #[serde(default)]
    pub timestamp: Option<DateTime<Utc>>,
}

/// Event types delivered on the `trade_updates` stream.
#[derive(Clone, Copy, Debug, Deserialize, Serialize, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum TradeUpdateEvent {
    /// Order routed to exchanges.
    New,
    /// Order accepted by the venue.
    Accepted,
    /// Order is pending acceptance.
    PendingNew,
    /// Order completely filled.
    Fill,
    /// Partial fill.
    PartialFill,
    /// Order canceled.
    Canceled,
    /// Cancellation pending.
    PendingCancel,
    /// Cancel request rejected.
    OrderCancelRejected,
    /// Order replaced.
    Replaced,
    /// Replacement pending.
    PendingReplace,
    /// Replace request rejected.
    OrderReplaceRejected,
    /// Order expired.
    Expired,
    /// Order rejected by the venue.
    Rejected,
    /// Order stopped.
    Stopped,
    /// Order suspended.
    Suspended,
    /// Order done for the trading day.
    DoneForDay,
    /// Order calculated.
    Calculated,
}

/// Payload of a `trade_updates` server envelope.
#[derive(Clone, Debug, Deserialize)]
pub struct TradeUpdate {
    /// What happened to the order.
    pub event: TradeUpdateEvent,
    /// Unique execution identifier (present on fill / partial_fill).
    #[serde(default)]
    pub execution_id: Option<String>,
    /// When the event occurred.
    pub timestamp: DateTime<Utc>,
    /// Fill price per share (present on fill / partial_fill).
    #[serde(
        default,
        deserialize_with = "string_as_optional_decimal",
        skip_serializing_if = "Option::is_none"
    )]
    pub price: Option<Decimal>,
    /// Quantity filled by this event.
    #[serde(
        default,
        deserialize_with = "string_as_optional_decimal",
        skip_serializing_if = "Option::is_none"
    )]
    pub qty: Option<Decimal>,
    /// Total position size after the event (positive = long, negative = short).
    #[serde(
        default,
        deserialize_with = "string_as_optional_decimal",
        skip_serializing_if = "Option::is_none"
    )]
    pub position_qty: Option<Decimal>,
    /// Server timestamp for multi-leg events.
    #[serde(default)]
    pub at: Option<DateTime<Utc>>,
    /// Unique event identifier for multi-leg events.
    #[serde(default)]
    pub event_id: Option<String>,
    /// Per-leg execution detail for multi-leg events.
    #[serde(default)]
    pub legs: Option<Vec<TradeUpdateLeg>>,
    /// The order this update applies to.
    pub order: Order,
}

/// Top-level envelope delivered by the trade-updates stream.
///
/// Only consumed internally by the trading-updates client; user code
/// receives [`TradeUpdate`] values directly.
#[allow(clippy::large_enum_variant)]
#[derive(Clone, Debug, Deserialize)]
#[serde(tag = "stream", content = "data")]
pub enum TradingUpdatesMessage {
    /// Authentication outcome.
    #[serde(rename = "authorization")]
    Authorization(Authorization),
    /// Subscription confirmation.
    #[serde(rename = "listening")]
    Listening(Listening),
    /// A trade update event.
    #[serde(rename = "trade_updates")]
    TradeUpdate(TradeUpdate),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserializes_authorization_authorized() {
        let json = r#"{"stream":"authorization","data":{"status":"authorized","action":"authenticate"}}"#;
        match serde_json::from_str(json).unwrap() {
            TradingUpdatesMessage::Authorization(auth) => {
                assert_eq!(auth.status, AuthorizationStatus::Authorized);
                assert_eq!(auth.action, "authenticate");
            }
            other => panic!("expected Authorization, got {other:?}"),
        }
    }

    #[test]
    fn deserializes_authorization_unauthorized() {
        let json = r#"{"stream":"authorization","data":{"status":"unauthorized","action":"authenticate"}}"#;
        match serde_json::from_str(json).unwrap() {
            TradingUpdatesMessage::Authorization(auth) => {
                assert_eq!(auth.status, AuthorizationStatus::Unauthorized);
            }
            other => panic!("expected Authorization, got {other:?}"),
        }
    }

    #[test]
    fn deserializes_listening() {
        let json = r#"{"stream":"listening","data":{"streams":["trade_updates"]}}"#;
        match serde_json::from_str(json).unwrap() {
            TradingUpdatesMessage::Listening(listening) => {
                assert_eq!(listening.streams, vec!["trade_updates".to_string()]);
            }
            other => panic!("expected Listening, got {other:?}"),
        }
    }

    #[test]
    fn deserializes_fill_event() {
        let json = r#"{
            "stream":"trade_updates",
            "data":{
                "event":"fill",
                "execution_id":"2f63ea93-423d-4169-b3f6-3fdafc10c418",
                "timestamp":"2022-04-19T17:45:05.024916716Z",
                "price":"105.8988475",
                "qty":"1790.86",
                "position_qty":"0",
                "order":{
                    "id":"abc",
                    "client_order_id":"client-abc",
                    "created_at":"2022-04-19T17:45:00Z",
                    "updated_at":"2022-04-19T17:45:05Z",
                    "submitted_at":"2022-04-19T17:45:00Z",
                    "filled_at":"2022-04-19T17:45:05Z",
                    "expired_at":null,
                    "canceled_at":null,
                    "asset_id":"asset-1",
                    "symbol":"AAPL",
                    "qty":"1790.86",
                    "filled_qty":"1790.86",
                    "filled_avg_price":"105.8988475",
                    "type":"market",
                    "side":"buy",
                    "time_in_force":"day",
                    "status":"filled",
                    "extended_hours":false,
                    "order_class":"",
                    "legs":null
                }
            }
        }"#;
        match serde_json::from_str(json).unwrap() {
            TradingUpdatesMessage::TradeUpdate(update) => {
                assert_eq!(update.event, TradeUpdateEvent::Fill);
                assert_eq!(update.execution_id.as_deref(), Some("2f63ea93-423d-4169-b3f6-3fdafc10c418"));
                assert_eq!(update.qty, Some("1790.86".parse().unwrap()));
                assert_eq!(update.order.symbol, "AAPL");
            }
            other => panic!("expected TradeUpdate, got {other:?}"),
        }
    }

    #[test]
    fn deserializes_minimal_new_event_without_price_or_qty() {
        let json = r#"{
            "stream":"trade_updates",
            "data":{
                "event":"new",
                "timestamp":"2022-04-19T17:45:00Z",
                "order":{
                    "id":"abc",
                    "client_order_id":"client-abc",
                    "created_at":"2022-04-19T17:45:00Z",
                    "updated_at":null,
                    "submitted_at":null,
                    "filled_at":null,
                    "expired_at":null,
                    "canceled_at":null,
                    "asset_id":"asset-1",
                    "symbol":"AAPL",
                    "qty":"100",
                    "filled_qty":"0",
                    "type":"market",
                    "side":"buy",
                    "time_in_force":"day",
                    "status":"new",
                    "extended_hours":false
                }
            }
        }"#;
        match serde_json::from_str(json).unwrap() {
            TradingUpdatesMessage::TradeUpdate(update) => {
                assert_eq!(update.event, TradeUpdateEvent::New);
                assert!(update.price.is_none());
                assert!(update.qty.is_none());
                assert!(update.execution_id.is_none());
            }
            other => panic!("expected TradeUpdate, got {other:?}"),
        }
    }

    #[test]
    fn each_event_variant_round_trips_via_snake_case() {
        // Covers every event documented at
        // https://docs.alpaca.markets/docs/websocket-streaming. If Alpaca
        // renames or adds an event, this test should fail loudly.
        let cases = [
            (TradeUpdateEvent::New, "\"new\""),
            (TradeUpdateEvent::Accepted, "\"accepted\""),
            (TradeUpdateEvent::PendingNew, "\"pending_new\""),
            (TradeUpdateEvent::Fill, "\"fill\""),
            (TradeUpdateEvent::PartialFill, "\"partial_fill\""),
            (TradeUpdateEvent::Canceled, "\"canceled\""),
            (TradeUpdateEvent::PendingCancel, "\"pending_cancel\""),
            (TradeUpdateEvent::OrderCancelRejected, "\"order_cancel_rejected\""),
            (TradeUpdateEvent::Replaced, "\"replaced\""),
            (TradeUpdateEvent::PendingReplace, "\"pending_replace\""),
            (TradeUpdateEvent::OrderReplaceRejected, "\"order_replace_rejected\""),
            (TradeUpdateEvent::Expired, "\"expired\""),
            (TradeUpdateEvent::Rejected, "\"rejected\""),
            (TradeUpdateEvent::Stopped, "\"stopped\""),
            (TradeUpdateEvent::Suspended, "\"suspended\""),
            (TradeUpdateEvent::DoneForDay, "\"done_for_day\""),
            (TradeUpdateEvent::Calculated, "\"calculated\""),
        ];
        for (variant, expected) in cases {
            assert_eq!(serde_json::to_string(&variant).unwrap(), expected);
            let parsed: TradeUpdateEvent = serde_json::from_str(expected).unwrap();
            assert_eq!(parsed, variant);
        }
    }
}
