use crate::restful::{SortDirection, TradingClient};
use crate::{AssetClass, ClientOrderId, OrderId};
use chrono::{DateTime, Utc};
use reqwest::Method;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::marker::PhantomData;

pub use crate::orders::{Order, OrderClass, OrderStatus, OrderType, Side, TimeInForce};

/// Marker for a [`CreateOrderRequest`] that has not had a size
/// ([`qty`][CreateOrderRequest::qty] or
/// [`notional`][CreateOrderRequest::notional]) chosen yet.
/// `execute()` is unavailable in this state.
#[derive(Debug)]
pub enum Draft {}

/// Marker for a [`CreateOrderRequest`] that has had a size chosen and
/// is ready to submit. `execute()` is only available in this state.
#[derive(Debug)]
pub enum Ready {}

fn infer_order_class(has_take_profit: bool, has_stop_loss: bool) -> Option<OrderClass> {
    match (has_take_profit, has_stop_loss) {
        (true, true) => Some(OrderClass::Bracket),
        (true, false) | (false, true) => Some(OrderClass::Oto),
        (false, false) => None,
    }
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

/// Per-order outcome from the bulk cancel endpoint.
///
/// Alpaca's `DELETE /v2/orders` returns HTTP 207 with one of these
/// entries per order it tried to cancel. `status` is the per-order HTTP
/// status code; entries with `status == 200` are cancellations that
/// succeeded, anything else is a failure with details in `body`.
#[derive(Clone, Debug, Deserialize)]
#[non_exhaustive]
pub struct CancelOrderStatus {
    /// The order id the cancel was attempted for.
    pub id: OrderId,
    /// HTTP status code reported for this individual cancel.
    pub status: u16,
    /// Per-order response payload. Contains the cancelled order on
    /// success, an error object on failure. Held as a raw JSON value
    /// so callers can decide how strictly to interpret it.
    #[serde(default)]
    pub body: Option<serde_json::Value>,
}

/// Take-profit leg configuration for bracket / OTO orders.
#[derive(Clone, Debug, Serialize)]
#[non_exhaustive]
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
#[non_exhaustive]
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

/// Builder for submitting a new order.
///
/// Construct one through a per-`OrderType` entry point on
/// [`TradingClient`] (e.g. [`TradingClient::market_order`],
/// [`TradingClient::limit_order`], [`TradingClient::stop_limit_order`]).
/// Each entry point bakes in the parameters that order type requires.
///
/// Alpaca requires every order to specify either a share quantity
/// ([`qty`][Self::qty]) or a dollar amount ([`notional`][Self::notional])
/// — never both, never neither. That invariant is encoded in the type
/// state: builders start in [`Draft`] and `execute()` is only available
/// once `qty` or `notional` has been called, transitioning to [`Ready`].
/// The remaining setters ([`time_in_force`][Self::time_in_force],
/// [`extended_hours`][Self::extended_hours],
/// [`client_order_id`][Self::client_order_id],
/// [`take_profit`][Self::take_profit], [`stop_loss`][Self::stop_loss],
/// [`order_class`][Self::order_class]) are genuinely optional.
#[derive(Debug, Serialize)]
#[must_use]
pub struct CreateOrderRequest<'a, S = Draft> {
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
    client_order_id: Option<ClientOrderId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    order_class: Option<OrderClass>,
    #[serde(skip_serializing_if = "Option::is_none")]
    take_profit: Option<TakeProfit>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stop_loss: Option<StopLoss>,
    #[serde(skip)]
    _marker: PhantomData<S>,
}

impl<'a, S> CreateOrderRequest<'a, S> {
    fn into_state<S2>(
        self,
        qty: Option<Decimal>,
        notional: Option<Decimal>,
    ) -> CreateOrderRequest<'a, S2> {
        CreateOrderRequest {
            client: self.client,
            symbol: self.symbol,
            side: self.side,
            order_type: self.order_type,
            time_in_force: self.time_in_force,
            qty,
            notional,
            limit_price: self.limit_price,
            stop_price: self.stop_price,
            trail_price: self.trail_price,
            trail_percent: self.trail_percent,
            extended_hours: self.extended_hours,
            client_order_id: self.client_order_id,
            order_class: self.order_class,
            take_profit: self.take_profit,
            stop_loss: self.stop_loss,
            _marker: PhantomData,
        }
    }

    /// Size the order by share quantity. Mutually exclusive with
    /// [`notional`][Self::notional]; calling this transitions the
    /// builder into the [`Ready`] state where `execute()` becomes
    /// available.
    pub fn qty(self, qty: Decimal) -> CreateOrderRequest<'a, Ready> {
        self.into_state(Some(qty), None)
    }

    /// Size the order by dollar amount. Mutually exclusive with
    /// [`qty`][Self::qty]; calling this transitions the builder into
    /// the [`Ready`] state where `execute()` becomes available.
    pub fn notional(self, notional: Decimal) -> CreateOrderRequest<'a, Ready> {
        self.into_state(None, Some(notional))
    }

    /// Set the time in force.
    pub fn time_in_force(mut self, tif: TimeInForce) -> Self {
        self.time_in_force = tif;
        self
    }

    /// Allow extended hours trading.
    pub fn extended_hours(mut self, extended: bool) -> Self {
        self.extended_hours = Some(extended);
        self
    }

    /// Set a client-defined order ID (max 128 characters).
    pub fn client_order_id(mut self, id: impl Into<ClientOrderId>) -> Self {
        self.client_order_id = Some(id.into());
        self
    }

    /// Attach a take-profit leg, promoting this order into the
    /// appropriate advanced order class. With both `take_profit` and
    /// [`stop_loss`][Self::stop_loss] set the order is submitted as a
    /// bracket; with only one set it becomes an OTO. Use
    /// [`order_class`][Self::order_class] to override the inference
    /// (e.g. for OCO).
    pub fn take_profit(mut self, take_profit: TakeProfit) -> Self {
        self.take_profit = Some(take_profit);
        self
    }

    /// Attach a stop-loss leg. See [`take_profit`][Self::take_profit]
    /// for how this interacts with the inferred order class.
    pub fn stop_loss(mut self, stop_loss: StopLoss) -> Self {
        self.stop_loss = Some(stop_loss);
        self
    }

    /// Override the inferred order class. Most callers should not
    /// touch this — it only matters for OCO exits, where neither
    /// "bracket" nor "OTO" applies even though both legs are set.
    pub fn order_class(mut self, class: OrderClass) -> Self {
        self.order_class = Some(class);
        self
    }
}

impl CreateOrderRequest<'_, Ready> {
    /// Submit the order.
    pub async fn execute(mut self) -> crate::Result<Order> {
        if self.order_class.is_none() {
            self.order_class =
                infer_order_class(self.take_profit.is_some(), self.stop_loss.is_some());
        }
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
    #[serde(skip_serializing_if = "Option::is_none")]
    asset_class: Option<AssetClass>,
    #[serde(skip_serializing_if = "Option::is_none")]
    before_order_id: Option<OrderId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    after_order_id: Option<OrderId>,
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

    /// Only return orders submitted after this timestamp. Mutually
    /// exclusive with [`before_order_id`](Self::before_order_id) and
    /// [`after_order_id`](Self::after_order_id); setting this clears
    /// either cursor.
    pub fn after(mut self, after: DateTime<Utc>) -> Self {
        self.after = Some(after);
        self.before_order_id = None;
        self.after_order_id = None;
        self
    }

    /// Only return orders submitted up to this timestamp. Mutually
    /// exclusive with [`before_order_id`](Self::before_order_id) and
    /// [`after_order_id`](Self::after_order_id); setting this clears
    /// either cursor.
    pub fn until(mut self, until: DateTime<Utc>) -> Self {
        self.until = Some(until);
        self.before_order_id = None;
        self.after_order_id = None;
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

    /// Filter by asset class (equities, options, crypto, …).
    pub fn asset_class(mut self, asset_class: AssetClass) -> Self {
        self.asset_class = Some(asset_class);
        self
    }

    /// Cursor pagination: only return orders submitted before the order
    /// with `id`. Mutually exclusive with [`after`](Self::after) and
    /// [`until`](Self::until); setting this clears both timestamp
    /// filters.
    pub fn before_order_id(mut self, id: impl Into<OrderId>) -> Self {
        self.before_order_id = Some(id.into());
        self.after = None;
        self.until = None;
        self
    }

    /// Cursor pagination: only return orders submitted after the order
    /// with `id`. Mutually exclusive with [`after`](Self::after) and
    /// [`until`](Self::until); setting this clears both timestamp
    /// filters.
    pub fn after_order_id(mut self, id: impl Into<OrderId>) -> Self {
        self.after_order_id = Some(id.into());
        self.after = None;
        self.until = None;
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
    order_id: OrderId,
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
    client_order_id: Option<ClientOrderId>,
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
    pub fn client_order_id(mut self, id: impl Into<ClientOrderId>) -> Self {
        self.client_order_id = Some(id.into());
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
    /// Internal helper that constructs a fresh [`CreateOrderRequest`]
    /// in the [`Draft`] state with all type-specific price fields unset.
    /// The public per-`OrderType` entry points layer the required fields
    /// on top.
    fn new_create_order_request(
        &self,
        symbol: &str,
        side: Side,
        order_type: OrderType,
    ) -> CreateOrderRequest<'_, Draft> {
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
            _marker: PhantomData,
        }
    }

    /// Submit a market order.
    ///
    /// ```ignore
    /// use rust_decimal_macros::dec;
    ///
    /// let order = client.market_order("AAPL", Side::Buy)
    ///     .qty(dec!(10))
    ///     .execute().await?;
    /// ```
    pub fn market_order(&self, symbol: &str, side: Side) -> CreateOrderRequest<'_> {
        self.new_create_order_request(symbol, side, OrderType::Market)
    }

    /// Submit a limit order at `limit_price`.
    ///
    /// ```ignore
    /// use rust_decimal_macros::dec;
    ///
    /// let order = client.limit_order("AAPL", Side::Buy, dec!(150))
    ///     .qty(dec!(10))
    ///     .time_in_force(TimeInForce::Gtc)
    ///     .execute().await?;
    /// ```
    pub fn limit_order(
        &self,
        symbol: &str,
        side: Side,
        limit_price: Decimal,
    ) -> CreateOrderRequest<'_> {
        let mut req = self.new_create_order_request(symbol, side, OrderType::Limit);
        req.limit_price = Some(limit_price);
        req
    }

    /// Submit a stop order that triggers at `stop_price`.
    pub fn stop_order(
        &self,
        symbol: &str,
        side: Side,
        stop_price: Decimal,
    ) -> CreateOrderRequest<'_> {
        let mut req = self.new_create_order_request(symbol, side, OrderType::Stop);
        req.stop_price = Some(stop_price);
        req
    }

    /// Submit a stop-limit order that triggers at `stop_price` and then
    /// rests as a limit order at `limit_price`.
    pub fn stop_limit_order(
        &self,
        symbol: &str,
        side: Side,
        stop_price: Decimal,
        limit_price: Decimal,
    ) -> CreateOrderRequest<'_> {
        let mut req = self.new_create_order_request(symbol, side, OrderType::StopLimit);
        req.stop_price = Some(stop_price);
        req.limit_price = Some(limit_price);
        req
    }

    /// Submit a trailing-stop order that trails the favorable price by
    /// the given absolute amount.
    ///
    /// Mutually exclusive with
    /// [`trailing_stop_order_by_percent`][Self::trailing_stop_order_by_percent].
    pub fn trailing_stop_order_by_price(
        &self,
        symbol: &str,
        side: Side,
        trail_price: Decimal,
    ) -> CreateOrderRequest<'_> {
        let mut req = self.new_create_order_request(symbol, side, OrderType::TrailingStop);
        req.trail_price = Some(trail_price);
        req
    }

    /// Submit a trailing-stop order that trails the favorable price by
    /// the given percentage.
    ///
    /// Mutually exclusive with
    /// [`trailing_stop_order_by_price`][Self::trailing_stop_order_by_price].
    pub fn trailing_stop_order_by_percent(
        &self,
        symbol: &str,
        side: Side,
        trail_percent: Decimal,
    ) -> CreateOrderRequest<'_> {
        let mut req = self.new_create_order_request(symbol, side, OrderType::TrailingStop);
        req.trail_percent = Some(trail_percent);
        req
    }

    /// List orders with optional filters.
    ///
    /// ```ignore
    /// let orders = client.list_orders()
    ///     .status(OrderStatusFilter::Open)
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
            asset_class: None,
            before_order_id: None,
            after_order_id: None,
        }
    }

    /// Get a specific order by ID.
    pub async fn get_order(&self, order_id: &OrderId) -> crate::Result<Order> {
        let request = self.request(Method::GET, &format!("v2/orders/{order_id}"))?;
        self.send_and_deserialize(request).await
    }

    /// Get an order by client order ID.
    pub async fn get_order_by_client_id(
        &self,
        client_order_id: &ClientOrderId,
    ) -> crate::Result<Order> {
        let request = self
            .request(Method::GET, "v2/orders/by_client_order_id")?
            .query(&[("client_order_id", client_order_id.as_str())]);
        self.send_and_deserialize(request).await
    }

    /// Cancel a specific order.
    pub async fn cancel_order(&self, order_id: &OrderId) -> crate::Result<()> {
        let request = self.request(Method::DELETE, &format!("v2/orders/{order_id}"))?;
        self.send_no_body(request).await
    }

    /// Attempt to cancel every open order.
    ///
    /// Alpaca processes each open order individually and returns a
    /// per-order outcome; success is HTTP 207 with the array surfaced
    /// here. Inspect each [`CancelOrderStatus::status`] to distinguish
    /// successful cancels (`200`) from failures.
    pub async fn cancel_all_orders(&self) -> crate::Result<Vec<CancelOrderStatus>> {
        let request = self.request(Method::DELETE, "v2/orders")?;
        self.send_and_deserialize(request).await
    }

    /// Replace (modify) an existing order.
    ///
    /// ```ignore
    /// use oxidized_alpaca::OrderId;
    /// use rust_decimal_macros::dec;
    ///
    /// let order_id = OrderId::new("order-id");
    /// let order = client.replace_order(&order_id)
    ///     .qty(dec!(5))
    ///     .limit_price(dec!(150.00))
    ///     .execute().await?;
    /// ```
    pub fn replace_order(&self, order_id: &OrderId) -> ReplaceOrderRequest<'_> {
        ReplaceOrderRequest {
            client: self,
            order_id: order_id.clone(),
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
        assert_eq!(
            obj.get("stop_price").and_then(|v| v.as_str()),
            Some("140.50")
        );
    }

    #[test]
    #[serial]
    fn create_order_take_profit_setter_serializes() {
        let client = paper_client();
        let request = client
            .market_order("AAPL", Side::Buy)
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
            .market_order("AAPL", Side::Sell)
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
            .market_order("AAPL", Side::Sell)
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
    #[serial]
    fn list_orders_asset_class_serializes_alpaca_wire_value() {
        let client = paper_client();
        let request = client.list_orders().asset_class(AssetClass::UsOption);
        let query = serde_urlencoded::to_string(&request).unwrap();
        assert!(
            query.contains("asset_class=us_option"),
            "expected asset_class=us_option in query string, got {query}"
        );
    }

    #[test]
    #[serial]
    fn list_orders_cursor_clears_timestamp_filters() {
        let client = paper_client();
        let request = client
            .list_orders()
            .after(
                DateTime::parse_from_rfc3339("2025-05-01T00:00:00Z")
                    .unwrap()
                    .with_timezone(&Utc),
            )
            .until(
                DateTime::parse_from_rfc3339("2025-05-10T00:00:00Z")
                    .unwrap()
                    .with_timezone(&Utc),
            )
            .before_order_id("11111111-1111-1111-1111-111111111111");
        let query = serde_urlencoded::to_string(&request).unwrap();
        assert!(
            query.contains("before_order_id=11111111-1111-1111-1111-111111111111"),
            "expected before_order_id in query string, got {query}"
        );
        assert!(
            !query.contains("after="),
            "after should be cleared by before_order_id, got {query}"
        );
        assert!(
            !query.contains("until="),
            "until should be cleared by before_order_id, got {query}"
        );
    }

    #[test]
    #[serial]
    fn list_orders_timestamp_filters_clear_cursor() {
        let client = paper_client();
        let request = client
            .list_orders()
            .after_order_id("22222222-2222-2222-2222-222222222222")
            .after(
                DateTime::parse_from_rfc3339("2025-05-01T00:00:00Z")
                    .unwrap()
                    .with_timezone(&Utc),
            );
        let query = serde_urlencoded::to_string(&request).unwrap();
        assert!(
            query.contains("after="),
            "expected after timestamp in query string, got {query}"
        );
        assert!(
            !query.contains("after_order_id="),
            "after_order_id should be cleared by after(...), got {query}"
        );
    }

    #[test]
    #[serial]
    fn limit_order_pre_populates_type_and_price() {
        let client = paper_client();
        let request = client.limit_order("AAPL", Side::Buy, dec("150"));
        let value = serde_json::to_value(&request).unwrap();
        assert_eq!(value.get("type").and_then(|v| v.as_str()), Some("limit"));
        assert_eq!(
            value.get("limit_price").and_then(|v| v.as_str()),
            Some("150")
        );
    }

    #[test]
    #[serial]
    fn stop_limit_order_pre_populates_both_prices() {
        let client = paper_client();
        let request = client.stop_limit_order("AAPL", Side::Sell, dec("140"), dec("139"));
        let value = serde_json::to_value(&request).unwrap();
        assert_eq!(
            value.get("type").and_then(|v| v.as_str()),
            Some("stop_limit")
        );
        assert_eq!(
            value.get("stop_price").and_then(|v| v.as_str()),
            Some("140")
        );
        assert_eq!(
            value.get("limit_price").and_then(|v| v.as_str()),
            Some("139")
        );
    }

    #[test]
    #[serial]
    fn trailing_stop_by_price_and_percent_are_distinct() {
        let client = paper_client();
        let by_price = serde_json::to_value(client.trailing_stop_order_by_price(
            "AAPL",
            Side::Sell,
            dec("0.50"),
        ))
        .unwrap();
        assert_eq!(
            by_price.get("trail_price").and_then(|v| v.as_str()),
            Some("0.50")
        );
        assert!(by_price.get("trail_percent").is_none());

        let by_pct = serde_json::to_value(client.trailing_stop_order_by_percent(
            "AAPL",
            Side::Sell,
            dec("2.5"),
        ))
        .unwrap();
        assert_eq!(
            by_pct.get("trail_percent").and_then(|v| v.as_str()),
            Some("2.5")
        );
        assert!(by_pct.get("trail_price").is_none());
    }

    #[test]
    fn infer_order_class_returns_bracket_when_both_legs_set() {
        assert_eq!(infer_order_class(true, true), Some(OrderClass::Bracket));
    }

    #[test]
    fn infer_order_class_returns_oto_when_one_leg_set() {
        assert_eq!(infer_order_class(true, false), Some(OrderClass::Oto));
        assert_eq!(infer_order_class(false, true), Some(OrderClass::Oto));
    }

    #[test]
    fn infer_order_class_returns_none_when_no_legs_set() {
        assert_eq!(infer_order_class(false, false), None);
    }

    #[test]
    #[serial]
    fn explicit_order_class_overrides_inference() {
        let client = paper_client();
        let request = client
            .limit_order("AAPL", Side::Sell, dec("150"))
            .qty(dec("10"))
            .take_profit(TakeProfit::new(dec("160")))
            .stop_loss(StopLoss::new(dec("145")))
            .order_class(OrderClass::Oco);
        let value = serde_json::to_value(&request).unwrap();
        assert_eq!(
            value.get("order_class").and_then(|v| v.as_str()),
            Some("oco")
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
