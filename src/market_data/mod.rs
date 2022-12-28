use serde::{Deserialize, Serialize};
pub mod stock_pricing;

#[derive(Deserialize, Serialize)]
pub struct SubscriptionList {
    /// List of symbols for bars subscriptions
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bars: Option<Vec<String>>,
    /// List of symbols for quotes subscriptions
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quotes: Option<Vec<String>>,
    /// List of symbols for trades subscriptions
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trades: Option<Vec<String>>,
    /// List of symbols for news subscriptions
    #[serde(skip_serializing_if = "Option::is_none")]
    pub news: Option<Vec<String>>,
}

/// Streaming Authentication Message
#[derive(Serialize)]
#[serde(tag = "action")]
pub enum Request {
    #[serde(rename = "auth")]
    AuthMessage { key: String, secret: String },
    #[serde(rename = "subscribe")]
    Subscribe(SubscriptionList),
    #[serde(rename = "unsubscribe")]
    Unsubscribe(SubscriptionList),
}
