use serde::{Deserialize, Serialize};
use serde_json::to_string;

/// Side determines whether an order is a buy or sell order
#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Side {
    /// Purchase an asset, or exit a short position
    Buy,
    /// Sell an asset, or enter a short position
    Sell,
}

/// Order type determines how an order should be executed
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum OrderType {
    /// Execute as quickly as possible at the current market price.
    Market,
    /// Execute at a specific price or better.
    Limit,
    /// Execute as market order when a specific price is reached.
    Stop,
    /// Execute as limit order when a specific price is reached.
    StopLimit,
    /// Conditional order which automatically adjusts the stop price based on the market price.
    TrailingStop,
}

/// Time in force determines how long an order should remain active
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum TimeInForce {
    /// A day order is eligible for execution only on the day it is live.
    /// By default day orders are only valid during regular market hours.
    /// If the order is unfilled by the closing acution, it is automatically cancelled.
    /// If marked as elligible for extended hours, the order will remain active until the end of the extended hours session.
    /// Any day order submitted after the close, it is queued for the next trading day.
    #[serde(rename = "day")]
    Day,
    /// A good-till-canceled order remains active until it is either filled or manually cancelled.
    #[serde(rename = "gtc")]
    GoodTilCanceled,
    /// Order type used to create Market On Open (MOO) and Limit On Open (LOO) orders.
    /// Eligible to execute only in the market opening auction.
    /// If the order is not filled during the auction, it is automatically cancelled.
    /// Orders submitted betwen 9:28am and 7:00pm ET are automatically cancelled.
    ///
    #[serde(rename = "opg")]
    OPG,
    #[serde(rename = "cls")]
    CLS,
    #[serde(rename = "ioc")]
    IOC,
    #[serde(rename = "fok")]
    FOK,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Order {
    symbol: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    qty: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    notional: Option<String>,
    side: String,
    #[serde(rename = "type")]
    order_type: String,
    time_in_force: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    limit_price: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stop_price: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    trail_price: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    trail_percent: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    extended_hours: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    client_order_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    order_class: Option<String>,
}

impl Order {
    pub fn place_market_order(symbol: String, side: Side, time_in_force: TimeInForce) -> Self {
        Order {
            symbol,
            qty: None,
            notional: None,
            side: to_string(&side).unwrap(),
            order_type: to_string(&OrderType::Market).unwrap(),
            time_in_force: to_string(&time_in_force).unwrap(),
            limit_price: None,
            stop_price: None,
            trail_price: None,
            trail_percent: None,
            extended_hours: None,
            client_order_id: None,
            order_class: None,
        }
    }
}
