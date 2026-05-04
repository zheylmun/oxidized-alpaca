use crate::restful::MarketDataClient;
use reqwest::Method;
use serde::Deserialize;

/// A most-active stock.
#[derive(Clone, Debug, Deserialize)]
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
pub struct MarketMovers {
    /// Top gaining stocks.
    pub gainers: Vec<Mover>,
    /// Top losing stocks.
    pub losers: Vec<Mover>,
}

/// Market category supported by the movers screener.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
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

impl MarketDataClient {
    /// Get most active stocks by volume.
    pub async fn most_actives(&self, limit: Option<u32>) -> crate::Result<Vec<MostActive>> {
        let mut request = self.request(Method::GET, "v1beta1/screener/stocks/most-actives")?;
        if let Some(limit) = limit {
            request = request.query(&[("top", limit)]);
        }
        let response: MostActivesResponse = self.send_and_deserialize(request).await?;
        Ok(response.most_actives)
    }

    /// Get top market movers (gainers and losers).
    pub async fn market_movers(
        &self,
        market: MoverMarket,
        limit: Option<u32>,
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
