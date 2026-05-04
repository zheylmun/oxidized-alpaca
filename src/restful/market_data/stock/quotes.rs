use crate::{
    Feed,
    restful::{MarketDataClient, null_def_vec},
};
use chrono::{DateTime, Utc};
use reqwest::Method;
use serde::{Deserialize, Serialize};

/// A stock quote (NBBO).
#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct StockQuote {
    /// The timestamp of the quote.
    #[serde(rename = "t")]
    pub timestamp: DateTime<Utc>,
    /// The bid exchange code.
    #[serde(rename = "bx")]
    pub bid_exchange: String,
    /// The bid price.
    #[serde(rename = "bp")]
    pub bid_price: f64,
    /// The bid size.
    #[serde(rename = "bs")]
    pub bid_size: u32,
    /// The ask exchange code.
    #[serde(rename = "ax")]
    pub ask_exchange: String,
    /// The ask price.
    #[serde(rename = "ap")]
    pub ask_price: f64,
    /// The ask size.
    #[serde(rename = "as")]
    pub ask_size: u32,
    /// Quote condition flags.
    #[serde(rename = "c", default)]
    pub conditions: Vec<String>,
    /// The tape.
    #[serde(rename = "z", default)]
    pub tape: Option<String>,
}

#[derive(Debug, Deserialize)]
struct QuotesResponse {
    #[serde(default, deserialize_with = "null_def_vec")]
    quotes: Vec<StockQuote>,
    #[allow(dead_code)]
    symbol: String,
    next_page_token: Option<String>,
}

#[derive(Debug, Deserialize)]
struct LatestQuoteResponse {
    quote: StockQuote,
    #[allow(dead_code)]
    symbol: String,
}

#[derive(Debug, Deserialize)]
struct MultiLatestQuotesResponse {
    quotes: std::collections::HashMap<String, StockQuote>,
}

/// Builder for requesting historical stock quotes.
#[derive(Debug, Serialize)]
#[must_use]
pub struct QuotesRequest<'a> {
    #[serde(skip)]
    client: &'a MarketDataClient,
    #[serde(skip)]
    symbol: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    start: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    end: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    feed: Option<Feed>,
    #[serde(skip_serializing_if = "Option::is_none")]
    limit: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    page_token: Option<String>,
}

impl QuotesRequest<'_> {
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
    /// Set the data feed to use.
    pub fn feed(mut self, feed: Feed) -> Self {
        self.feed = Some(feed);
        self
    }
    /// Cap the total number of quotes returned across all auto-paginated pages.
    pub fn limit(mut self, limit: usize) -> Self {
        self.limit = Some(limit);
        self
    }

    /// Execute the request, auto-paginating until all matching quotes are
    /// retrieved or the configured `limit` is reached.
    pub async fn execute(mut self) -> crate::Result<Vec<StockQuote>> {
        let cap = self.limit;
        let mut all_quotes = Vec::new();
        loop {
            let symbol = &self.symbol;
            let path = format!("v2/stocks/{symbol}/quotes");
            let request = self.client.request(Method::GET, &path)?.query(&self);
            let response: QuotesResponse = self.client.send_and_deserialize(request).await?;
            all_quotes.extend(response.quotes);
            if let Some(cap) = cap
                && all_quotes.len() >= cap
            {
                all_quotes.truncate(cap);
                break;
            }
            match response.next_page_token {
                Some(token) => self.page_token = Some(token),
                None => break,
            }
        }
        Ok(all_quotes)
    }
}

impl MarketDataClient {
    /// Request historical quotes for a single stock symbol.
    pub fn stock_quotes<'a>(&'a self, symbol: &str) -> QuotesRequest<'a> {
        QuotesRequest {
            client: self,
            symbol: symbol.to_string(),
            start: None,
            end: None,
            feed: None,
            limit: None,
            page_token: None,
        }
    }

    /// Get the latest quote for a single stock symbol.
    pub async fn stock_latest_quote(&self, symbol: &str) -> crate::Result<StockQuote> {
        let path = format!("v2/stocks/{symbol}/quotes/latest");
        let request = self.request(Method::GET, &path)?;
        let response: LatestQuoteResponse = self.send_and_deserialize(request).await?;
        Ok(response.quote)
    }

    /// Get the latest quotes for multiple stock symbols.
    pub async fn stock_latest_quotes(
        &self,
        symbols: &[&str],
    ) -> crate::Result<std::collections::HashMap<String, StockQuote>> {
        let request = self
            .request(Method::GET, "v2/stocks/quotes/latest")?
            .query(&[("symbols", symbols.join(","))]);
        let response: MultiLatestQuotesResponse = self.send_and_deserialize(request).await?;
        Ok(response.quotes)
    }
}
