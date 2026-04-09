use crate::restful::TradingClient;
use chrono::{DateTime, NaiveDate, Utc};
use reqwest::Method;
use serde::{Deserialize, Serialize};

/// Type of account activity.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
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
    /// Option assignment
    OPASN,
    /// Option exercise
    OPEXP,
    /// Option expiration
    OPXRC,
    /// Splits
    SPLIT,
}

/// An account activity event.
#[derive(Clone, Debug, Deserialize)]
pub struct Activity {
    pub id: String,
    pub activity_type: ActivityType,
    #[serde(default)]
    pub symbol: Option<String>,
    #[serde(default)]
    pub date: Option<NaiveDate>,
    #[serde(default)]
    pub net_amount: Option<String>,
    #[serde(default)]
    pub qty: Option<String>,
    #[serde(default)]
    pub per_share_amount: Option<String>,
    #[serde(default)]
    pub price: Option<String>,
    #[serde(default)]
    pub cum_qty: Option<String>,
    #[serde(default)]
    pub leaves_qty: Option<String>,
    #[serde(default)]
    pub side: Option<String>,
    #[serde(default)]
    pub order_id: Option<String>,
    #[serde(default)]
    pub transaction_time: Option<DateTime<Utc>>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub status: Option<String>,
}

/// Builder for listing account activities.
#[derive(Debug, Serialize)]
#[must_use]
pub struct ListActivitiesRequest<'a> {
    #[serde(skip)]
    client: &'a TradingClient,
    #[serde(skip)]
    activity_type: Option<ActivityType>,
    #[serde(skip_serializing_if = "Option::is_none")]
    date: Option<NaiveDate>,
    #[serde(skip_serializing_if = "Option::is_none")]
    until: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    after: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    direction: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    page_size: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    page_token: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    category: Option<String>,
}

impl ListActivitiesRequest<'_> {
    pub fn activity_type(mut self, activity_type: ActivityType) -> Self {
        self.activity_type = Some(activity_type);
        self
    }
    pub fn date(mut self, date: NaiveDate) -> Self {
        self.date = Some(date);
        self
    }
    pub fn until(mut self, until: DateTime<Utc>) -> Self {
        self.until = Some(until);
        self
    }
    pub fn after(mut self, after: DateTime<Utc>) -> Self {
        self.after = Some(after);
        self
    }
    pub fn direction(mut self, direction: &str) -> Self {
        self.direction = Some(direction.to_string());
        self
    }
    pub fn page_size(mut self, size: u32) -> Self {
        self.page_size = Some(size);
        self
    }
    pub fn page_token(mut self, token: impl Into<String>) -> Self {
        self.page_token = Some(token.into());
        self
    }
    pub fn category(mut self, category: &str) -> Self {
        self.category = Some(category.to_string());
        self
    }

    /// Execute the request.
    pub async fn execute(self) -> crate::Result<Vec<Activity>> {
        let path = match &self.activity_type {
            Some(at) => format!("account/activities/{at:?}"),
            None => "account/activities".to_string(),
        };
        let request = self.client.request(Method::GET, &path).query(&self);
        self.client.send_and_deserialize(request).await
    }
}

impl TradingClient {
    /// List account activities with optional filters.
    ///
    /// ```ignore
    /// let activities = client.list_activities()
    ///     .activity_type(ActivityType::FILL)
    ///     .page_size(50)
    ///     .execute().await?;
    /// ```
    pub fn list_activities(&self) -> ListActivitiesRequest<'_> {
        ListActivitiesRequest {
            client: self,
            activity_type: None,
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
