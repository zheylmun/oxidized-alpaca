use crate::restful::MarketDataClient;
use reqwest::Method;

/// Tick type for condition code lookups.
#[derive(Clone, Copy, Debug)]
pub enum TickType {
    Trade,
    Quote,
}

impl TickType {
    fn as_str(&self) -> &str {
        match self {
            Self::Trade => "trade",
            Self::Quote => "quote",
        }
    }
}

impl MarketDataClient {
    /// Get stock trade or quote condition codes.
    pub async fn stock_conditions(
        &self,
        tick_type: TickType,
    ) -> crate::Result<std::collections::HashMap<String, String>> {
        let path = format!("v2/stocks/meta/conditions/{}", tick_type.as_str());
        let request = self.request(Method::GET, &path);
        self.send_and_deserialize(request).await
    }

    /// Get stock exchange codes.
    pub async fn stock_exchanges(
        &self,
    ) -> crate::Result<std::collections::HashMap<String, String>> {
        let request = self.request(Method::GET, "v2/stocks/meta/exchanges");
        self.send_and_deserialize(request).await
    }
}
