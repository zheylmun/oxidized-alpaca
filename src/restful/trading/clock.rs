use crate::restful::TradingClient;
use chrono::{DateTime, Utc};
use reqwest::Method;
use serde::Deserialize;

/// Current market clock status.
#[derive(Clone, Debug, Deserialize)]
pub struct Clock {
    /// Current timestamp.
    pub timestamp: DateTime<Utc>,
    /// Whether the market is currently open.
    pub is_open: bool,
    /// Next market open time.
    pub next_open: DateTime<Utc>,
    /// Next market close time.
    pub next_close: DateTime<Utc>,
}

impl TradingClient {
    /// Get the current market clock.
    ///
    /// Returns whether the market is open and when it next opens/closes.
    pub async fn get_clock(&self) -> crate::Result<Clock> {
        let request = self.request(Method::GET, "v2/clock")?;
        self.send_and_deserialize(request).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clock_deserialization() {
        let json = r#"{
            "timestamp": "2024-01-15T10:30:00Z",
            "is_open": true,
            "next_open": "2024-01-16T14:30:00Z",
            "next_close": "2024-01-15T21:00:00Z"
        }"#;
        let clock: Clock = serde_json::from_str(json).unwrap();
        assert!(clock.is_open);
    }
}
