use crate::{
    RestFeed,
    error::Error,
    restful::{MarketDataClient, SortDirection, null_def_vec},
};
use chrono::{DateTime, Utc};
use reqwest::Method;
use serde::{Deserialize, Serialize};

use super::{Adjustment, AdjustmentList, AsOf, Bar, TimeFrame};

/// A request for /v2/stocks/{symbol}/bars
#[derive(Debug, Serialize)]
#[must_use]
#[serde(rename_all = "snake_case")]
pub struct StockBarsRequest<'a> {
    /// The `MarketDataClient` to use.
    #[serde(skip)]
    client: &'a MarketDataClient,
    /// The symbol for which to retrieve market data.
    #[serde(skip)]
    symbol: String,
    /// The time frame for the bars.
    timeframe: TimeFrame,
    /// The maximum total number of bars to return across all pages.
    ///
    /// When unset all matching bars are returned.
    #[serde(skip_serializing_if = "Option::is_none")]
    limit: Option<usize>,
    /// Filter bars equal to or after this time.
    #[serde(skip_serializing_if = "Option::is_none")]
    start: Option<DateTime<Utc>>,
    /// Filter bars equal to or before this time.
    #[serde(skip_serializing_if = "Option::is_none")]
    end: Option<DateTime<Utc>>,
    /// The adjustment(s) to use (defaults to raw). Multiple values are
    /// comma-joined per Alpaca's docs (e.g. `split,dividend,spin-off`).
    #[serde(skip_serializing_if = "Option::is_none")]
    adjustment: Option<AdjustmentList>,
    /// The data feed to use.
    ///
    /// Defaults to [`IEX`][RestFeed::IEX] for free users and
    /// [`SIP`][RestFeed::SIP] for users with an unlimited subscription.
    #[serde(skip_serializing_if = "Option::is_none")]
    feed: Option<RestFeed>,
    /// Optional `asof` filter (date for symbol-mapping anchor or skip-mapping).
    #[serde(skip_serializing_if = "Option::is_none")]
    asof: Option<AsOf>,
    /// Optional ISO 4217 currency code (defaults to USD).
    #[serde(skip_serializing_if = "Option::is_none")]
    currency: Option<String>,
    /// Optional sort order (defaults to ascending).
    #[serde(skip_serializing_if = "Option::is_none")]
    sort: Option<SortDirection>,
    /// If provided we will pass a page token to continue where we left off.
    #[serde(skip_serializing_if = "Option::is_none")]
    page_token: Option<String>,
}

impl StockBarsRequest<'_> {
    /// Cap the total number of bars returned across all auto-paginated pages.
    pub fn limit(mut self, limit: usize) -> Self {
        self.limit = Some(limit);
        self
    }

    /// Set the `start` date for the bars request.
    pub fn start(mut self, start: DateTime<Utc>) -> Self {
        self.start = Some(start);
        self
    }

    /// Set the `end` date for the bars request.
    pub fn end(mut self, end: DateTime<Utc>) -> Self {
        self.end = Some(end);
        self
    }

    /// Set a single `adjustment` for the bars request.
    pub fn adjustment(mut self, adjustment: Adjustment) -> Self {
        self.adjustment = Some(adjustment.into());
        self
    }

    /// Set multiple `adjustment` values for the bars request. Alpaca
    /// accepts any combination of `raw`, `split`, `dividend`, `spin-off`,
    /// and `all` joined with commas. An empty iterator leaves the
    /// parameter unset, falling back to Alpaca's default of `raw`.
    pub fn adjustments<I: IntoIterator<Item = Adjustment>>(mut self, adjustments: I) -> Self {
        let list = AdjustmentList::new(adjustments);
        self.adjustment = if list.is_empty() { None } else { Some(list) };
        self
    }

    /// Set the `feed` for the bars request.
    pub fn feed(mut self, feed: RestFeed) -> Self {
        self.feed = Some(feed);
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

    /// Attempt to execute the configured request
    ///
    /// # Errors
    /// - Returns a [`Error::ReqwestSend`] if the rest request fails.
    /// - Returns a [`Error::ReqwestDeserialize`] if the response cannot be parsed
    #[cfg_attr(feature = "tracing", tracing::instrument)]
    pub async fn execute(mut self) -> Result<Vec<Bar>, Error> {
        let cap = self.limit;
        let mut results = Vec::new();
        loop {
            let response = self.internal_execute().await?;
            results.extend(response.bars);
            if let Some(cap) = cap
                && results.len() >= cap
            {
                results.truncate(cap);
                break;
            }
            match response.next_page_token {
                Some(token) => self.page_token = Some(token),
                None => break,
            }
        }
        Ok(results)
    }

    #[cfg_attr(feature = "tracing", tracing::instrument)]
    async fn internal_execute(&self) -> Result<Bars, Error> {
        let symbol = &self.symbol;
        let path = format!("v2/stocks/{symbol}/bars");
        let request = self.client.request(Method::GET, &path)?.query(&self);
        self.client.send_and_deserialize(request).await
    }
}

/// A collection of bars as returned by the API. This is one page of bars.
#[derive(Debug, Deserialize, PartialEq)]
struct Bars {
    /// The list of returned bars.
    #[serde(default, deserialize_with = "null_def_vec")]
    pub bars: Vec<Bar>,
    /// The symbol the bars correspond to.
    pub symbol: String,
    /// The token to provide to a request to get the next page of bars for this request.
    pub next_page_token: Option<String>,
}

impl MarketDataClient {
    /// Request historical bars for a single stock symbol.
    ///
    /// ```ignore
    /// let bars = client.stock_bars("AAPL", TimeFrame::OneDay)
    ///     .start(dt)
    ///     .limit(100)
    ///     .execute().await?;
    /// ```
    pub fn stock_bars<'a>(&'a self, symbol: &str, timeframe: TimeFrame) -> StockBarsRequest<'a> {
        StockBarsRequest {
            client: self,
            symbol: symbol.to_string(),
            timeframe,
            limit: None,
            start: None,
            end: None,
            adjustment: None,
            feed: None,
            asof: None,
            currency: None,
            sort: None,
            page_token: None,
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
    fn single_adjustment_serializes_in_query() {
        let client = paper_client();
        let request = client
            .stock_bars("AAPL", TimeFrame::OneDay)
            .adjustment(Adjustment::Split);
        let query = serde_urlencoded::to_string(&request).unwrap();
        assert!(
            query.contains("adjustment=split"),
            "expected `adjustment=split` in {query}"
        );
    }

    #[test]
    #[serial]
    fn multiple_adjustments_join_with_commas_in_query() {
        let client = paper_client();
        let request = client.stock_bars("AAPL", TimeFrame::OneDay).adjustments([
            Adjustment::Split,
            Adjustment::Dividend,
            Adjustment::SpinOff,
        ]);
        let query = serde_urlencoded::to_string(&request).unwrap();
        // `,` is percent-encoded as %2C in query strings.
        assert!(
            query.contains("adjustment=split%2Cdividend%2Cspin-off"),
            "expected joined adjustments in {query}"
        );
    }

    #[test]
    #[serial]
    fn empty_adjustments_omits_parameter() {
        let client = paper_client();
        let request = client
            .stock_bars("AAPL", TimeFrame::OneDay)
            .adjustments(std::iter::empty::<Adjustment>());
        let query = serde_urlencoded::to_string(&request).unwrap();
        assert!(
            !query.contains("adjustment"),
            "expected `adjustment` to be absent from {query}"
        );
    }
}
