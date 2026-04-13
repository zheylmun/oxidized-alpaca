use crate::restful::MarketDataClient;
use reqwest::Method;
use serde::Deserialize;

/// Corporate action types returned by the API.
#[derive(Clone, Debug, Deserialize)]
pub struct CorporateActions {
    /// Forward stock splits.
    #[serde(default)]
    pub forward_splits: Vec<serde_json::Value>,
    /// Reverse stock splits.
    #[serde(default)]
    pub reverse_splits: Vec<serde_json::Value>,
    /// Cash dividend events.
    #[serde(default)]
    pub cash_dividends: Vec<serde_json::Value>,
    /// Stock dividend events.
    #[serde(default)]
    pub stock_dividends: Vec<serde_json::Value>,
    /// Cash merger events.
    #[serde(default)]
    pub cash_mergers: Vec<serde_json::Value>,
    /// Stock merger events.
    #[serde(default)]
    pub stock_mergers: Vec<serde_json::Value>,
    /// Stock-and-cash merger events.
    #[serde(default)]
    pub stock_and_cash_mergers: Vec<serde_json::Value>,
    /// Name change events.
    #[serde(default)]
    pub name_changes: Vec<serde_json::Value>,
    /// Spin-off events.
    #[serde(default)]
    pub spin_offs: Vec<serde_json::Value>,
    /// Redemption events.
    #[serde(default)]
    pub redemptions: Vec<serde_json::Value>,
}

impl MarketDataClient {
    /// Get corporate actions (splits, dividends, mergers, etc.).
    ///
    /// Returns raw JSON values for corporate action events since the schema
    /// varies significantly by action type.
    pub async fn corporate_actions(
        &self,
        symbols: Option<&str>,
        types: Option<&str>,
    ) -> crate::Result<CorporateActions> {
        let mut request = self.request(Method::GET, "v1/corporate-actions");
        if let Some(symbols) = symbols {
            request = request.query(&[("symbols", symbols)]);
        }
        if let Some(types) = types {
            request = request.query(&[("types", types)]);
        }
        self.send_and_deserialize(request).await
    }
}
