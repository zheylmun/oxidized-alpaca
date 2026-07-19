use crate::restful::MarketDataClient;
use reqwest::Method;
use serde::Deserialize;

use super::{CryptoLocation, CryptoSnapshot};

#[derive(Debug, Deserialize)]
struct SnapshotsResponse {
    snapshots: std::collections::HashMap<String, CryptoSnapshot>,
}

impl MarketDataClient {
    /// Get crypto snapshots.
    pub async fn crypto_snapshots(
        &self,
        symbols: &[&str],
        loc: CryptoLocation,
    ) -> crate::Result<std::collections::HashMap<String, CryptoSnapshot>> {
        let path = format!("v1beta3/crypto/{loc}/snapshots");
        let request = self
            .request(Method::GET, &path)?
            .query(&[("symbols", symbols.join(","))]);
        let response: SnapshotsResponse = self.send_and_deserialize(request).await?;
        Ok(response.snapshots)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserializes_snapshots_response() {
        let json = r#"{
            "snapshots": {
                "BTC/USD": {
                    "latestTrade": {"t":"2026-05-07T13:30:00Z","p":103250.5,"s":0.014,"i":12345,"tks":"S"},
                    "latestQuote": {"t":"2026-05-07T13:30:00Z","bp":103250.0,"bs":0.5,"ap":103251.0,"as":0.4},
                    "minuteBar": {"t":"2026-05-07T13:29:00Z","o":103200.0,"h":103260.0,"l":103190.0,"c":103250.0,"v":12.5,"n":420,"vw":103225.0},
                    "dailyBar": {"t":"2026-05-07T00:00:00Z","o":102000.0,"h":103500.0,"l":101800.0,"c":103250.0,"v":900.0,"n":50000,"vw":102900.0},
                    "prevDailyBar": null
                }
            }
        }"#;
        let parsed: SnapshotsResponse = serde_json::from_str(json).unwrap();
        let snapshot = &parsed.snapshots["BTC/USD"];
        assert_eq!(
            snapshot.latest_trade.as_ref().unwrap().taker_side,
            Some(crate::CryptoTakerSide::Seller)
        );
        assert_eq!(snapshot.latest_quote.as_ref().unwrap().ask_size, 0.4);
        assert_eq!(snapshot.minute_bar.as_ref().unwrap().trade_count, 420);
        assert_eq!(snapshot.daily_bar.as_ref().unwrap().open, 102_000.0);
        assert!(snapshot.prev_daily_bar.is_none());
    }

    #[test]
    fn deserializes_snapshot_with_all_sections_absent() {
        let json = r#"{"snapshots": {"ETH/USD": {}}}"#;
        let parsed: SnapshotsResponse = serde_json::from_str(json).unwrap();
        let snapshot = &parsed.snapshots["ETH/USD"];
        assert!(snapshot.latest_trade.is_none());
        assert!(snapshot.minute_bar.is_none());
    }
}
