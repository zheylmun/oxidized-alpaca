use crate::restful::{MarketDataClient, SortDirection, null_def_vec};
use chrono::{DateTime, Utc};
use reqwest::Method;
use serde::{Deserialize, Serialize};

/// A news article.
#[derive(Clone, Debug, Deserialize)]
#[non_exhaustive]
pub struct NewsArticle {
    /// The article ID.
    pub id: i64,
    /// The headline.
    pub headline: String,
    /// A brief summary.
    #[serde(default)]
    pub summary: Option<String>,
    /// The author name.
    pub author: String,
    /// When the article was created.
    pub created_at: DateTime<Utc>,
    /// When the article was last updated.
    pub updated_at: DateTime<Utc>,
    /// The article URL.
    #[serde(default)]
    pub url: Option<String>,
    /// The full article content.
    #[serde(default)]
    pub content: Option<String>,
    /// Related stock symbols.
    #[serde(default)]
    pub symbols: Vec<String>,
    /// The news source.
    #[serde(default)]
    pub source: Option<String>,
    /// Associated images.
    #[serde(default)]
    pub images: Vec<NewsImage>,
}

/// An image associated with a news article.
#[derive(Clone, Debug, Deserialize)]
#[non_exhaustive]
pub struct NewsImage {
    /// The image size descriptor (e.g. "thumb", "small", "large").
    #[serde(default)]
    pub size: Option<String>,
    /// The image URL.
    pub url: String,
}

#[derive(Debug, Deserialize)]
struct NewsResponse {
    #[serde(default, deserialize_with = "null_def_vec")]
    news: Vec<NewsArticle>,
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
    limit: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    sort: Option<SortDirection>,
    #[serde(skip_serializing_if = "Option::is_none")]
    include_content: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    exclude_contentless: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    page_token: Option<String>,
}

impl NewsRequest<'_> {
    /// Filter by stock symbols.
    pub fn symbols(mut self, symbols: &[&str]) -> Self {
        self.symbols = Some(symbols.join(","));
        self
    }
    /// Set the start time filter.
    pub fn start(mut self, start: DateTime<Utc>) -> Self {
        self.start = Some(start);
        self
    }
    /// Set the end time filter.
    pub fn end(mut self, end: DateTime<Utc>) -> Self {
        self.end = Some(end);
        self
    }
    /// Cap the total number of articles returned across all auto-paginated pages.
    pub fn limit(mut self, limit: usize) -> Self {
        self.limit = Some(limit);
        self
    }
    /// Set the sort order (ascending or descending).
    pub fn sort(mut self, sort: SortDirection) -> Self {
        self.sort = Some(sort);
        self
    }
    /// Whether to include the full article content.
    pub fn include_content(mut self, include: bool) -> Self {
        self.include_content = Some(include);
        self
    }
    /// Whether to exclude articles without content.
    pub fn exclude_contentless(mut self, exclude: bool) -> Self {
        self.exclude_contentless = Some(exclude);
        self
    }

    /// Execute the request, auto-paginating until all matching articles are
    /// retrieved or the configured `limit` is reached.
    pub async fn execute(mut self) -> crate::Result<Vec<NewsArticle>> {
        let cap = self.limit;
        let mut all = Vec::new();
        loop {
            let request = self
                .client
                .request(Method::GET, "v1beta1/news")?
                .query(&self);
            let response: NewsResponse = self.client.send_and_deserialize(request).await?;
            all.extend(response.news);
            if let Some(cap) = cap
                && all.len() >= cap
            {
                all.truncate(cap);
                break;
            }
            match response.next_page_token {
                Some(token) => self.page_token = Some(token),
                None => break,
            }
        }
        Ok(all)
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
