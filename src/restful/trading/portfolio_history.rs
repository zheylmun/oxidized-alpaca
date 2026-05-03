use crate::restful::TradingClient;
use chrono::{DateTime, Utc};
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

/// Period over which portfolio history should be reported.
#[derive(Clone, Copy, Debug, Serialize, PartialEq, Eq)]
#[non_exhaustive]
pub enum HistoryPeriod {
    /// One day.
    #[serde(rename = "1D")]
    OneDay,
    /// One week.
    #[serde(rename = "1W")]
    OneWeek,
    /// One month.
    #[serde(rename = "1M")]
    OneMonth,
    /// Three months.
    #[serde(rename = "3M")]
    ThreeMonths,
    /// Six months.
    #[serde(rename = "6M")]
    SixMonths,
    /// One year (alias for 1A in the Alpaca API).
    #[serde(rename = "1A")]
    OneYear,
}

/// Sample resolution for portfolio history data points.
#[derive(Clone, Copy, Debug, Serialize, PartialEq, Eq)]
#[non_exhaustive]
pub enum HistoryTimeFrame {
    /// One-minute samples.
    #[serde(rename = "1Min")]
    OneMinute,
    /// Five-minute samples.
    #[serde(rename = "5Min")]
    FiveMinutes,
    /// Fifteen-minute samples.
    #[serde(rename = "15Min")]
    FifteenMinutes,
    /// One-hour samples.
    #[serde(rename = "1H")]
    OneHour,
    /// One-day samples.
    #[serde(rename = "1D")]
    OneDay,
}

/// Intraday reporting window for portfolio history.
#[derive(Clone, Copy, Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum IntradayReporting {
    /// Only include data points during regular market hours.
    MarketHours,
    /// Include extended-hours data points.
    ExtendedHours,
    /// Continuous reporting across the calendar day.
    Continuous,
}

/// Mode for resetting profit/loss accumulation in the portfolio history.
#[derive(Clone, Copy, Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum PnlReset {
    /// Reset P/L at the start of every trading day.
    PerDay,
    /// Never reset P/L; accumulate across the queried range.
    NoReset,
}

/// Builder for requesting portfolio history.
#[derive(Debug, Serialize)]
#[must_use]
pub struct PortfolioHistoryRequest<'a> {
    #[serde(skip)]
    client: &'a TradingClient,
    #[serde(skip_serializing_if = "Option::is_none")]
    period: Option<HistoryPeriod>,
    #[serde(skip_serializing_if = "Option::is_none")]
    timeframe: Option<HistoryTimeFrame>,
    #[serde(skip_serializing_if = "Option::is_none")]
    intraday_reporting: Option<IntradayReporting>,
    #[serde(skip_serializing_if = "Option::is_none")]
    start: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    end: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pnl_reset: Option<PnlReset>,
}

impl PortfolioHistoryRequest<'_> {
    /// Set the reporting period.
    pub fn period(mut self, period: HistoryPeriod) -> Self {
        self.period = Some(period);
        self
    }

    /// Set the resolution of each data point.
    pub fn timeframe(mut self, timeframe: HistoryTimeFrame) -> Self {
        self.timeframe = Some(timeframe);
        self
    }

    /// Set the intraday reporting window.
    pub fn intraday_reporting(mut self, reporting: IntradayReporting) -> Self {
        self.intraday_reporting = Some(reporting);
        self
    }

    /// Filter samples to those at or after this timestamp.
    pub fn start(mut self, start: DateTime<Utc>) -> Self {
        self.start = Some(start);
        self
    }

    /// Filter samples to those at or before this timestamp.
    pub fn end(mut self, end: DateTime<Utc>) -> Self {
        self.end = Some(end);
        self
    }

    /// Set how P/L is reset across the queried range.
    pub fn pnl_reset(mut self, mode: PnlReset) -> Self {
        self.pnl_reset = Some(mode);
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
    ///     .period(HistoryPeriod::OneMonth)
    ///     .timeframe(HistoryTimeFrame::OneDay)
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
