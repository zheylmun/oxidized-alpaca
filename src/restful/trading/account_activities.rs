use crate::restful::{SortDirection, TradingClient};
use chrono::{DateTime, NaiveDate, Utc};
use reqwest::Method;
use serde::{Deserialize, Serialize};

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
    /// Order fills
    FILL,
    /// Transactions (cash)
    TRANS,
    /// Miscellaneous
    MISC,
    /// ACATS in
    ACATC,
    /// ACATS out
    ACATS,
    /// Cash deposits
    CSD,
    /// Cash withdrawals
    CSW,
    /// Dividends
    DIV,
    /// Journal entries (cash)
    JNLC,
    /// Journal entries (stock)
    JNLS,
    /// Interest
    INT,
    /// Fees
    FEE,
    /// Option assignment
    OPASN,
    /// Option exercise
    OPEXP,
    /// Option expiration
    OPXRC,
    /// Splits
    SPLIT,
    /// Any activity code not modeled above; the raw string from the API.
    Other(String),
}

impl ActivityType {
    fn as_str(&self) -> &str {
        match self {
            Self::FILL => "FILL",
            Self::TRANS => "TRANS",
            Self::MISC => "MISC",
            Self::ACATC => "ACATC",
            Self::ACATS => "ACATS",
            Self::CSD => "CSD",
            Self::CSW => "CSW",
            Self::DIV => "DIV",
            Self::JNLC => "JNLC",
            Self::JNLS => "JNLS",
            Self::INT => "INT",
            Self::FEE => "FEE",
            Self::OPASN => "OPASN",
            Self::OPEXP => "OPEXP",
            Self::OPXRC => "OPXRC",
            Self::SPLIT => "SPLIT",
            Self::Other(raw) => raw,
        }
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
            "FILL" => Self::FILL,
            "TRANS" => Self::TRANS,
            "MISC" => Self::MISC,
            "ACATC" => Self::ACATC,
            "ACATS" => Self::ACATS,
            "CSD" => Self::CSD,
            "CSW" => Self::CSW,
            "DIV" => Self::DIV,
            "JNLC" => Self::JNLC,
            "JNLS" => Self::JNLS,
            "INT" => Self::INT,
            "FEE" => Self::FEE,
            "OPASN" => Self::OPASN,
            "OPEXP" => Self::OPEXP,
            "OPXRC" => Self::OPXRC,
            "SPLIT" => Self::SPLIT,
            _ => Self::Other(raw),
        })
    }
}

/// An account activity event.
#[derive(Clone, Debug, Deserialize)]
pub struct Activity {
    /// Activity ID.
    pub id: String,
    /// Type of activity.
    pub activity_type: ActivityType,
    /// Ticker symbol (for trade activities).
    #[serde(default)]
    pub symbol: Option<String>,
    /// Date of the activity.
    #[serde(default)]
    pub date: Option<NaiveDate>,
    /// Net dollar amount of the activity.
    #[serde(default)]
    pub net_amount: Option<String>,
    /// Quantity of shares.
    #[serde(default)]
    pub qty: Option<String>,
    /// Per-share amount (e.g., dividend per share).
    #[serde(default)]
    pub per_share_amount: Option<String>,
    /// Price per share.
    #[serde(default)]
    pub price: Option<String>,
    /// Cumulative quantity filled.
    #[serde(default)]
    pub cum_qty: Option<String>,
    /// Remaining quantity to fill.
    #[serde(default)]
    pub leaves_qty: Option<String>,
    /// Buy or sell side.
    #[serde(default)]
    pub side: Option<String>,
    /// Associated order ID.
    #[serde(default)]
    pub order_id: Option<String>,
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
            Some(at) => format!("account/activities/{}", at.as_str()),
            None => "account/activities".to_string(),
        };
        self.page_size = Some(ACTIVITIES_PAGE_SIZE);
        let mut all: Vec<Activity> = Vec::new();
        loop {
            let request = self.client.request(Method::GET, &path).query(&self);
            let page: Vec<Activity> = self.client.send_and_deserialize(request).await?;
            let received = page.len();
            let last_id = page.last().map(|a| a.id.clone());
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
    ///     .activity_type(ActivityType::FILL)
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
