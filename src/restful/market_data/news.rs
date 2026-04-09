use crate::restful::MarketDataClient;
use chrono::{DateTime, Utc};
use reqwest::Method;
use serde::{Deserialize, Serialize};

/// A news article.
#[derive(Clone, Debug, Deserialize)]
pub struct NewsArticle {
    pub id: i64,
    pub headline: String,
    #[serde(default)]
    pub summary: Option<String>,
    pub author: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    #[serde(default)]
    pub url: Option<String>,
    #[serde(default)]
    pub content: Option<String>,
    #[serde(default)]
    pub symbols: Vec<String>,
    #[serde(default)]
    pub source: Option<String>,
    #[serde(default)]
    pub images: Vec<NewsImage>,
}

/// An image associated with a news article.
#[derive(Clone, Debug, Deserialize)]
pub struct NewsImage {
    #[serde(default)]
    pub size: Option<String>,
    pub url: String,
}

#[derive(Debug, Deserialize)]
struct NewsResponse {
    news: Vec<NewsArticle>,
    #[allow(dead_code)]
    next_page_token: Option<String>,
}

/// Builder for requesting news articles.
#[derive(Debug, Serialize)]
#[must_use]
pub struct NewsRequest<'a> {
    #[serde(skip)]
    client: &'a MarketDataClient,
    #[serde(skip_serializing_if = "Option::is_none")]
    symbols: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    start: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    end: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    limit: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    sort: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    include_content: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    exclude_contentless: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    page_token: Option<String>,
}

impl NewsRequest<'_> {
    pub fn symbols(mut self, symbols: &str) -> Self {
        self.symbols = Some(symbols.to_string());
        self
    }
    pub fn start(mut self, start: DateTime<Utc>) -> Self {
        self.start = Some(start);
        self
    }
    pub fn end(mut self, end: DateTime<Utc>) -> Self {
        self.end = Some(end);
        self
    }
    pub fn limit(mut self, limit: u32) -> Self {
        self.limit = Some(limit);
        self
    }
    pub fn sort(mut self, sort: &str) -> Self {
        self.sort = Some(sort.to_string());
        self
    }
    pub fn include_content(mut self, include: bool) -> Self {
        self.include_content = Some(include);
        self
    }
    pub fn exclude_contentless(mut self, exclude: bool) -> Self {
        self.exclude_contentless = Some(exclude);
        self
    }

    pub async fn execute(self) -> crate::Result<Vec<NewsArticle>> {
        let request = self
            .client
            .request(Method::GET, "v1beta1/news")
            .query(&self);
        let response: NewsResponse = self.client.send_and_deserialize(request).await?;
        Ok(response.news)
    }
}

impl MarketDataClient {
    /// Request news articles.
    ///
    /// ```ignore
    /// let news = client.news()
    ///     .symbols("AAPL,GOOG")
    ///     .limit(10)
    ///     .execute().await?;
    /// ```
    pub fn news(&self) -> NewsRequest<'_> {
        NewsRequest {
            client: self,
            symbols: None,
            start: None,
            end: None,
            limit: None,
            sort: None,
            include_content: None,
            exclude_contentless: None,
            page_token: None,
        }
    }
}
