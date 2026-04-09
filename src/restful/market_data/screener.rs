use crate::restful::MarketDataClient;
use reqwest::Method;
use serde::Deserialize;

/// A most-active stock.
#[derive(Clone, Debug, Deserialize)]
pub struct MostActive {
    pub symbol: String,
    pub volume: u64,
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
    pub symbol: String,
    pub percent_change: f64,
    pub change: f64,
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
    pub gainers: Vec<Mover>,
    pub losers: Vec<Mover>,
}

impl MarketDataClient {
    /// Get most active stocks by volume.
    pub async fn most_actives(&self, top: Option<u32>) -> crate::Result<Vec<MostActive>> {
        let mut request = self.request(Method::GET, "v1beta1/screener/stocks/most-actives");
        if let Some(top) = top {
            request = request.query(&[("top", top)]);
        }
        let response: MostActivesResponse = self.send_and_deserialize(request).await?;
        Ok(response.most_actives)
    }

    /// Get top market movers (gainers and losers).
    pub async fn market_movers(
        &self,
        market_type: &str,
        top: Option<u32>,
    ) -> crate::Result<MarketMovers> {
        let path = format!("v1beta1/screener/{market_type}/movers");
        let mut request = self.request(Method::GET, &path);
        if let Some(top) = top {
            request = request.query(&[("top", top)]);
        }
        let response: MoversResponse = self.send_and_deserialize(request).await?;
        Ok(MarketMovers {
            gainers: response.gainers.unwrap_or_default(),
            losers: response.losers.unwrap_or_default(),
        })
    }
}
