use crate::{
    RestFeed,
    restful::{MarketDataClient, SortDirection, null_def_vec},
};
use chrono::{DateTime, Utc};
use reqwest::Method;
use serde::{Deserialize, Serialize};

use super::{AsOf, pagination};

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
    feed: Option<RestFeed>,
    #[serde(skip_serializing_if = "Option::is_none")]
    limit: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    asof: Option<AsOf>,
    #[serde(skip_serializing_if = "Option::is_none")]
    currency: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    sort: Option<SortDirection>,
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
    pub fn feed(mut self, feed: RestFeed) -> Self {
        self.feed = Some(feed);
        self
    }
    /// Cap the total number of quotes returned across all auto-paginated pages.
    pub fn limit(mut self, limit: usize) -> Self {
        self.limit = Some(limit);
        self
    }
    /// Set the `asof` value, used to anchor symbol mapping for renamed
    /// instruments. Pass [`AsOf::SkipSymbolMapping`] to disable mapping.
    pub fn asof(mut self, asof: AsOf) -> Self {
        self.asof = Some(asof);
        self
    }
    /// Set the response `currency` (ISO 4217). Defaults to USD when unset.
    pub fn currency(mut self, currency: impl Into<String>) -> Self {
        self.currency = Some(currency.into());
        self
    }
    /// Set the result `sort` order. Defaults to ascending when unset.
    pub fn sort(mut self, sort: SortDirection) -> Self {
        self.sort = Some(sort);
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
            asof: None,
            currency: None,
            sort: None,
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

    /// Request historical quotes for multiple stock symbols. Returns a
    /// map keyed by symbol; symbols with no quotes in the queried range
    /// are omitted from the response.
    pub fn stock_quotes_multi<'a>(&'a self, symbols: &[&str]) -> MultiSymbolQuotesRequest<'a> {
        MultiSymbolQuotesRequest {
            client: self,
            symbols: symbols.join(","),
            start: None,
            end: None,
            feed: None,
            limit: None,
            asof: None,
            currency: None,
            sort: None,
            page_token: None,
        }
    }
}

/// A request for `/v2/stocks/quotes` (multi-symbol historical quotes).
#[derive(Debug, Serialize)]
#[must_use]
pub struct MultiSymbolQuotesRequest<'a> {
    #[serde(skip)]
    client: &'a MarketDataClient,
    symbols: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    start: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    end: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    feed: Option<RestFeed>,
    /// Per-symbol cap applied client-side after pagination (see
    /// [`MultiSymbolBarsRequest`][super::bars::MultiSymbolBarsRequest]
    /// for rationale).
    #[serde(skip)]
    limit: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    asof: Option<AsOf>,
    #[serde(skip_serializing_if = "Option::is_none")]
    currency: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    sort: Option<SortDirection>,
    #[serde(skip_serializing_if = "Option::is_none")]
    page_token: Option<String>,
}

impl MultiSymbolQuotesRequest<'_> {
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
    pub fn feed(mut self, feed: RestFeed) -> Self {
        self.feed = Some(feed);
        self
    }
    /// Cap the total number of quotes returned per symbol across all
    /// auto-paginated pages.
    pub fn limit(mut self, limit: usize) -> Self {
        self.limit = Some(limit);
        self
    }
    /// Set the `asof` value, used to anchor symbol mapping for renamed
    /// instruments. Pass [`AsOf::SkipSymbolMapping`] to disable mapping.
    pub fn asof(mut self, asof: AsOf) -> Self {
        self.asof = Some(asof);
        self
    }
    /// Set the response `currency` (ISO 4217). Defaults to USD when unset.
    pub fn currency(mut self, currency: impl Into<String>) -> Self {
        self.currency = Some(currency.into());
        self
    }
    /// Set the result `sort` order. Defaults to ascending when unset.
    pub fn sort(mut self, sort: SortDirection) -> Self {
        self.sort = Some(sort);
        self
    }

    /// Execute the request, auto-paginating until all matching quotes are
    /// retrieved. When `limit` is set, each symbol's series is truncated
    /// to the cap as pages arrive, and pagination stops as soon as every
    /// requested symbol has reached the cap (or the API runs out of pages).
    pub async fn execute(
        mut self,
    ) -> crate::Result<std::collections::HashMap<String, Vec<StockQuote>>> {
        let cap = self.limit;
        if cap == Some(0) {
            return Ok(std::collections::HashMap::new());
        }
        let requested: Vec<String> = self.symbols.split(',').map(str::to_string).collect();
        let mut combined: std::collections::HashMap<String, Vec<StockQuote>> =
            std::collections::HashMap::new();
        loop {
            let request = self
                .client
                .request(Method::GET, "v2/stocks/quotes")?
                .query(&self);
            let response: MultiQuotesResponse = self.client.send_and_deserialize(request).await?;
            pagination::extend_capped(&mut combined, response.quotes, cap);
            if let Some(cap) = cap
                && pagination::all_symbols_filled(&combined, &requested, cap)
            {
                break;
            }
            match response.next_page_token {
                Some(token) => self.page_token = Some(token),
                None => break,
            }
        }
        Ok(combined)
    }
}

#[derive(Debug, Deserialize)]
struct MultiQuotesResponse {
    #[serde(default)]
    quotes: std::collections::HashMap<String, Vec<StockQuote>>,
    next_page_token: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::MultiQuotesResponse;

    #[test]
    fn deserializes_multi_symbol_quotes_response_with_pagination() {
        let json = r#"{
            "next_page_token": "QUFQTHwxNzc4MTYwNjAwMDM0OTM4NDAzfFF8Mjg4Ljk2fDEyMHxRfDI4OS4yfDgwfFI=",
            "quotes": {
                "AAPL": [
                    {"ap":289.29,"as":120,"ax":"Q","bp":288.96,"bs":120,"bx":"Q","c":["R"],"t":"2026-05-07T13:30:00.015531758Z","z":"C"},
                    {"ap":289.2,"as":40,"ax":"Q","bp":288.96,"bs":120,"bx":"Q","c":["R"],"t":"2026-05-07T13:30:00.017793788Z","z":"C"}
                ]
            }
        }"#;
        let parsed: MultiQuotesResponse = serde_json::from_str(json).unwrap();
        assert_eq!(parsed.quotes.len(), 1);
        assert_eq!(parsed.quotes["AAPL"].len(), 2);
        assert_eq!(parsed.quotes["AAPL"][0].ask_price, 289.29);
        assert_eq!(parsed.quotes["AAPL"][0].ask_size, 120);
        assert!(parsed.next_page_token.is_some());
    }

    #[test]
    fn deserializes_empty_multi_symbol_quotes_response() {
        let json = r#"{"quotes": {}, "next_page_token": null}"#;
        let parsed: MultiQuotesResponse = serde_json::from_str(json).unwrap();
        assert!(parsed.quotes.is_empty());
    }
}
