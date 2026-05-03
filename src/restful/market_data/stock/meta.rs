use crate::restful::MarketDataClient;
use reqwest::Method;

/// Tick type for condition code lookups.
#[derive(Clone, Copy, Debug)]
pub enum TickType {
    /// Trade tick type.
    Trade,
    /// Quote tick type.
    Quote,
}

impl TickType {
    fn as_str(&self) -> &str {
        match self {
            Self::Trade => "trade",
            Self::Quote => "quote",
        }
    }
}

/// Tape identifier required by the conditions endpoint.
#[derive(Clone, Copy, Debug)]
pub enum Tape {
    /// Tape A — NYSE-listed securities.
    A,
    /// Tape B — NYSE Arca / regional exchange-listed securities.
    B,
    /// Tape C — NASDAQ-listed securities.
    C,
}

impl Tape {
    fn as_str(&self) -> &str {
        match self {
            Self::A => "A",
            Self::B => "B",
            Self::C => "C",
        }
    }
}

impl MarketDataClient {
    /// Get stock trade or quote condition codes for the given tape.
    pub async fn stock_conditions(
        &self,
        tick_type: TickType,
        tape: Tape,
    ) -> crate::Result<std::collections::HashMap<String, String>> {
        let path = format!("v2/stocks/meta/conditions/{}", tick_type.as_str());
        let request = self
            .request(Method::GET, &path)
            .query(&[("tape", tape.as_str())]);
        self.send_and_deserialize(request).await
    }

    /// Get stock exchange codes.
    pub async fn stock_exchanges(
        &self,
    ) -> crate::Result<std::collections::HashMap<String, String>> {
        let request = self.request(Method::GET, "v2/stocks/meta/exchanges");
        self.send_and_deserialize(request).await
    }
}
