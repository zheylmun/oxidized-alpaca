use crate::restful::{TradingClient, string_as_decimal, trading::assets::Exchange};
use crate::{AssetClass, AssetId};
use reqwest::Method;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// Per-symbol outcome from the bulk close-positions endpoint.
///
/// Alpaca's `DELETE /v2/positions` returns HTTP 207 with one of these
/// entries per symbol it tried to close. `status == 200` indicates the
/// close order was submitted; anything else is a failure with details
/// in `body`.
#[derive(Clone, Debug, Deserialize)]
#[non_exhaustive]
pub struct ClosePositionStatus {
    /// Symbol the close was attempted for.
    pub symbol: String,
    /// HTTP status code reported for this individual close.
    pub status: u16,
    /// Per-symbol response payload. Contains the submitted closing
    /// order on success, an error object on failure. Held as a raw
    /// JSON value so callers can decide how strictly to interpret it.
    #[serde(default)]
    pub body: Option<serde_json::Value>,
}

/// Side of a position.
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum PositionSide {
    /// Long position.
    Long,
    /// Short position.
    Short,
}

/// An open position as returned by the Alpaca API.
#[derive(Clone, Debug, Deserialize)]
#[non_exhaustive]
pub struct Position {
    /// Asset ID.
    pub asset_id: AssetId,
    /// Ticker symbol.
    pub symbol: String,
    /// Exchange the asset is traded on.
    pub exchange: Exchange,
    /// Asset class.
    pub asset_class: AssetClass,
    /// Whether the asset is marginable.
    pub asset_marginable: Option<bool>,
    /// Average entry price of the position.
    #[serde(deserialize_with = "string_as_decimal")]
    pub avg_entry_price: Decimal,
    /// Number of shares in the position.
    #[serde(deserialize_with = "string_as_decimal")]
    pub qty: Decimal,
    /// Number of shares available to trade.
    #[serde(deserialize_with = "string_as_decimal")]
    pub qty_available: Decimal,
    /// Long or short side.
    pub side: PositionSide,
    /// Current market value of the position.
    #[serde(deserialize_with = "string_as_decimal")]
    pub market_value: Decimal,
    /// Total cost basis.
    #[serde(deserialize_with = "string_as_decimal")]
    pub cost_basis: Decimal,
    /// Unrealized profit/loss.
    #[serde(deserialize_with = "string_as_decimal")]
    pub unrealized_pl: Decimal,
    /// Unrealized profit/loss percentage.
    #[serde(deserialize_with = "string_as_decimal")]
    pub unrealized_plpc: Decimal,
    /// Unrealized intraday profit/loss.
    #[serde(deserialize_with = "string_as_decimal")]
    pub unrealized_intraday_pl: Decimal,
    /// Unrealized intraday profit/loss percentage.
    #[serde(deserialize_with = "string_as_decimal")]
    pub unrealized_intraday_plpc: Decimal,
    /// Current asset price.
    #[serde(deserialize_with = "string_as_decimal")]
    pub current_price: Decimal,
    /// Previous trading day close price.
    #[serde(deserialize_with = "string_as_decimal")]
    pub lastday_price: Decimal,
    /// Percent change from previous day.
    #[serde(deserialize_with = "string_as_decimal")]
    pub change_today: Decimal,
}

/// Builder for closing a specific position.
#[derive(Debug, Serialize)]
#[must_use]
pub struct ClosePositionRequest<'a> {
    #[serde(skip)]
    client: &'a TradingClient,
    #[serde(skip)]
    symbol_or_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    qty: Option<Decimal>,
    #[serde(skip_serializing_if = "Option::is_none")]
    percentage: Option<Decimal>,
}

impl ClosePositionRequest<'_> {
    /// Close a specific number of shares.
    pub fn qty(mut self, qty: Decimal) -> Self {
        self.qty = Some(qty);
        self.percentage = None;
        self
    }

    /// Close a percentage of the position (0-100).
    pub fn percentage(mut self, percentage: Decimal) -> Self {
        self.percentage = Some(percentage);
        self.qty = None;
        self
    }

    /// Execute the close request.
    pub async fn execute(self) -> crate::Result<Order> {
        let symbol_or_id = &self.symbol_or_id;
        let path = format!("v2/positions/{symbol_or_id}");
        let request = self.client.request(Method::DELETE, &path)?.query(&self);
        self.client.send_and_deserialize(request).await
    }
}

use super::orders::Order;

/// Builder for closing every open position (`DELETE /v2/positions`).
#[derive(Debug, Serialize)]
#[must_use]
pub struct CloseAllPositionsRequest<'a> {
    #[serde(skip)]
    client: &'a TradingClient,
    #[serde(skip_serializing_if = "Option::is_none")]
    cancel_orders: Option<bool>,
}

impl CloseAllPositionsRequest<'_> {
    /// Also cancel all open orders before liquidating positions.
    pub fn cancel_orders(mut self, cancel_orders: bool) -> Self {
        self.cancel_orders = Some(cancel_orders);
        self
    }

    /// Submit the bulk close request.
    pub async fn execute(self) -> crate::Result<Vec<ClosePositionStatus>> {
        let request = self
            .client
            .request(Method::DELETE, "v2/positions")?
            .query(&self);
        self.client.send_and_deserialize(request).await
    }
}

impl TradingClient {
    /// List all open positions.
    pub async fn list_positions(&self) -> crate::Result<Vec<Position>> {
        let request = self.request(Method::GET, "v2/positions")?;
        self.send_and_deserialize(request).await
    }

    /// Get a specific open position by symbol or asset ID.
    pub async fn get_position(&self, symbol_or_id: &str) -> crate::Result<Position> {
        let request = self.request(Method::GET, &format!("v2/positions/{symbol_or_id}"))?;
        self.send_and_deserialize(request).await
    }

    /// Close a specific position by symbol or asset ID.
    ///
    /// ```ignore
    /// use rust_decimal_macros::dec;
    ///
    /// // Close 5 shares
    /// let order = client.close_position("AAPL").qty(dec!(5)).execute().await?;
    ///
    /// // Close 50% of position
    /// let order = client.close_position("AAPL").percentage(dec!(50)).execute().await?;
    /// ```
    pub fn close_position(&self, symbol_or_id: &str) -> ClosePositionRequest<'_> {
        ClosePositionRequest {
            client: self,
            symbol_or_id: symbol_or_id.to_string(),
            qty: None,
            percentage: None,
        }
    }

    /// Attempt to close every open position.
    ///
    /// Returns a builder; call [`CloseAllPositionsRequest::execute`] to send
    /// the request. Alpaca processes each position individually and returns a
    /// per-symbol outcome (HTTP 207); inspect each
    /// [`ClosePositionStatus::status`] to distinguish successful closes
    /// (`200`) from failures.
    pub fn close_all_positions(&self) -> CloseAllPositionsRequest<'_> {
        CloseAllPositionsRequest {
            client: self,
            cancel_orders: None,
        }
    }

    /// Exercise an options position.
    pub async fn exercise_option(&self, symbol_or_contract_id: &str) -> crate::Result<()> {
        let request = self.request(
            Method::POST,
            &format!("v2/positions/{symbol_or_contract_id}/exercise"),
        )?;
        self.send_no_body(request).await
    }

    /// Submit a do-not-exercise instruction for an options position.
    pub async fn do_not_exercise(&self, symbol_or_contract_id: &str) -> crate::Result<()> {
        let request = self.request(
            Method::POST,
            &format!("v2/positions/{symbol_or_contract_id}/do-not-exercise"),
        )?;
        self.send_no_body(request).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::AccountType;
    use serial_test::serial;
    use std::env;

    fn paper_client() -> TradingClient {
        unsafe {
            if env::var("ALPACA_PAPER_API_KEY_ID").is_err() {
                env::set_var("ALPACA_PAPER_API_KEY_ID", "test_key_id");
            }
            if env::var("ALPACA_PAPER_API_SECRET_KEY").is_err() {
                env::set_var("ALPACA_PAPER_API_SECRET_KEY", "test_secret_key");
            }
        }
        TradingClient::new(AccountType::Paper).unwrap()
    }

    #[test]
    #[serial]
    fn close_all_positions_cancel_orders_serializes_to_query() {
        let client = paper_client();
        let request = client.close_all_positions().cancel_orders(true);
        let query = serde_urlencoded::to_string(&request).unwrap();
        assert_eq!(query, "cancel_orders=true");
    }

    #[test]
    fn test_position_deserialization() {
        let json = r#"{
            "asset_id": "904837e3-3b76-47ec-b432-046db621571b",
            "symbol": "AAPL",
            "exchange": "NASDAQ",
            "asset_class": "us_equity",
            "asset_marginable": true,
            "avg_entry_price": "150.25",
            "qty": "10",
            "qty_available": "10",
            "side": "long",
            "market_value": "1525.00",
            "cost_basis": "1502.50",
            "unrealized_pl": "22.50",
            "unrealized_plpc": "0.0150",
            "unrealized_intraday_pl": "5.00",
            "unrealized_intraday_plpc": "0.0033",
            "current_price": "152.50",
            "lastday_price": "152.00",
            "change_today": "0.0033"
        }"#;
        let position: Position = serde_json::from_str(json).unwrap();
        assert_eq!(position.symbol, "AAPL");
        assert_eq!(position.side, PositionSide::Long);
        assert_eq!(
            position.avg_entry_price,
            Decimal::from_str_exact("150.25").unwrap()
        );
    }
}
