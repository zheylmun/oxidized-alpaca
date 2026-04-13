use crate::restful::TradingClient;
use reqwest::Method;
use serde::{Deserialize, Serialize};

/// Portfolio history response.
#[derive(Clone, Debug, Deserialize)]
pub struct PortfolioHistory {
    /// Unix timestamps for each data point.
    pub timestamp: Vec<i64>,
    /// Equity values at each timestamp.
    pub equity: Vec<f64>,
    /// Profit/loss values at each timestamp.
    pub profit_loss: Vec<f64>,
    /// Profit/loss percentage at each timestamp.
    pub profit_loss_pct: Vec<f64>,
    /// Base portfolio value.
    pub base_value: f64,
    /// Timeframe of the data points.
    pub timeframe: String,
}

/// Builder for requesting portfolio history.
#[derive(Debug, Serialize)]
#[must_use]
pub struct PortfolioHistoryRequest<'a> {
    #[serde(skip)]
    client: &'a TradingClient,
    #[serde(skip_serializing_if = "Option::is_none")]
    period: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    timeframe: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    intraday_reporting: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    start: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    end: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pnl_reset: Option<String>,
}

impl PortfolioHistoryRequest<'_> {
    /// Set the period (e.g., "1D", "1W", "1M", "3M", "1A").
    pub fn period(mut self, period: &str) -> Self {
        self.period = Some(period.to_string());
        self
    }

    /// Set the timeframe for data points (e.g., "1Min", "5Min", "15Min", "1H", "1D").
    pub fn timeframe(mut self, timeframe: &str) -> Self {
        self.timeframe = Some(timeframe.to_string());
        self
    }

    /// Set intraday reporting mode ("market_hours" or "extended_hours" or "continuous").
    pub fn intraday_reporting(mut self, reporting: &str) -> Self {
        self.intraday_reporting = Some(reporting.to_string());
        self
    }

    /// Set start date (RFC 3339 or date string).
    pub fn start(mut self, start: &str) -> Self {
        self.start = Some(start.to_string());
        self
    }

    /// Set end date (RFC 3339 or date string).
    pub fn end(mut self, end: &str) -> Self {
        self.end = Some(end.to_string());
        self
    }

    /// Set P/L reset mode ("per_day" or "no_reset").
    pub fn pnl_reset(mut self, mode: &str) -> Self {
        self.pnl_reset = Some(mode.to_string());
        self
    }

    /// Execute the request.
    pub async fn execute(self) -> crate::Result<PortfolioHistory> {
        let request = self
            .client
            .request(Method::GET, "account/portfolio/history")
            .query(&self);
        self.client.send_and_deserialize(request).await
    }
}

impl TradingClient {
    /// Request portfolio history.
    ///
    /// ```ignore
    /// let history = client.portfolio_history()
    ///     .period("1M")
    ///     .timeframe("1D")
    ///     .execute().await?;
    /// ```
    pub fn portfolio_history(&self) -> PortfolioHistoryRequest<'_> {
        PortfolioHistoryRequest {
            client: self,
            period: None,
            timeframe: None,
            intraday_reporting: None,
            start: None,
            end: None,
            pnl_reset: None,
        }
    }
}
