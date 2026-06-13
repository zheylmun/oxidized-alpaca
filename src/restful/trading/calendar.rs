use crate::restful::TradingClient;
use chrono::{NaiveDate, NaiveTime};
use reqwest::Method;
use serde::{Deserialize, Serialize};

/// A market trading day.
#[derive(Clone, Debug, Deserialize)]
#[non_exhaustive]
pub struct MarketDay {
    /// Calendar date.
    pub date: NaiveDate,
    /// Market open time (`HH:MM`).
    #[serde(deserialize_with = "deserialize_hhmm")]
    pub open: NaiveTime,
    /// Market close time (`HH:MM`).
    #[serde(deserialize_with = "deserialize_hhmm")]
    pub close: NaiveTime,
    /// Session (extended-hours) open time (`HHMM`).
    #[serde(default, deserialize_with = "deserialize_opt_hhmm_compact")]
    pub session_open: Option<NaiveTime>,
    /// Session (extended-hours) close time (`HHMM`).
    #[serde(default, deserialize_with = "deserialize_opt_hhmm_compact")]
    pub session_close: Option<NaiveTime>,
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

fn deserialize_opt_hhmm_compact<'de, D>(deserializer: D) -> Result<Option<NaiveTime>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let opt = Option::<String>::deserialize(deserializer)?;
    match opt {
        Some(raw) => NaiveTime::parse_from_str(&raw, "%H%M")
            .map(Some)
            .map_err(serde::de::Error::custom),
        None => Ok(None),
    }
}

/// Whether calendar days are filtered by trading date or settlement date.
#[derive(Clone, Copy, Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "UPPERCASE")]
#[non_exhaustive]
pub enum CalendarDateType {
    /// Filter by trading date.
    Trading,
    /// Filter by settlement date.
    Settlement,
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
    #[serde(skip_serializing_if = "Option::is_none")]
    date_type: Option<CalendarDateType>,
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

    /// Filter by trading date or settlement date (defaults to trading).
    pub fn date_type(mut self, date_type: CalendarDateType) -> Self {
        self.date_type = Some(date_type);
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
            date_type: None,
        }
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
    fn deserializes_market_day_with_session_times() {
        let json = r#"{
            "date": "2025-01-02",
            "open": "09:30",
            "close": "16:00",
            "session_open": "0400",
            "session_close": "2000",
            "settlement_date": "2025-01-03"
        }"#;
        let day: MarketDay = serde_json::from_str(json).unwrap();
        assert_eq!(day.open, NaiveTime::from_hms_opt(9, 30, 0).unwrap());
        assert_eq!(day.session_open, NaiveTime::from_hms_opt(4, 0, 0));
        assert_eq!(day.session_close, NaiveTime::from_hms_opt(20, 0, 0));
    }

    #[test]
    #[serial]
    fn date_type_serializes_to_query() {
        let client = paper_client();
        let request = client
            .get_calendar()
            .date_type(CalendarDateType::Settlement);
        let query = serde_urlencoded::to_string(&request).unwrap();
        assert_eq!(query, "date_type=SETTLEMENT");
    }
}
