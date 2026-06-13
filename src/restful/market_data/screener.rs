use crate::restful::MarketDataClient;
use reqwest::Method;
use serde::{Deserialize, Serialize};

/// A most-active stock.
#[derive(Clone, Debug, Deserialize)]
#[non_exhaustive]
pub struct MostActive {
    /// The stock symbol.
    pub symbol: String,
    /// The trading volume.
    pub volume: u64,
    /// The number of trades.
    pub trade_count: u64,
}

#[derive(Debug, Deserialize)]
struct MostActivesResponse {
    most_actives: Vec<MostActive>,
    #[allow(dead_code)]
    last_updated: Option<String>,
}

/// A market mover (gainer or loser).
#[derive(Clone, Debug, Deserialize)]
#[non_exhaustive]
pub struct Mover {
    /// The stock symbol.
    pub symbol: String,
    /// The percentage change.
    pub percent_change: f64,
    /// The absolute price change.
    pub change: f64,
    /// The current price.
    pub price: f64,
}

#[derive(Debug, Deserialize)]
struct MoversResponse {
    gainers: Option<Vec<Mover>>,
    losers: Option<Vec<Mover>>,
    #[allow(dead_code)]
    last_updated: Option<String>,
}

/// Market movers result.
#[derive(Clone, Debug)]
#[non_exhaustive]
pub struct MarketMovers {
    /// Top gaining stocks.
    pub gainers: Vec<Mover>,
    /// Top losing stocks.
    pub losers: Vec<Mover>,
}

/// Market category supported by the movers screener.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
#[non_exhaustive]
pub enum MoverMarket {
    /// Equity market.
    Stocks,
    /// Crypto market.
    Crypto,
}

impl std::fmt::Display for MoverMarket {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Stocks => "stocks",
            Self::Crypto => "crypto",
        })
    }
}

/// Ranking metric for the most-actives screener.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum MostActivesBy {
    /// Rank by trading volume (the API default).
    Volume,
    /// Rank by number of trades.
    Trades,
}

impl MostActivesBy {
    fn wire(self) -> &'static str {
        match self {
            Self::Volume => "volume",
            Self::Trades => "trades",
        }
    }
}

impl MarketDataClient {
    /// Get most active stocks, ranked by volume or trade count.
    pub async fn most_actives(
        &self,
        limit: Option<usize>,
        by: Option<MostActivesBy>,
    ) -> crate::Result<Vec<MostActive>> {
        let request = self.most_actives_request(limit, by)?;
        let response: MostActivesResponse = self.send_and_deserialize(request).await?;
        Ok(response.most_actives)
    }

    fn most_actives_request(
        &self,
        limit: Option<usize>,
        by: Option<MostActivesBy>,
    ) -> crate::Result<reqwest::RequestBuilder> {
        let mut params: Vec<(&str, String)> = Vec::new();
        if let Some(limit) = limit {
            params.push(("top", limit.to_string()));
        }
        if let Some(by) = by {
            params.push(("by", by.wire().to_string()));
        }
        let mut request = self.request(Method::GET, "v1beta1/screener/stocks/most-actives")?;
        if !params.is_empty() {
            request = request.query(&params);
        }
        Ok(request)
    }

    /// Get top market movers (gainers and losers).
    pub async fn market_movers(
        &self,
        market: MoverMarket,
        limit: Option<usize>,
    ) -> crate::Result<MarketMovers> {
        let path = format!("v1beta1/screener/{market}/movers");
        let mut request = self.request(Method::GET, &path)?;
        if let Some(limit) = limit {
            request = request.query(&[("top", limit)]);
        }
        let response: MoversResponse = self.send_and_deserialize(request).await?;
        Ok(MarketMovers {
            gainers: response.gainers.unwrap_or_default(),
            losers: response.losers.unwrap_or_default(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::AccountType;
    use serial_test::serial;
    use std::env;

    fn paper_client() -> MarketDataClient {
        unsafe {
            if env::var("ALPACA_PAPER_API_KEY_ID").is_err() {
                env::set_var("ALPACA_PAPER_API_KEY_ID", "test_key_id");
            }
            if env::var("ALPACA_PAPER_API_SECRET_KEY").is_err() {
                env::set_var("ALPACA_PAPER_API_SECRET_KEY", "test_secret_key");
            }
        }
        MarketDataClient::new(AccountType::Paper).unwrap()
    }

    #[test]
    #[serial]
    fn most_actives_by_serializes_to_query() {
        let client = paper_client();
        let request = client
            .most_actives_request(Some(5), Some(MostActivesBy::Trades))
            .unwrap()
            .build()
            .unwrap();
        let query = request.url().query().unwrap();
        assert!(query.contains("top=5"), "{query}");
        assert!(query.contains("by=trades"), "{query}");
    }
}
