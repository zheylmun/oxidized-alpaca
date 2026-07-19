use crate::restful::MarketDataClient;
use reqwest::Method;
use serde::Deserialize;

use super::{CryptoLocation, CryptoOrderbook};

#[derive(Debug, Deserialize)]
struct OrderbooksResponse {
    orderbooks: std::collections::HashMap<String, CryptoOrderbook>,
}

impl MarketDataClient {
    /// Get the latest crypto orderbooks.
    pub async fn crypto_latest_orderbooks(
        &self,
        symbols: &[&str],
        loc: CryptoLocation,
    ) -> crate::Result<std::collections::HashMap<String, CryptoOrderbook>> {
        let path = format!("v1beta3/crypto/{loc}/latest/orderbooks");
        let request = self
            .request(Method::GET, &path)?
            .query(&[("symbols", symbols.join(","))]);
        let response: OrderbooksResponse = self.send_and_deserialize(request).await?;
        Ok(response.orderbooks)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserializes_orderbooks_response() {
        let json = r#"{
            "orderbooks": {
                "BTC/USD": {
                    "t": "2026-05-07T13:30:00.015531758Z",
                    "b": [{"p": 103250.0, "s": 0.5}, {"p": 103249.0, "s": 1.25}],
                    "a": [{"p": 103251.0, "s": 0.4}],
                    "r": true
                }
            }
        }"#;
        let parsed: OrderbooksResponse = serde_json::from_str(json).unwrap();
        let book = &parsed.orderbooks["BTC/USD"];
        assert_eq!(book.bids.len(), 2);
        assert_eq!(book.bids[0].price, 103_250.0);
        assert_eq!(book.bids[1].size, 1.25);
        assert_eq!(book.asks.len(), 1);
        assert!(book.reset);
    }

    #[test]
    fn orderbook_reset_defaults_to_false_when_absent() {
        let json = r#"{
            "orderbooks": {
                "ETH/USD": {"t": "2026-05-07T13:30:00Z", "b": [], "a": []}
            }
        }"#;
        let parsed: OrderbooksResponse = serde_json::from_str(json).unwrap();
        assert!(!parsed.orderbooks["ETH/USD"].reset);
    }
}
