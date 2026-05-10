use crate::restful::{SortDirection, TradingClient, string_as_optional_decimal};
use crate::{ActivityId, OrderId};
use chrono::{DateTime, NaiveDate, Utc};
use reqwest::Method;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use super::orders::Side;

/// Category filter accepted by the account-activities endpoint.
#[derive(Clone, Copy, Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum ActivityCategory {
    /// Trade-related activities only.
    Trade,
    /// Non-trade activities only.
    NonTrade,
}

/// Type of account activity.
///
/// Alpaca exposes a long and growing list of activity codes; the variants
/// below are the ones with stable, well-known semantics. Anything else is
/// preserved verbatim under [`ActivityType::Other`].
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum ActivityType {
    /// Order fills (`FILL`).
    Fill,
    /// Transactions, cash (`TRANS`).
    Trans,
    /// Miscellaneous (`MISC`).
    Misc,
    /// ACATS in (`ACATC`).
    AcatsIn,
    /// ACATS out (`ACATS`).
    AcatsOut,
    /// Cash deposits (`CSD`).
    CashDeposit,
    /// Cash withdrawals (`CSW`).
    CashWithdrawal,
    /// Dividends (`DIV`).
    Dividend,
    /// Journal entries, cash (`JNLC`).
    JournalCash,
    /// Journal entries, stock (`JNLS`).
    JournalStock,
    /// Interest (`INT`).
    Interest,
    /// Fees (`FEE`).
    Fee,
    /// Option assignment (`OPASN`).
    OptionAssignment,
    /// Option corporate action (`OPCA`).
    OptionCorporateAction,
    /// Option exercise (`OPEXP`).
    OptionExercise,
    /// Option expiration (`OPXRC`).
    OptionExpiration,
    /// Splits (`SPLIT`).
    Split,
    /// Any activity code not modeled above; the raw string from the API.
    Other(String),
}

impl ActivityType {
    fn as_str(&self) -> &str {
        match self {
            Self::Fill => "FILL",
            Self::Trans => "TRANS",
            Self::Misc => "MISC",
            Self::AcatsIn => "ACATC",
            Self::AcatsOut => "ACATS",
            Self::CashDeposit => "CSD",
            Self::CashWithdrawal => "CSW",
            Self::Dividend => "DIV",
            Self::JournalCash => "JNLC",
            Self::JournalStock => "JNLS",
            Self::Interest => "INT",
            Self::Fee => "FEE",
            Self::OptionAssignment => "OPASN",
            Self::OptionCorporateAction => "OPCA",
            Self::OptionExercise => "OPEXP",
            Self::OptionExpiration => "OPXRC",
            Self::Split => "SPLIT",
            Self::Other(raw) => raw,
        }
    }
}

impl std::fmt::Display for ActivityType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl Serialize for ActivityType {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for ActivityType {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let raw = String::deserialize(deserializer)?;
        Ok(match raw.as_str() {
            "FILL" => Self::Fill,
            "TRANS" => Self::Trans,
            "MISC" => Self::Misc,
            "ACATC" => Self::AcatsIn,
            "ACATS" => Self::AcatsOut,
            "CSD" => Self::CashDeposit,
            "CSW" => Self::CashWithdrawal,
            "DIV" => Self::Dividend,
            "JNLC" => Self::JournalCash,
            "JNLS" => Self::JournalStock,
            "INT" => Self::Interest,
            "FEE" => Self::Fee,
            "OPASN" => Self::OptionAssignment,
            "OPCA" => Self::OptionCorporateAction,
            "OPEXP" => Self::OptionExercise,
            "OPXRC" => Self::OptionExpiration,
            "SPLIT" => Self::Split,
            _ => Self::Other(raw),
        })
    }
}

/// An account activity event.
#[derive(Clone, Debug, Deserialize)]
#[non_exhaustive]
pub struct Activity {
    /// Activity ID.
    pub id: ActivityId,
    /// Type of activity.
    pub activity_type: ActivityType,
    /// Sub-classification of the activity, when the API provides one.
    /// Free-form descriptor (e.g. `"FILL"`, `"PARTIAL_FILL"` on trade
    /// activities, or corporate-action sub-types on `OPCA`); preserved
    /// verbatim because Alpaca expands the vocabulary over time.
    #[serde(default)]
    pub activity_sub_type: Option<String>,
    /// Ticker symbol (for trade activities).
    #[serde(default)]
    pub symbol: Option<String>,
    /// Date of the activity.
    #[serde(default)]
    pub date: Option<NaiveDate>,
    /// Net dollar amount of the activity.
    #[serde(default, deserialize_with = "string_as_optional_decimal")]
    pub net_amount: Option<Decimal>,
    /// Quantity of shares.
    #[serde(default, deserialize_with = "string_as_optional_decimal")]
    pub qty: Option<Decimal>,
    /// Per-share amount (e.g., dividend per share).
    #[serde(default, deserialize_with = "string_as_optional_decimal")]
    pub per_share_amount: Option<Decimal>,
    /// Price per share.
    #[serde(default, deserialize_with = "string_as_optional_decimal")]
    pub price: Option<Decimal>,
    /// Cumulative quantity filled.
    #[serde(default, deserialize_with = "string_as_optional_decimal")]
    pub cum_qty: Option<Decimal>,
    /// Remaining quantity to fill.
    #[serde(default, deserialize_with = "string_as_optional_decimal")]
    pub leaves_qty: Option<Decimal>,
    /// Buy or sell side.
    #[serde(default)]
    pub side: Option<Side>,
    /// Associated order ID.
    #[serde(default)]
    pub order_id: Option<OrderId>,
    /// Timestamp of the transaction.
    #[serde(default)]
    pub transaction_time: Option<DateTime<Utc>>,
    /// Description of the activity.
    #[serde(default)]
    pub description: Option<String>,
    /// Status of the activity.
    #[serde(default)]
    pub status: Option<String>,
}

/// Default per-page batch size used internally when auto-paginating
/// account activities. The trading API caps `page_size` at 100.
const ACTIVITIES_PAGE_SIZE: u32 = 100;

/// Builder for listing account activities.
#[derive(Debug, Serialize)]
#[must_use]
pub struct ListActivitiesRequest<'a> {
    #[serde(skip)]
    client: &'a TradingClient,
    #[serde(skip)]
    activity_type: Option<ActivityType>,
    #[serde(skip)]
    limit: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    date: Option<NaiveDate>,
    #[serde(skip_serializing_if = "Option::is_none")]
    until: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    after: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    direction: Option<SortDirection>,
    #[serde(skip_serializing_if = "Option::is_none")]
    page_size: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    page_token: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    category: Option<ActivityCategory>,
}

impl ListActivitiesRequest<'_> {
    /// Filter by activity type.
    pub fn activity_type(mut self, activity_type: ActivityType) -> Self {
        self.activity_type = Some(activity_type);
        self
    }
    /// Filter by exact date.
    pub fn date(mut self, date: NaiveDate) -> Self {
        self.date = Some(date);
        self
    }
    /// Only return activities before this timestamp.
    pub fn until(mut self, until: DateTime<Utc>) -> Self {
        self.until = Some(until);
        self
    }
    /// Only return activities after this timestamp.
    pub fn after(mut self, after: DateTime<Utc>) -> Self {
        self.after = Some(after);
        self
    }
    /// Sort direction (ascending or descending).
    pub fn direction(mut self, direction: SortDirection) -> Self {
        self.direction = Some(direction);
        self
    }
    /// Cap the total number of activities returned across all
    /// auto-paginated pages.
    pub fn limit(mut self, limit: usize) -> Self {
        self.limit = Some(limit);
        self
    }
    /// Filter by activity category (trade or non-trade).
    pub fn category(mut self, category: ActivityCategory) -> Self {
        self.category = Some(category);
        self
    }

    /// Execute the request, auto-paginating until all matching activities are
    /// retrieved or the configured `limit` is reached.
    pub async fn execute(mut self) -> crate::Result<Vec<Activity>> {
        let cap = self.limit;
        let path = match &self.activity_type {
            Some(at) => format!("v2/account/activities/{at}"),
            None => "v2/account/activities".to_string(),
        };
        self.page_size = Some(ACTIVITIES_PAGE_SIZE);
        let mut all: Vec<Activity> = Vec::new();
        loop {
            let request = self.client.request(Method::GET, &path)?.query(&self);
            let page: Vec<Activity> = self.client.send_and_deserialize(request).await?;
            let received = page.len();
            let last_id = page.last().map(|a| a.id.as_str().to_string());
            all.extend(page);
            if let Some(cap) = cap
                && all.len() >= cap
            {
                all.truncate(cap);
                break;
            }
            if received < ACTIVITIES_PAGE_SIZE as usize {
                break;
            }
            match last_id {
                Some(id) => self.page_token = Some(id),
                None => break,
            }
        }
        Ok(all)
    }
}

impl TradingClient {
    /// List account activities with optional filters. The result is fully
    /// auto-paginated; the trading API's `page_size` parameter is managed
    /// internally.
    ///
    /// ```ignore
    /// let activities = client.list_activities()
    ///     .activity_type(ActivityType::Fill)
    ///     .limit(500)
    ///     .execute().await?;
    /// ```
    pub fn list_activities(&self) -> ListActivitiesRequest<'_> {
        ListActivitiesRequest {
            client: self,
            activity_type: None,
            limit: None,
            date: None,
            until: None,
            after: None,
            direction: None,
            page_size: None,
            page_token: None,
            category: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn opca_round_trips() {
        let json = "\"OPCA\"";
        let parsed: ActivityType = serde_json::from_str(json).unwrap();
        assert_eq!(parsed, ActivityType::OptionCorporateAction);
        assert_eq!(parsed.to_string(), "OPCA");
        assert_eq!(serde_json::to_string(&parsed).unwrap(), json);
    }

    #[test]
    fn every_known_code_round_trips() {
        // Pins each variant's wire string so a typo in the manual ser/de
        // impl surfaces immediately.
        let cases = [
            (ActivityType::Fill, "\"FILL\""),
            (ActivityType::Trans, "\"TRANS\""),
            (ActivityType::Misc, "\"MISC\""),
            (ActivityType::AcatsIn, "\"ACATC\""),
            (ActivityType::AcatsOut, "\"ACATS\""),
            (ActivityType::CashDeposit, "\"CSD\""),
            (ActivityType::CashWithdrawal, "\"CSW\""),
            (ActivityType::Dividend, "\"DIV\""),
            (ActivityType::JournalCash, "\"JNLC\""),
            (ActivityType::JournalStock, "\"JNLS\""),
            (ActivityType::Interest, "\"INT\""),
            (ActivityType::Fee, "\"FEE\""),
            (ActivityType::OptionAssignment, "\"OPASN\""),
            (ActivityType::OptionCorporateAction, "\"OPCA\""),
            (ActivityType::OptionExercise, "\"OPEXP\""),
            (ActivityType::OptionExpiration, "\"OPXRC\""),
            (ActivityType::Split, "\"SPLIT\""),
        ];
        for (variant, expected) in cases {
            assert_eq!(serde_json::to_string(&variant).unwrap(), expected);
            let parsed: ActivityType = serde_json::from_str(expected).unwrap();
            assert_eq!(parsed, variant);
        }
    }

    #[test]
    fn unknown_activity_type_falls_back_to_other() {
        let parsed: ActivityType = serde_json::from_str("\"NEWCODE\"").unwrap();
        assert_eq!(parsed, ActivityType::Other("NEWCODE".to_string()));
        assert_eq!(serde_json::to_string(&parsed).unwrap(), "\"NEWCODE\"");
    }

    #[test]
    fn opca_activity_with_sub_type_deserializes() {
        let json = r#"{
            "id": "20250507000000000::abc",
            "activity_type": "OPCA",
            "activity_sub_type": "SPINOFF",
            "symbol": "AAPL",
            "date": "2025-05-07",
            "net_amount": "0.00",
            "description": "Option corporate action: spin-off"
        }"#;
        let activity: Activity = serde_json::from_str(json).unwrap();
        assert_eq!(activity.activity_type, ActivityType::OptionCorporateAction);
        assert_eq!(activity.activity_sub_type.as_deref(), Some("SPINOFF"));
        assert_eq!(activity.symbol.as_deref(), Some("AAPL"));
    }

    #[test]
    fn fill_activity_without_sub_type_deserializes() {
        let json = r#"{
            "id": "20250101000000000::xyz",
            "activity_type": "FILL",
            "symbol": "MSFT",
            "qty": "10",
            "price": "412.5",
            "side": "buy"
        }"#;
        let activity: Activity = serde_json::from_str(json).unwrap();
        assert_eq!(activity.activity_type, ActivityType::Fill);
        assert!(activity.activity_sub_type.is_none());
        assert_eq!(activity.qty, Some(Decimal::from(10)));
    }

    #[test]
    fn single_activity_lookup_response_deserializes() {
        let json = r#"{
            "id": "20250507000000000::abc",
            "activity_type": "DIV",
            "symbol": "AAPL",
            "date": "2025-05-07",
            "net_amount": "12.34",
            "per_share_amount": "0.24",
            "description": "Cash dividend"
        }"#;
        let activity: Activity = serde_json::from_str(json).unwrap();
        assert_eq!(activity.id.as_str(), "20250507000000000::abc");
        assert_eq!(activity.activity_type, ActivityType::Dividend);
        assert_eq!(activity.net_amount, Some(Decimal::new(1234, 2)));
        assert_eq!(activity.per_share_amount, Some(Decimal::new(24, 2)));
    }
}
