//! Order domain types shared between the REST trading API and the
//! streaming trade-updates feed.

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Deserializer, Serialize};

use crate::serde_helpers::{string_as_decimal, string_as_optional_decimal};

pub(crate) fn empty_string_as_none_order_class<'de, D>(
    deserializer: D,
) -> Result<Option<OrderClass>, D::Error>
where
    D: Deserializer<'de>,
{
    let opt = Option::<String>::deserialize(deserializer)?;
    match opt.as_deref() {
        None | Some("") => Ok(None),
        Some(s) => {
            OrderClass::deserialize(serde::de::value::StrDeserializer::<D::Error>::new(s)).map(Some)
        }
    }
}

/// Side determines whether an order is a buy or sell order
#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
#[non_exhaustive]
pub enum Side {
    /// Buy order.
    Buy,
    /// Sell order.
    Sell,
}

/// Order type determines how an order should be executed
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum OrderType {
    /// Market order executed at current price.
    Market,
    /// Limit order executed at specified price or better.
    Limit,
    /// Stop order triggered at a specified price.
    Stop,
    /// Stop limit order combining stop and limit prices.
    StopLimit,
    /// Trailing stop order with a dynamic stop price.
    TrailingStop,
}

/// Time in force determines how long an order should remain active
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
#[non_exhaustive]
pub enum TimeInForce {
    /// Valid for the trading day only.
    Day,
    /// Good until canceled.
    Gtc,
    /// Market on Open / Limit on Open.
    Opg,
    /// Market on Close / Limit on Close.
    Cls,
    /// Immediate or Cancel.
    Ioc,
    /// Fill or Kill.
    Fok,
}

/// Order class for advanced order types.
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
#[non_exhaustive]
pub enum OrderClass {
    /// Simple single-leg order.
    Simple,
    /// Bracket order with take profit and stop loss.
    Bracket,
    /// One-cancels-other order.
    Oco,
    /// One-triggers-other order.
    Oto,
}

/// Status of an order.
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum OrderStatus {
    /// Order has been received and is new.
    New,
    /// Order has been partially filled.
    PartiallyFilled,
    /// Order has been completely filled.
    Filled,
    /// Order is done for the day.
    DoneForDay,
    /// Order has been canceled.
    Canceled,
    /// Order has expired.
    Expired,
    /// Order has been replaced.
    Replaced,
    /// Order cancellation is pending.
    PendingCancel,
    /// Order replacement is pending.
    PendingReplace,
    /// New order is pending acceptance.
    PendingNew,
    /// Order has been accepted.
    Accepted,
    /// Order has been accepted for bidding.
    AcceptedForBidding,
    /// Order has been stopped.
    Stopped,
    /// Order has been rejected.
    Rejected,
    /// Order has been suspended.
    Suspended,
    /// Order has been calculated.
    Calculated,
    /// Order is held.
    Held,
}

/// An order as returned by the Alpaca API.
#[derive(Clone, Debug, Deserialize)]
pub struct Order {
    /// Order ID.
    pub id: String,
    /// Client-specified order ID.
    pub client_order_id: String,
    /// Timestamp when the order was created.
    pub created_at: DateTime<Utc>,
    /// Timestamp when the order was last updated.
    pub updated_at: Option<DateTime<Utc>>,
    /// Timestamp when the order was submitted.
    pub submitted_at: Option<DateTime<Utc>>,
    /// Timestamp when the order was filled.
    pub filled_at: Option<DateTime<Utc>>,
    /// Timestamp when the order expired.
    pub expired_at: Option<DateTime<Utc>>,
    /// Timestamp when the order was canceled.
    pub canceled_at: Option<DateTime<Utc>>,
    /// Asset ID for the order.
    pub asset_id: String,
    /// Ticker symbol.
    pub symbol: String,
    /// Quantity of shares to trade.
    #[serde(
        default,
        deserialize_with = "string_as_optional_decimal",
        skip_serializing_if = "Option::is_none"
    )]
    pub qty: Option<Decimal>,
    /// Notional (dollar) amount of the order.
    #[serde(
        default,
        deserialize_with = "string_as_optional_decimal",
        skip_serializing_if = "Option::is_none"
    )]
    pub notional: Option<Decimal>,
    /// Quantity of shares filled so far.
    #[serde(deserialize_with = "string_as_decimal")]
    pub filled_qty: Decimal,
    /// Average price at which shares were filled.
    #[serde(
        default,
        deserialize_with = "string_as_optional_decimal",
        skip_serializing_if = "Option::is_none"
    )]
    pub filled_avg_price: Option<Decimal>,
    /// Order type (market, limit, etc.).
    #[serde(rename = "type")]
    pub order_type: OrderType,
    /// Buy or sell side.
    pub side: Side,
    /// Time in force for the order.
    pub time_in_force: TimeInForce,
    /// Current order status.
    pub status: OrderStatus,
    /// Limit price for limit and stop-limit orders.
    #[serde(
        default,
        deserialize_with = "string_as_optional_decimal",
        skip_serializing_if = "Option::is_none"
    )]
    pub limit_price: Option<Decimal>,
    /// Stop price for stop and stop-limit orders.
    #[serde(
        default,
        deserialize_with = "string_as_optional_decimal",
        skip_serializing_if = "Option::is_none"
    )]
    pub stop_price: Option<Decimal>,
    /// Trail price for trailing stop orders.
    #[serde(
        default,
        deserialize_with = "string_as_optional_decimal",
        skip_serializing_if = "Option::is_none"
    )]
    pub trail_price: Option<Decimal>,
    /// Trail percent for trailing stop orders.
    #[serde(
        default,
        deserialize_with = "string_as_optional_decimal",
        skip_serializing_if = "Option::is_none"
    )]
    pub trail_percent: Option<Decimal>,
    /// Whether extended hours trading is enabled.
    pub extended_hours: Option<bool>,
    /// Order class (simple, bracket, etc.).
    #[serde(default, deserialize_with = "empty_string_as_none_order_class")]
    pub order_class: Option<OrderClass>,
    /// Legs of a multi-leg order.
    #[serde(default)]
    pub legs: Option<Vec<Order>>,
}
