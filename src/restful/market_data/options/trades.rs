use crate::restful::MarketDataClient;
use reqwest::Method;
use serde::Deserialize;

use super::OptionTrade;

#[derive(Debug, Deserialize)]
struct LatestTradesResponse {
    trades: std::collections::HashMap<String, OptionTrade>,
}

impl MarketDataClient {
    /// Get the latest option trades.
    pub async fn option_latest_trades(
        &self,
        symbols: &str,
    ) -> crate::Result<std::collections::HashMap<String, OptionTrade>> {
        let request = self
            .request(Method::GET, "v1beta1/options/trades/latest")
            .query(&[("symbols", symbols)]);
        let response: LatestTradesResponse = self.send_and_deserialize(request).await?;
        Ok(response.trades)
    }
}
