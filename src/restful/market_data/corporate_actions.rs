use crate::restful::MarketDataClient;
use reqwest::Method;
use serde::Deserialize;

/// Categories of corporate action accepted by the `types` filter.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum CorporateActionType {
    /// Forward stock split.
    ForwardSplit,
    /// Reverse stock split.
    ReverseSplit,
    /// Unit split.
    UnitSplit,
    /// Cash dividend.
    CashDividend,
    /// Stock dividend.
    StockDividend,
    /// Spin-off.
    SpinOff,
    /// Cash merger.
    CashMerger,
    /// Stock merger.
    StockMerger,
    /// Stock-and-cash merger.
    StockAndCashMerger,
    /// Redemption.
    Redemption,
    /// Name change.
    NameChange,
    /// Worthless removal.
    WorthlessRemoval,
    /// Rights distribution.
    RightsDistribution,
    /// Contract adjustment.
    ContractAdjustment,
    /// Partial call.
    PartialCall,
}

impl CorporateActionType {
    fn as_str(&self) -> &'static str {
        match self {
            Self::ForwardSplit => "forward_split",
            Self::ReverseSplit => "reverse_split",
            Self::UnitSplit => "unit_split",
            Self::CashDividend => "cash_dividend",
            Self::StockDividend => "stock_dividend",
            Self::SpinOff => "spin_off",
            Self::CashMerger => "cash_merger",
            Self::StockMerger => "stock_merger",
            Self::StockAndCashMerger => "stock_and_cash_merger",
            Self::Redemption => "redemption",
            Self::NameChange => "name_change",
            Self::WorthlessRemoval => "worthless_removal",
            Self::RightsDistribution => "rights_distribution",
            Self::ContractAdjustment => "contract_adjustment",
            Self::PartialCall => "partial_call",
        }
    }
}

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
    /// varies significantly by action type. Pass an empty slice for either
    /// `symbols` or `types` to omit that filter.
    pub async fn corporate_actions(
        &self,
        symbols: &[&str],
        types: &[CorporateActionType],
    ) -> crate::Result<CorporateActions> {
        let mut request = self.request(Method::GET, "v1/corporate-actions");
        if !symbols.is_empty() {
            request = request.query(&[("symbols", symbols.join(","))]);
        }
        if !types.is_empty() {
            let joined: Vec<&str> = types.iter().map(CorporateActionType::as_str).collect();
            request = request.query(&[("types", joined.join(","))]);
        }
        self.send_and_deserialize(request).await
    }
}
