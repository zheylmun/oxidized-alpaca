use crate::{Feed, restful::MarketDataClient};
use reqwest::Method;
use serde::{Deserialize, Serialize};

use super::{Bar, quotes::StockQuote, trades::StockTrade};

/// A stock snapshot containing latest trade, quote, and bar data.
#[derive(Clone, Debug, Deserialize)]
pub struct StockSnapshot {
    #[serde(rename = "latestTrade")]
    pub latest_trade: Option<StockTrade>,
    #[serde(rename = "latestQuote")]
    pub latest_quote: Option<StockQuote>,
    #[serde(rename = "minuteBar")]
    pub minute_bar: Option<Bar>,
    #[serde(rename = "dailyBar")]
    pub daily_bar: Option<Bar>,
    #[serde(rename = "prevDailyBar")]
    pub prev_daily_bar: Option<Bar>,
}

impl MarketDataClient {
    /// Get a snapshot for a single stock symbol.
    pub async fn stock_snapshot(
        &self,
        symbol: &str,
        feed: Option<Feed>,
    ) -> crate::Result<StockSnapshot> {
        let path = format!("v2/stocks/{symbol}/snapshot");
        let mut request = self.request(Method::GET, &path);
        if let Some(feed) = feed {
            #[derive(Serialize)]
            struct FeedQuery {
                feed: Feed,
            }
            request = request.query(&FeedQuery { feed });
        }
        self.send_and_deserialize(request).await
    }

    /// Get snapshots for multiple stock symbols.
    pub async fn stock_snapshots(
        &self,
        symbols: &[&str],
        feed: Option<Feed>,
    ) -> crate::Result<std::collections::HashMap<String, StockSnapshot>> {
        let mut request = self
            .request(Method::GET, "v2/stocks/snapshots")
            .query(&[("symbols", symbols.join(","))]);
        if let Some(feed) = feed {
            #[derive(Serialize)]
            struct FeedQuery {
                feed: Feed,
            }
            request = request.query(&FeedQuery { feed });
        }
        self.send_and_deserialize(request).await
    }
}
