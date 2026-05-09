use crate::restful::{MarketDataClient, SortDirection};
use chrono::NaiveDate;
use reqwest::Method;
use serde::{Deserialize, Serialize};

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
///
/// Each event still ships as a [`serde_json::Value`] because Alpaca's per-type
/// payloads diverge significantly; pull individual fields from the value as
/// needed (e.g. `event["id"]` for the action's stable identifier, used as
/// input to the [`CorporateActionsRequest::ids`] filter).
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

/// Builder for `/v1/corporate-actions`.
#[derive(Debug, Serialize)]
#[must_use]
pub struct CorporateActionsRequest<'a> {
    #[serde(skip)]
    client: &'a MarketDataClient,
    #[serde(skip_serializing_if = "Option::is_none")]
    symbols: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    types: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    ids: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    start: Option<NaiveDate>,
    #[serde(skip_serializing_if = "Option::is_none")]
    end: Option<NaiveDate>,
    #[serde(skip_serializing_if = "Option::is_none")]
    limit: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    sort: Option<SortDirection>,
}

impl CorporateActionsRequest<'_> {
    /// Filter to events that touch any of the given stock symbols.
    pub fn symbols(mut self, symbols: &[&str]) -> Self {
        self.symbols = if symbols.is_empty() {
            None
        } else {
            Some(symbols.join(","))
        };
        self
    }

    /// Filter to events of the given action types.
    pub fn types(mut self, types: &[CorporateActionType]) -> Self {
        self.types = if types.is_empty() {
            None
        } else {
            let joined: Vec<&str> = types.iter().map(CorporateActionType::as_str).collect();
            Some(joined.join(","))
        };
        self
    }

    /// Filter to events with one of the given Alpaca-issued action IDs.
    /// IDs come from the `id` field on each event payload returned by a
    /// previous call.
    pub fn ids(mut self, ids: &[&str]) -> Self {
        self.ids = if ids.is_empty() {
            None
        } else {
            Some(ids.join(","))
        };
        self
    }

    /// Only return events on or after this calendar date.
    pub fn start(mut self, start: NaiveDate) -> Self {
        self.start = Some(start);
        self
    }

    /// Only return events on or before this calendar date.
    pub fn end(mut self, end: NaiveDate) -> Self {
        self.end = Some(end);
        self
    }

    /// Cap the number of events returned.
    pub fn limit(mut self, limit: u32) -> Self {
        self.limit = Some(limit);
        self
    }

    /// Set the result sort order.
    pub fn sort(mut self, sort: SortDirection) -> Self {
        self.sort = Some(sort);
        self
    }

    /// Execute the request.
    pub async fn execute(self) -> crate::Result<CorporateActions> {
        let request = self
            .client
            .request(Method::GET, "v1/corporate-actions")?
            .query(&self);
        self.client.send_and_deserialize(request).await
    }
}

impl MarketDataClient {
    /// Build a request for corporate actions (splits, dividends, mergers,
    /// etc.). Filters are optional; an empty filter call returns the full
    /// available dataset for the queried window.
    ///
    /// ```ignore
    /// let actions = client.corporate_actions()
    ///     .symbols(&["AAPL"])
    ///     .types(&[CorporateActionType::CashDividend])
    ///     .start(NaiveDate::from_ymd_opt(2025, 1, 1).unwrap())
    ///     .execute().await?;
    /// ```
    pub fn corporate_actions(&self) -> CorporateActionsRequest<'_> {
        CorporateActionsRequest {
            client: self,
            symbols: None,
            types: None,
            ids: None,
            start: None,
            end: None,
            limit: None,
            sort: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::AccountType;
    use serial_test::serial;
    use std::env;

    fn ensure_paper_creds() {
        unsafe {
            if env::var("ALPACA_PAPER_API_KEY_ID").is_err() {
                env::set_var("ALPACA_PAPER_API_KEY_ID", "test_key_id");
            }
            if env::var("ALPACA_PAPER_API_SECRET_KEY").is_err() {
                env::set_var("ALPACA_PAPER_API_SECRET_KEY", "test_secret_key");
            }
        }
    }

    fn paper_client() -> MarketDataClient {
        ensure_paper_creds();
        MarketDataClient::new(AccountType::Paper).unwrap()
    }

    #[test]
    #[serial]
    fn empty_builder_serializes_to_empty_query() {
        let client = paper_client();
        let request = client.corporate_actions();
        let query = serde_urlencoded::to_string(&request).unwrap();
        assert!(query.is_empty(), "expected no query params; got {query}");
    }

    #[test]
    #[serial]
    fn builder_setters_serialize_to_query() {
        let client = paper_client();
        let request = client
            .corporate_actions()
            .symbols(&["AAPL", "MSFT"])
            .types(&[
                CorporateActionType::CashDividend,
                CorporateActionType::SpinOff,
            ])
            .ids(&["abc-1", "def-2"])
            .start(NaiveDate::from_ymd_opt(2025, 1, 1).unwrap())
            .end(NaiveDate::from_ymd_opt(2025, 12, 31).unwrap())
            .limit(50)
            .sort(SortDirection::Desc);
        let query = serde_urlencoded::to_string(&request).unwrap();
        assert!(query.contains("symbols=AAPL%2CMSFT"), "{query}");
        assert!(query.contains("types=cash_dividend%2Cspin_off"), "{query}");
        assert!(query.contains("ids=abc-1%2Cdef-2"), "{query}");
        assert!(query.contains("start=2025-01-01"), "{query}");
        assert!(query.contains("end=2025-12-31"), "{query}");
        assert!(query.contains("limit=50"), "{query}");
        assert!(query.contains("sort=desc"), "{query}");
    }

    #[test]
    #[serial]
    fn empty_filter_slices_omit_their_params() {
        let client = paper_client();
        let request = client.corporate_actions().symbols(&[]).types(&[]).ids(&[]);
        let query = serde_urlencoded::to_string(&request).unwrap();
        assert!(
            !query.contains("symbols") && !query.contains("types") && !query.contains("ids"),
            "expected empty filters to be omitted; got {query}"
        );
    }

    #[test]
    fn corporate_action_type_strings_match_alpaca_vocabulary() {
        assert_eq!(CorporateActionType::ForwardSplit.as_str(), "forward_split");
        assert_eq!(CorporateActionType::SpinOff.as_str(), "spin_off");
        assert_eq!(
            CorporateActionType::StockAndCashMerger.as_str(),
            "stock_and_cash_merger"
        );
        assert_eq!(CorporateActionType::PartialCall.as_str(), "partial_call");
    }

    #[test]
    fn deserializes_response_with_id_field_on_events() {
        let json = r#"{
            "cash_dividends": [
                {"id":"abc-123","symbol":"AAPL","ex_date":"2025-02-10","rate":"0.24"}
            ],
            "spin_offs": [
                {"id":"def-456","source_symbol":"AAPL","new_symbol":"NEWCO","ex_date":"2025-03-15"}
            ]
        }"#;
        let parsed: CorporateActions = serde_json::from_str(json).unwrap();
        assert_eq!(parsed.cash_dividends.len(), 1);
        assert_eq!(parsed.cash_dividends[0]["id"], "abc-123");
        assert_eq!(parsed.spin_offs.len(), 1);
        assert_eq!(parsed.spin_offs[0]["id"], "def-456");
        assert!(parsed.forward_splits.is_empty());
    }
}
