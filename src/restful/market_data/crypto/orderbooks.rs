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
