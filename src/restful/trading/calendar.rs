use crate::restful::TradingClient;
use chrono::{NaiveDate, NaiveTime};
use reqwest::Method;
use serde::{Deserialize, Serialize};

/// A market trading day.
#[derive(Clone, Debug, Deserialize)]
pub struct MarketDay {
    /// Calendar date.
    pub date: NaiveDate,
    /// Market open time (`HH:MM`).
    #[serde(deserialize_with = "deserialize_hhmm")]
    pub open: NaiveTime,
    /// Market close time (`HH:MM`).
    #[serde(deserialize_with = "deserialize_hhmm")]
    pub close: NaiveTime,
    /// Settlement date.
    #[serde(default)]
    pub settlement_date: Option<NaiveDate>,
}

fn deserialize_hhmm<'de, D>(deserializer: D) -> Result<NaiveTime, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let raw = String::deserialize(deserializer)?;
    NaiveTime::parse_from_str(&raw, "%H:%M").map_err(serde::de::Error::custom)
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
        let request = self
            .client
            .request(Method::GET, "v2/calendar")?
            .query(&self);
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
