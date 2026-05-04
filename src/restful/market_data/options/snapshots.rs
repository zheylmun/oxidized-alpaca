use crate::restful::MarketDataClient;
use reqwest::Method;
use serde::Deserialize;

use super::OptionSnapshot;

#[derive(Debug, Deserialize)]
struct SnapshotsResponse {
    snapshots: std::collections::HashMap<String, OptionSnapshot>,
}

impl MarketDataClient {
    /// Get option snapshots for given symbols.
    pub async fn option_snapshots(
        &self,
        symbols: &[&str],
    ) -> crate::Result<std::collections::HashMap<String, OptionSnapshot>> {
        let request = self
            .request(Method::GET, "v1beta1/options/snapshots")?
            .query(&[("symbols", symbols.join(","))]);
        let response: SnapshotsResponse = self.send_and_deserialize(request).await?;
        Ok(response.snapshots)
    }

    /// Get the option chain (all snapshots for an underlying symbol).
    pub async fn option_chain(
        &self,
        underlying_symbol: &str,
    ) -> crate::Result<std::collections::HashMap<String, OptionSnapshot>> {
        let path = format!("v1beta1/options/snapshots/{underlying_symbol}");
        let request = self.request(Method::GET, &path)?;
        let response: SnapshotsResponse = self.send_and_deserialize(request).await?;
        Ok(response.snapshots)
    }
}
