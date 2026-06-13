/// Option bars endpoint types and methods.
pub mod bars;
/// Option quotes endpoint types and methods.
pub mod quotes;
/// Option snapshots endpoint types and methods.
pub mod snapshots;
/// Option trades endpoint types and methods.
pub mod trades;

use chrono::{DateTime, Utc};
use serde::Deserialize;

/// An option bar (OHLCV).
#[derive(Clone, Debug, Deserialize, PartialEq)]
#[non_exhaustive]
pub struct OptionBar {
    /// The bar timestamp.
    #[serde(rename = "t")]
    pub timestamp: DateTime<Utc>,
    /// The open price.
    #[serde(rename = "o")]
    pub open: f64,
    /// The highest price.
    #[serde(rename = "h")]
    pub high: f64,
    /// The lowest price.
    #[serde(rename = "l")]
    pub low: f64,
    /// The close price.
    #[serde(rename = "c")]
    pub close: f64,
    /// The trading volume.
    #[serde(rename = "v")]
    pub volume: i64,
    /// The number of trades.
    #[serde(rename = "n")]
    pub trade_count: i64,
    /// The volume-weighted average price.
    #[serde(rename = "vw")]
    pub vwap: f64,
}

/// An option trade.
#[derive(Clone, Debug, Deserialize, PartialEq)]
#[non_exhaustive]
pub struct OptionTrade {
    /// The trade timestamp.
    #[serde(rename = "t")]
    pub timestamp: DateTime<Utc>,
    /// The exchange code.
    #[serde(rename = "x")]
    pub exchange: String,
    /// The trade price.
    #[serde(rename = "p")]
    pub price: f64,
    /// The trade size.
    #[serde(rename = "s")]
    pub size: u32,
    /// The trade condition.
    #[serde(rename = "c", default)]
    pub condition: Option<String>,
}

/// An option quote.
#[derive(Clone, Debug, Deserialize, PartialEq)]
#[non_exhaustive]
pub struct OptionQuote {
    /// The quote timestamp.
    #[serde(rename = "t")]
    pub timestamp: DateTime<Utc>,
    /// The bid exchange code.
    #[serde(rename = "bx")]
    pub bid_exchange: String,
    /// The bid price.
    #[serde(rename = "bp")]
    pub bid_price: f64,
    /// The bid size.
    #[serde(rename = "bs")]
    pub bid_size: u32,
    /// The ask exchange code.
    #[serde(rename = "ax")]
    pub ask_exchange: String,
    /// The ask price.
    #[serde(rename = "ap")]
    pub ask_price: f64,
    /// The ask size.
    #[serde(rename = "as")]
    pub ask_size: u32,
    /// The quote condition.
    #[serde(rename = "c", default)]
    pub condition: Option<String>,
}

/// Option greeks.
#[derive(Clone, Debug, Deserialize)]
#[non_exhaustive]
pub struct OptionGreeks {
    /// Delta value.
    pub delta: Option<f64>,
    /// Gamma value.
    pub gamma: Option<f64>,
    /// Theta value.
    pub theta: Option<f64>,
    /// Vega value.
    pub vega: Option<f64>,
    /// Rho value.
    pub rho: Option<f64>,
}

/// An option snapshot.
#[derive(Clone, Debug, Deserialize)]
#[non_exhaustive]
pub struct OptionSnapshot {
    /// The latest trade.
    #[serde(rename = "latestTrade")]
    pub latest_trade: Option<OptionTrade>,
    /// The latest quote.
    #[serde(rename = "latestQuote")]
    pub latest_quote: Option<OptionQuote>,
    /// The option greeks.
    #[serde(default)]
    pub greeks: Option<OptionGreeks>,
    /// The implied volatility.
    #[serde(rename = "impliedVolatility", default)]
    pub implied_volatility: Option<f64>,
    /// The latest minute bar.
    #[serde(rename = "minuteBar", default)]
    pub minute_bar: Option<OptionBar>,
    /// The latest daily bar.
    #[serde(rename = "dailyBar", default)]
    pub daily_bar: Option<OptionBar>,
    /// The previous daily bar.
    #[serde(rename = "prevDailyBar", default)]
    pub prev_daily_bar: Option<OptionBar>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn option_snapshot_deserializes_bars() {
        let json = r#"{
            "latestQuote": {"t":"2026-05-08T19:59:59Z","ax":"N","ap":1.2,"as":10,"bx":"N","bp":1.1,"bs":12,"c":"A"},
            "dailyBar": {"t":"2026-05-08T04:00:00Z","o":1.0,"h":1.3,"l":0.9,"c":1.2,"v":1500,"n":42,"vw":1.15},
            "minuteBar": {"t":"2026-05-08T19:59:00Z","o":1.18,"h":1.21,"l":1.17,"c":1.2,"v":30,"n":5,"vw":1.19},
            "prevDailyBar": {"t":"2026-05-07T04:00:00Z","o":0.95,"h":1.05,"l":0.9,"c":1.0,"v":1200,"n":33,"vw":0.98}
        }"#;
        let snapshot: OptionSnapshot = serde_json::from_str(json).unwrap();
        assert_eq!(snapshot.daily_bar.as_ref().unwrap().close, 1.2);
        assert_eq!(snapshot.minute_bar.as_ref().unwrap().trade_count, 5);
        assert_eq!(snapshot.prev_daily_bar.as_ref().unwrap().open, 0.95);
    }
}
