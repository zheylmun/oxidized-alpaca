use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::streaming::wire::StreamError;

/// Subscriptions for the news streaming feed.
///
/// Use the symbol `"*"` to receive news for every covered ticker.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NewsSubscriptionList {
    /// Symbols subscribed to news (use `"*"` for all).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub news: Option<Vec<String>>,
}

impl NewsSubscriptionList {
    /// Create an empty subscription list.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Subscribe to news for `symbol` (`"*"` to subscribe to everything).
    #[must_use]
    pub fn add_news(self, symbol: &str) -> Self {
        let mut list = self.news.unwrap_or_default();
        if !list.iter().any(|s| s == symbol) {
            list.push(symbol.to_string());
        }
        Self { news: Some(list) }
    }
}

/// A single news article delivered on the news stream.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct NewsArticle {
    /// Unique news article ID.
    pub id: i64,
    /// Article headline.
    pub headline: String,
    /// Brief excerpt (often the first sentence).
    pub summary: String,
    /// Full article body. May contain HTML.
    pub content: String,
    /// Original article author.
    pub author: String,
    /// Tickers referenced by the article.
    pub symbols: Vec<String>,
    /// News origin (e.g. `"benzinga"`).
    pub source: String,
    /// Article URL.
    pub url: String,
    /// When the article was published.
    pub created_at: DateTime<Utc>,
    /// When the article was last modified.
    pub updated_at: DateTime<Utc>,
}

/// Messages received from the news streaming feed.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "T")]
pub enum NewsStreamMessage {
    /// Internally consumed stream acknowledging successful completion of requests.
    #[serde(rename = "success")]
    Control {
        /// The control message payload.
        msg: crate::streaming::wire::ControlMessage,
    },
    /// Error message from the server.
    #[serde(rename = "error")]
    Error(StreamError),
    /// Subscription confirmation with the current subscription list.
    #[serde(rename = "subscription")]
    Subscription(NewsSubscriptionList),
    /// A news article.
    #[serde(rename = "n")]
    News(NewsArticle),
}

impl NewsStreamMessage {
    pub(crate) const fn control(&self) -> Option<&crate::streaming::wire::ControlMessage> {
        match self {
            NewsStreamMessage::Control { msg } => Some(msg),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserializes_news_article() {
        let json = r#"{
            "T":"n",
            "id":24818772,
            "headline":"Apple Hits New High",
            "summary":"Shares rise after earnings.",
            "content":"<p>Apple ...</p>",
            "author":"Jane Doe",
            "symbols":["AAPL","MSFT"],
            "source":"benzinga",
            "url":"https://example.com/news/24818772",
            "created_at":"2024-01-02T15:30:00Z",
            "updated_at":"2024-01-02T15:31:00Z"
        }"#;
        match serde_json::from_str(json).unwrap() {
            NewsStreamMessage::News(article) => {
                assert_eq!(article.id, 24818772);
                assert_eq!(
                    article.symbols,
                    vec!["AAPL".to_string(), "MSFT".to_string()]
                );
                assert_eq!(article.source, "benzinga");
            }
            other => panic!("expected News, got {other:?}"),
        }
    }

    #[test]
    fn subscription_list_serializes_only_news() {
        let list = NewsSubscriptionList::new().add_news("*");
        let json = serde_json::to_string(&list).unwrap();
        assert_eq!(json, r#"{"news":["*"]}"#);
    }

    #[test]
    fn add_news_dedupes() {
        let list = NewsSubscriptionList::new()
            .add_news("AAPL")
            .add_news("AAPL")
            .add_news("MSFT");
        assert_eq!(
            list.news.unwrap(),
            vec!["AAPL".to_string(), "MSFT".to_string()]
        );
    }
}
