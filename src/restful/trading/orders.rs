use crate::restful::{SortDirection, TradingClient};
use chrono::{DateTime, Utc};
use reqwest::Method;
use rust_decimal::Decimal;
use serde::Serialize;

pub use crate::orders::{Order, OrderClass, OrderStatus, OrderType, Side, TimeInForce};

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

/// Take-profit leg configuration for bracket / OTO orders.
#[derive(Clone, Debug, Serialize)]
pub struct TakeProfit {
    /// Target limit price at which the take-profit child order fires.
    pub limit_price: Decimal,
}

impl TakeProfit {
    /// Build a take-profit leg with the given target limit price.
    pub fn new(limit_price: Decimal) -> Self {
        Self { limit_price }
    }
}

/// Stop-loss leg configuration for bracket / OTO orders.
#[derive(Clone, Debug, Serialize)]
pub struct StopLoss {
    /// Stop price that triggers the stop-loss child order.
    pub stop_price: Decimal,
    /// Optional limit price; when set, the child order is a stop-limit
    /// instead of a plain stop.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit_price: Option<Decimal>,
}

impl StopLoss {
    /// Build a plain stop-loss leg that fires a market order at `stop_price`.
    pub fn new(stop_price: Decimal) -> Self {
        Self {
            stop_price,
            limit_price: None,
        }
    }

    /// Build a stop-limit-loss leg: triggers at `stop_price` and submits a
    /// limit order at `limit_price`.
    pub fn with_limit(stop_price: Decimal, limit_price: Decimal) -> Self {
        Self {
            stop_price,
            limit_price: Some(limit_price),
        }
    }
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
    pub fn client_order_id(mut self, id: &str) -> Self {
        self.client_order_id = Some(id.to_string());
        self
    }

    /// Set the order class for advanced order types.
    pub fn order_class(mut self, class: OrderClass) -> Self {
        self.order_class = Some(class);
        self
    }

    /// Attach a take-profit leg for bracket / OTO orders.
    pub fn take_profit(mut self, take_profit: TakeProfit) -> Self {
        self.take_profit = Some(take_profit);
        self
    }

    /// Attach a stop-loss leg for bracket / OTO orders.
    pub fn stop_loss(mut self, stop_loss: StopLoss) -> Self {
        self.stop_loss = Some(stop_loss);
        self
    }

    /// Submit the order.
    pub async fn execute(self) -> crate::Result<Order> {
        let request = self.client.request(Method::POST, "v2/orders")?.json(&self);
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
    limit: Option<usize>,
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
    pub fn limit(mut self, limit: usize) -> Self {
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

    /// Filter by symbols.
    pub fn symbols(mut self, symbols: &[&str]) -> Self {
        self.symbols = Some(symbols.join(","));
        self
    }

    /// Filter by side.
    pub fn side(mut self, side: Side) -> Self {
        self.side = Some(side);
        self
    }

    /// Execute the list request.
    pub async fn execute(self) -> crate::Result<Vec<Order>> {
        let request = self.client.request(Method::GET, "v2/orders")?.query(&self);
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
    pub fn client_order_id(mut self, id: &str) -> Self {
        self.client_order_id = Some(id.to_string());
        self
    }

    /// Submit the replacement.
    pub async fn execute(self) -> crate::Result<Order> {
        let order_id = &self.order_id;
        let path = format!("v2/orders/{order_id}");
        let request = self.client.request(Method::PATCH, &path)?.json(&self);
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
        let request = self.request(Method::GET, &format!("v2/orders/{order_id}"))?;
        self.send_and_deserialize(request).await
    }

    /// Get an order by client order ID.
    pub async fn get_order_by_client_id(&self, client_order_id: &str) -> crate::Result<Order> {
        let request = self
            .request(Method::GET, "v2/orders/by_client_order_id")?
            .query(&[("client_order_id", client_order_id)]);
        self.send_and_deserialize(request).await
    }

    /// Cancel a specific order.
    pub async fn cancel_order(&self, order_id: &str) -> crate::Result<()> {
        let request = self.request(Method::DELETE, &format!("v2/orders/{order_id}"))?;
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
        let request = self.request(Method::DELETE, "v2/orders")?;
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
    use crate::AccountType;
    use serial_test::serial;
    use std::env;

    fn ensure_paper_creds() {
        unsafe {
            if env::var("ALPACA_PAPER_API_KEY_ID").is_err() {
                env::set_var("ALPACA_PAPER_API_KEY_ID", "test_key_id");
            }
            if env::var("ALPACA_PAPER_API_SECRET_KEY").is_err() {
                env::set_var("ALPACA_PAPER_API_SECRET_KEY", "test_secret_key");
            }
        }
    }

    fn paper_client() -> TradingClient {
        ensure_paper_creds();
        TradingClient::new(AccountType::Paper).unwrap()
    }

    fn dec(s: &str) -> Decimal {
        Decimal::from_str_exact(s).unwrap()
    }

    #[test]
    fn take_profit_new_sets_limit_price() {
        let tp = TakeProfit::new(dec("150.00"));
        assert_eq!(tp.limit_price, dec("150.00"));
    }

    #[test]
    fn stop_loss_new_sets_stop_price_only() {
        let sl = StopLoss::new(dec("140.50"));
        assert_eq!(sl.stop_price, dec("140.50"));
        assert!(sl.limit_price.is_none());
    }

    #[test]
    fn stop_loss_with_limit_sets_both_fields() {
        let sl = StopLoss::with_limit(dec("140.50"), dec("139.00"));
        assert_eq!(sl.stop_price, dec("140.50"));
        assert_eq!(sl.limit_price, Some(dec("139.00")));
    }

    #[test]
    fn stop_loss_new_skips_limit_price_in_serialization() {
        let sl = StopLoss::new(dec("140.50"));
        let value = serde_json::to_value(&sl).unwrap();
        let obj = value.as_object().expect("StopLoss serializes to object");
        assert!(
            !obj.contains_key("limit_price"),
            "expected no limit_price key for plain StopLoss, got {value}"
        );
        assert_eq!(obj.get("stop_price").and_then(|v| v.as_str()), Some("140.50"));
    }

    #[test]
    #[serial]
    fn create_order_take_profit_setter_serializes() {
        let client = paper_client();
        let request = client
            .create_order("AAPL", Side::Buy, OrderType::Market)
            .qty(dec("10"))
            .take_profit(TakeProfit::new(dec("150.00")));
        let value = serde_json::to_value(&request).unwrap();
        let tp = value
            .get("take_profit")
            .expect("take_profit field present in JSON");
        assert_eq!(
            tp.get("limit_price").and_then(|v| v.as_str()),
            Some("150.00")
        );
    }

    #[test]
    #[serial]
    fn create_order_stop_loss_setter_serializes_without_limit() {
        let client = paper_client();
        let request = client
            .create_order("AAPL", Side::Sell, OrderType::Market)
            .qty(dec("10"))
            .stop_loss(StopLoss::new(dec("140.50")));
        let value = serde_json::to_value(&request).unwrap();
        let sl = value
            .get("stop_loss")
            .expect("stop_loss field present in JSON");
        let sl_obj = sl.as_object().expect("stop_loss serializes to object");
        assert_eq!(
            sl_obj.get("stop_price").and_then(|v| v.as_str()),
            Some("140.50")
        );
        assert!(
            !sl_obj.contains_key("limit_price"),
            "expected no limit_price key on plain stop loss, got {sl}"
        );
    }

    #[test]
    #[serial]
    fn create_order_stop_loss_with_limit_serializes_both_fields() {
        let client = paper_client();
        let request = client
            .create_order("AAPL", Side::Sell, OrderType::Market)
            .qty(dec("10"))
            .stop_loss(StopLoss::with_limit(dec("140.50"), dec("139.00")));
        let value = serde_json::to_value(&request).unwrap();
        let sl = value
            .get("stop_loss")
            .expect("stop_loss field present in JSON");
        assert_eq!(
            sl.get("stop_price").and_then(|v| v.as_str()),
            Some("140.50")
        );
        assert_eq!(
            sl.get("limit_price").and_then(|v| v.as_str()),
            Some("139.00")
        );
    }

    #[test]
    #[serial]
    fn list_orders_limit_setter_serializes_to_query() {
        let client = paper_client();
        let request = client.list_orders().limit(10);
        let query = serde_urlencoded::to_string(&request).unwrap();
        assert!(
            query.contains("limit=10"),
            "expected limit=10 in query string, got {query}"
        );
    }

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
