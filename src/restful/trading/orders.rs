use crate::restful::{SortDirection, TradingClient, string_as_decimal, string_as_optional_decimal};
use chrono::{DateTime, Utc};
use reqwest::Method;
use rust_decimal::Decimal;
use serde::{Deserialize, Deserializer, Serialize};

fn empty_string_as_none_order_class<'de, D>(deserializer: D) -> Result<Option<OrderClass>, D::Error>
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

/// Status filter accepted by the list-orders endpoint.
#[derive(Clone, Copy, Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
#[non_exhaustive]
pub enum OrderStatusFilter {
    /// Only return open orders.
    Open,
    /// Only return closed orders.
    Closed,
    /// Return both open and closed orders.
    All,
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

/// Take profit configuration for bracket orders.
#[derive(Clone, Debug, Serialize)]
pub struct TakeProfit {
    /// Target limit price for taking profit.
    pub limit_price: Decimal,
}

/// Stop loss configuration for bracket orders.
#[derive(Clone, Debug, Serialize)]
pub struct StopLoss {
    /// Stop price that triggers the stop loss.
    pub stop_price: Decimal,
    /// Optional limit price for a stop-limit loss order.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit_price: Option<Decimal>,
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

/// Builder for creating a new order.
#[derive(Debug, Serialize)]
#[must_use]
pub struct CreateOrderRequest<'a> {
    #[serde(skip)]
    client: &'a TradingClient,
    symbol: String,
    side: Side,
    #[serde(rename = "type")]
    order_type: OrderType,
    time_in_force: TimeInForce,
    #[serde(skip_serializing_if = "Option::is_none")]
    qty: Option<Decimal>,
    #[serde(skip_serializing_if = "Option::is_none")]
    notional: Option<Decimal>,
    #[serde(skip_serializing_if = "Option::is_none")]
    limit_price: Option<Decimal>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stop_price: Option<Decimal>,
    #[serde(skip_serializing_if = "Option::is_none")]
    trail_price: Option<Decimal>,
    #[serde(skip_serializing_if = "Option::is_none")]
    trail_percent: Option<Decimal>,
    #[serde(skip_serializing_if = "Option::is_none")]
    extended_hours: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    client_order_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    order_class: Option<OrderClass>,
    #[serde(skip_serializing_if = "Option::is_none")]
    take_profit: Option<TakeProfit>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stop_loss: Option<StopLoss>,
}

impl CreateOrderRequest<'_> {
    /// Set the quantity of shares to trade.
    pub fn qty(mut self, qty: Decimal) -> Self {
        self.qty = Some(qty);
        self.notional = None;
        self
    }

    /// Set the notional (dollar) amount to trade. Mutually exclusive with `qty`.
    pub fn notional(mut self, notional: Decimal) -> Self {
        self.notional = Some(notional);
        self.qty = None;
        self
    }

    /// Set the time in force.
    pub fn time_in_force(mut self, tif: TimeInForce) -> Self {
        self.time_in_force = tif;
        self
    }

    /// Set the limit price (required for Limit and StopLimit orders).
    pub fn limit_price(mut self, price: Decimal) -> Self {
        self.limit_price = Some(price);
        self
    }

    /// Set the stop price (required for Stop and StopLimit orders).
    pub fn stop_price(mut self, price: Decimal) -> Self {
        self.stop_price = Some(price);
        self
    }

    /// Set the trail price (for trailing stop orders).
    pub fn trail_price(mut self, price: Decimal) -> Self {
        self.trail_price = Some(price);
        self
    }

    /// Set the trail percent (for trailing stop orders).
    pub fn trail_percent(mut self, percent: Decimal) -> Self {
        self.trail_percent = Some(percent);
        self
    }

    /// Allow extended hours trading.
    pub fn extended_hours(mut self, extended: bool) -> Self {
        self.extended_hours = Some(extended);
        self
    }

    /// Set a client-defined order ID (max 128 characters).
    pub fn client_order_id(mut self, id: impl Into<String>) -> Self {
        self.client_order_id = Some(id.into());
        self
    }

    /// Set the order class for advanced order types.
    pub fn order_class(mut self, class: OrderClass) -> Self {
        self.order_class = Some(class);
        self
    }

    /// Set take profit for bracket orders.
    pub fn take_profit(mut self, limit_price: Decimal) -> Self {
        self.take_profit = Some(TakeProfit { limit_price });
        self
    }

    /// Set stop loss for bracket orders.
    pub fn stop_loss(mut self, stop_price: Decimal, limit_price: Option<Decimal>) -> Self {
        self.stop_loss = Some(StopLoss {
            stop_price,
            limit_price,
        });
        self
    }

    /// Submit the order.
    pub async fn execute(self) -> crate::Result<Order> {
        let request = self.client.request(Method::POST, "orders").json(&self);
        self.client.send_and_deserialize(request).await
    }
}

/// Builder for listing orders with filters.
#[derive(Debug, Serialize)]
#[must_use]
pub struct ListOrdersRequest<'a> {
    #[serde(skip)]
    client: &'a TradingClient,
    #[serde(skip_serializing_if = "Option::is_none")]
    status: Option<OrderStatusFilter>,
    #[serde(skip_serializing_if = "Option::is_none")]
    limit: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    after: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    until: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    direction: Option<SortDirection>,
    #[serde(skip_serializing_if = "Option::is_none")]
    nested: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    symbols: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    side: Option<Side>,
}

impl ListOrdersRequest<'_> {
    /// Filter by order status (open, closed, or all).
    pub fn status(mut self, status: OrderStatusFilter) -> Self {
        self.status = Some(status);
        self
    }

    /// Maximum number of orders to return (default 50, max 500).
    pub fn limit(mut self, limit: u32) -> Self {
        self.limit = Some(limit);
        self
    }

    /// Only return orders after this timestamp.
    pub fn after(mut self, after: DateTime<Utc>) -> Self {
        self.after = Some(after);
        self
    }

    /// Only return orders until this timestamp.
    pub fn until(mut self, until: DateTime<Utc>) -> Self {
        self.until = Some(until);
        self
    }

    /// Sort direction (ascending or descending).
    pub fn direction(mut self, direction: SortDirection) -> Self {
        self.direction = Some(direction);
        self
    }

    /// Include nested multi-leg order legs.
    pub fn nested(mut self, nested: bool) -> Self {
        self.nested = Some(nested);
        self
    }

    /// Filter by comma-separated symbols.
    pub fn symbols(mut self, symbols: &str) -> Self {
        self.symbols = Some(symbols.to_string());
        self
    }

    /// Filter by side.
    pub fn side(mut self, side: Side) -> Self {
        self.side = Some(side);
        self
    }

    /// Execute the list request.
    pub async fn execute(self) -> crate::Result<Vec<Order>> {
        let request = self.client.request(Method::GET, "orders").query(&self);
        self.client.send_and_deserialize(request).await
    }
}

/// Builder for replacing (modifying) an existing order.
#[derive(Debug, Serialize)]
#[must_use]
pub struct ReplaceOrderRequest<'a> {
    #[serde(skip)]
    client: &'a TradingClient,
    #[serde(skip)]
    order_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    qty: Option<Decimal>,
    #[serde(skip_serializing_if = "Option::is_none")]
    limit_price: Option<Decimal>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stop_price: Option<Decimal>,
    #[serde(skip_serializing_if = "Option::is_none")]
    time_in_force: Option<TimeInForce>,
    #[serde(skip_serializing_if = "Option::is_none")]
    trail: Option<Decimal>,
    #[serde(skip_serializing_if = "Option::is_none")]
    client_order_id: Option<String>,
}

impl ReplaceOrderRequest<'_> {
    /// Set the new quantity.
    pub fn qty(mut self, qty: Decimal) -> Self {
        self.qty = Some(qty);
        self
    }

    /// Set the new limit price.
    pub fn limit_price(mut self, price: Decimal) -> Self {
        self.limit_price = Some(price);
        self
    }

    /// Set the new stop price.
    pub fn stop_price(mut self, price: Decimal) -> Self {
        self.stop_price = Some(price);
        self
    }

    /// Set the new time in force.
    pub fn time_in_force(mut self, tif: TimeInForce) -> Self {
        self.time_in_force = Some(tif);
        self
    }

    /// Set the new trail value.
    pub fn trail(mut self, trail: Decimal) -> Self {
        self.trail = Some(trail);
        self
    }

    /// Set a new client order ID.
    pub fn client_order_id(mut self, id: impl Into<String>) -> Self {
        self.client_order_id = Some(id.into());
        self
    }

    /// Submit the replacement.
    pub async fn execute(self) -> crate::Result<Order> {
        let order_id = &self.order_id;
        let path = format!("orders/{order_id}");
        let request = self.client.request(Method::PATCH, &path).json(&self);
        self.client.send_and_deserialize(request).await
    }
}

impl TradingClient {
    /// Create a new order.
    ///
    /// ```ignore
    /// use rust_decimal_macros::dec;
    ///
    /// let order = client.create_order("AAPL", Side::Buy, OrderType::Market)
    ///     .qty(dec!(10))
    ///     .time_in_force(TimeInForce::Day)
    ///     .execute().await?;
    /// ```
    pub fn create_order(
        &self,
        symbol: &str,
        side: Side,
        order_type: OrderType,
    ) -> CreateOrderRequest<'_> {
        CreateOrderRequest {
            client: self,
            symbol: symbol.to_string(),
            side,
            order_type,
            time_in_force: TimeInForce::Day,
            qty: None,
            notional: None,
            limit_price: None,
            stop_price: None,
            trail_price: None,
            trail_percent: None,
            extended_hours: None,
            client_order_id: None,
            order_class: None,
            take_profit: None,
            stop_loss: None,
        }
    }

    /// List orders with optional filters.
    ///
    /// ```ignore
    /// let orders = client.list_orders()
    ///     .status("open")
    ///     .limit(10)
    ///     .execute().await?;
    /// ```
    pub fn list_orders(&self) -> ListOrdersRequest<'_> {
        ListOrdersRequest {
            client: self,
            status: None,
            limit: None,
            after: None,
            until: None,
            direction: None,
            nested: None,
            symbols: None,
            side: None,
        }
    }

    /// Get a specific order by ID.
    pub async fn get_order(&self, order_id: &str) -> crate::Result<Order> {
        let request = self.request(Method::GET, &format!("orders/{order_id}"));
        self.send_and_deserialize(request).await
    }

    /// Get an order by client order ID.
    pub async fn get_order_by_client_id(&self, client_order_id: &str) -> crate::Result<Order> {
        let request = self
            .request(Method::GET, "orders/by_client_order_id")
            .query(&[("client_order_id", client_order_id)]);
        self.send_and_deserialize(request).await
    }

    /// Cancel a specific order.
    pub async fn cancel_order(&self, order_id: &str) -> crate::Result<()> {
        let request = self.request(Method::DELETE, &format!("orders/{order_id}"));
        let response = request.send().await.map_err(crate::Error::ReqwestSend)?;
        let status = response.status();
        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            return Err(crate::Error::ApiError {
                status: status.as_u16(),
                body,
            });
        }
        Ok(())
    }

    /// Cancel all open orders.
    pub async fn cancel_all_orders(&self) -> crate::Result<()> {
        let request = self.request(Method::DELETE, "orders");
        let response = request.send().await.map_err(crate::Error::ReqwestSend)?;
        let status = response.status();
        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            return Err(crate::Error::ApiError {
                status: status.as_u16(),
                body,
            });
        }
        Ok(())
    }

    /// Replace (modify) an existing order.
    ///
    /// ```ignore
    /// use rust_decimal_macros::dec;
    ///
    /// let order = client.replace_order("order-id")
    ///     .qty(dec!(5))
    ///     .limit_price(dec!(150.00))
    ///     .execute().await?;
    /// ```
    pub fn replace_order(&self, order_id: &str) -> ReplaceOrderRequest<'_> {
        ReplaceOrderRequest {
            client: self,
            order_id: order_id.to_string(),
            qty: None,
            limit_price: None,
            stop_price: None,
            time_in_force: None,
            trail: None,
            client_order_id: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_order_deserialization() {
        let json = r#"{
            "id": "61e69015-8549-4baf-b96e-2c44f1f4e01b",
            "client_order_id": "my_order_1",
            "created_at": "2023-01-15T10:30:00Z",
            "updated_at": "2023-01-15T10:30:01Z",
            "submitted_at": "2023-01-15T10:30:00Z",
            "filled_at": "2023-01-15T10:30:01Z",
            "expired_at": null,
            "canceled_at": null,
            "asset_id": "904837e3-3b76-47ec-b432-046db621571b",
            "symbol": "AAPL",
            "qty": "10",
            "notional": null,
            "filled_qty": "10",
            "filled_avg_price": "150.25",
            "type": "market",
            "side": "buy",
            "time_in_force": "day",
            "status": "filled",
            "limit_price": null,
            "stop_price": null,
            "trail_price": null,
            "trail_percent": null,
            "extended_hours": false,
            "order_class": "simple",
            "legs": null
        }"#;
        let order: Order = serde_json::from_str(json).unwrap();
        assert_eq!(order.symbol, "AAPL");
        assert_eq!(order.status, OrderStatus::Filled);
        assert_eq!(order.qty, Some(Decimal::from_str_exact("10").unwrap()));
        assert_eq!(
            order.filled_avg_price,
            Some(Decimal::from_str_exact("150.25").unwrap())
        );
        assert_eq!(order.order_type, OrderType::Market);
        assert_eq!(order.side, Side::Buy);
    }
}
