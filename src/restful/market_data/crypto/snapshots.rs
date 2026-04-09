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
        symbols: &str,
        loc: CryptoLocation,
    ) -> crate::Result<std::collections::HashMap<String, CryptoSnapshot>> {
        let path = format!("v1beta3/crypto/{}/snapshots", loc.as_str());
        let request = self
            .request(Method::GET, &path)
            .query(&[("symbols", symbols)]);
        let response: SnapshotsResponse = self.send_and_deserialize(request).await?;
        Ok(response.snapshots)
    }
}
