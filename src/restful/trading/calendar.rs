use crate::restful::TradingClient;
use chrono::NaiveDate;
use reqwest::Method;
use serde::{Deserialize, Serialize};

/// A market trading day.
#[derive(Clone, Debug, Deserialize)]
pub struct MarketDay {
    /// Calendar date.
    pub date: NaiveDate,
    /// Market open time (HH:MM format).
    pub open: String,
    /// Market close time (HH:MM format).
    pub close: String,
    /// Settlement date.
    #[serde(default)]
    pub settlement_date: Option<NaiveDate>,
}

/// Builder for requesting the market calendar.
#[derive(Debug, Serialize)]
#[must_use]
pub struct CalendarRequest<'a> {
    #[serde(skip)]
    client: &'a TradingClient,
    #[serde(skip_serializing_if = "Option::is_none")]
    start: Option<NaiveDate>,
    #[serde(skip_serializing_if = "Option::is_none")]
    end: Option<NaiveDate>,
}

impl CalendarRequest<'_> {
    /// Filter calendar days starting from this date.
    pub fn start(mut self, start: NaiveDate) -> Self {
        self.start = Some(start);
        self
    }

    /// Filter calendar days up to this date.
    pub fn end(mut self, end: NaiveDate) -> Self {
        self.end = Some(end);
        self
    }

    /// Execute the request.
    pub async fn execute(self) -> crate::Result<Vec<MarketDay>> {
        let request = self.client.request(Method::GET, "calendar").query(&self);
        self.client.send_and_deserialize(request).await
    }
}

impl TradingClient {
    /// Get the market calendar with optional date range.
    ///
    /// ```ignore
    /// let days = client.get_calendar()
    ///     .start(NaiveDate::from_ymd_opt(2024, 1, 1).unwrap())
    ///     .end(NaiveDate::from_ymd_opt(2024, 1, 31).unwrap())
    ///     .execute().await?;
    /// ```
    pub fn get_calendar(&self) -> CalendarRequest<'_> {
        CalendarRequest {
            client: self,
            start: None,
            end: None,
        }
    }
}
