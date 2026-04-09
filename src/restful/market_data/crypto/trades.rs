use crate::restful::MarketDataClient;
use reqwest::Method;
use serde::Deserialize;

use super::{CryptoLocation, CryptoTrade};

#[derive(Debug, Deserialize)]
struct LatestTradesResponse {
    trades: std::collections::HashMap<String, CryptoTrade>,
}

impl MarketDataClient {
    /// Get the latest crypto trades.
    pub async fn crypto_latest_trades(
        &self,
        symbols: &str,
        loc: CryptoLocation,
    ) -> crate::Result<std::collections::HashMap<String, CryptoTrade>> {
        let path = format!("v1beta3/crypto/{}/latest/trades", loc.as_str());
        let request = self
            .request(Method::GET, &path)
            .query(&[("symbols", symbols)]);
        let response: LatestTradesResponse = self.send_and_deserialize(request).await?;
        Ok(response.trades)
    }
}
