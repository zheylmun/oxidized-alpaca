use crate::restful::{TradingClient, string_as_decimal};
use reqwest::Method;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

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
pub struct Position {
    /// Asset ID.
    pub asset_id: String,
    /// Ticker symbol.
    pub symbol: String,
    /// Exchange the asset is traded on.
    pub exchange: String,
    /// Asset class (e.g., "us_equity").
    pub asset_class: String,
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
        let path = format!("positions/{symbol_or_id}");
        let request = self.client.request(Method::DELETE, &path).query(&self);
        self.client.send_and_deserialize(request).await
    }
}

use super::orders::Order;

impl TradingClient {
    /// List all open positions.
    pub async fn list_positions(&self) -> crate::Result<Vec<Position>> {
        let request = self.request(Method::GET, "positions");
        self.send_and_deserialize(request).await
    }

    /// Get a specific open position by symbol or asset ID.
    pub async fn get_position(&self, symbol_or_id: &str) -> crate::Result<Position> {
        let request = self.request(Method::GET, &format!("positions/{symbol_or_id}"));
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

    /// Close all open positions.
    pub async fn close_all_positions(&self) -> crate::Result<Vec<Order>> {
        let request = self.request(Method::DELETE, "positions");
        self.send_and_deserialize(request).await
    }

    /// Exercise an options position.
    pub async fn exercise_option(&self, symbol_or_contract_id: &str) -> crate::Result<()> {
        let request = self.request(
            Method::POST,
            &format!("positions/{symbol_or_contract_id}/exercise"),
        );
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

    /// Submit a do-not-exercise instruction for an options position.
    pub async fn do_not_exercise(&self, symbol_or_contract_id: &str) -> crate::Result<()> {
        let request = self.request(
            Method::POST,
            &format!("positions/{symbol_or_contract_id}/do-not-exercise"),
        );
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
}

#[cfg(test)]
mod tests {
    use super::*;

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
